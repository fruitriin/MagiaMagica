//! dev-server (Phase 2.1〜2.4, Phase 3.5 式トグル, Phase 4.0 ソース連動ビュー)。
//!
//! `magia serve <FILE>` でファイル全体を監視し、ブラウザに
//! 「ソース | 魔法陣」の左右ペインを配信する (spec §7 / Phase 4.0 計画)。
//!
//! 状態モデル (Phase 4.0):
//! - サーバは**直近の正常スナップショット** (ソース全文 + 関数索引) と
//!   現在の解析エラーだけを持つ。どの関数を見ているかは**クライアントの状態**
//!   (URL `?fn=`) であり、サーバは保持しない
//! - `/state` = ファイルメタ + 関数一覧 + エラー。`/spell/<fn>` = 関数単位の
//!   (svg 両式 + SH 済みソース + 書き起こし) をオンデマンドでレンダリング
//! - 構文エラー中は直近の正常スナップショットから配信し続ける (会話を切らない原則)
//!
//! 構成は同期スレッドモデル: tiny_http の受信ループ + リクエストごとのスレッド
//! (SSE 接続はスレッドを1本占有するが、ローカル開発ツールの同時接続数では問題ない)。

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use notify::{RecursiveMode, Watcher};

use magia_core::layout::layout;
use magia_core::proximity::{NeighborSeed, classify_neighbors};
use magia_core::render::{
    RenderStyle,
    ir_export::{NeighborMeta, focus_layout, spell_ir},
    render,
};
use magia_rust::{FunctionEntry, function_index, parse_function};

use crate::srcview::highlight_rust;

/// エディタの「テンポラリ書き込み → rename」保存で複数イベントが連射されるため、
/// 最後のイベントからこの時間だけ静まるのを待ってから1回だけ再解析する。
const DEBOUNCE: Duration = Duration::from_millis(120);

/// 直近の正常スナップショット。構文エラー中の配信はここから行う。
#[derive(Default)]
struct GoodSnapshot {
    source: String,
    functions: Vec<FunctionEntry>,
}

/// 現在の解析エラー (正常時は None)。
struct ServeError {
    message: String,
    /// 構文エラーの行 (1始まり)。ソースペインの案内に使う。
    line: Option<usize>,
}

/// ブラウザに配る共有状態。
struct Shared {
    /// 表示用ファイル名。
    file_label: String,
    good: Mutex<GoodSnapshot>,
    error: Mutex<Option<ServeError>>,
    /// 更新世代。クライアントはこの値の変化で再取得する。
    version: AtomicU64,
    /// SSE 接続中クライアントへの通知チャネル。
    clients: Mutex<Vec<Sender<u64>>>,
}

/// Mutex の poisoning から回復してロックを取る。
/// dev-server は一部スレッドのパニック後も「壊れたまま応答し続ける」より
/// 「最後の正常状態で応答し続ける」方が望ましい (クラッシュループの回避)。
fn lock_or_recover<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl Shared {
    fn new(file_label: String) -> Self {
        Self {
            file_label,
            good: Mutex::new(GoodSnapshot::default()),
            error: Mutex::new(None),
            version: AtomicU64::new(0),
            clients: Mutex::new(Vec::new()),
        }
    }

    /// ファイルを読み直してスナップショットを更新し、SSE クライアントへ通知する。
    fn reload(&self, file: &Path) {
        match load_snapshot(file) {
            Ok(snapshot) => {
                *lock_or_recover(&self.good) = snapshot;
                *lock_or_recover(&self.error) = None;
            }
            Err(error) => {
                // スナップショットは触らない: 直前の正常な内容を配信し続ける (spec §7)。
                *lock_or_recover(&self.error) = Some(error);
            }
        }
        let version = self.version.fetch_add(1, Ordering::SeqCst) + 1;
        lock_or_recover(&self.clients).retain(|client| client.send(version).is_ok());
    }

    /// `/state`: ファイルメタ + 関数一覧 + エラー (魔法陣・ソースは `/spell/<fn>` 側)。
    fn state_json(&self) -> String {
        let good = lock_or_recover(&self.good);
        let functions: Vec<_> = good
            .functions
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "qualified": entry.qualified,
                    "name": entry.name,
                    "impl_context": entry.impl_context,
                    "signature": entry.signature,
                    "start_line": entry.start_line,
                    "end_line": entry.end_line,
                })
            })
            .collect();
        let error = lock_or_recover(&self.error);
        serde_json::json!({
            "version": self.version.load(Ordering::SeqCst),
            "file": self.file_label,
            "functions": functions,
            "error": error.as_ref().map(|e| serde_json::json!({
                "message": e.message,
                "line": e.line,
            })),
        })
        .to_string()
    }

    /// `/spell/<fn>`: 関数単位のレンダリング (オンデマンド、サーバに選択状態を持たない)。
    /// 不明な関数は `None` (404 は呼び出し側の責務)。
    fn spell_json(&self, qualified: &str, with_neighbors: bool) -> Option<Result<String, String>> {
        // レンダリング (parse + layout + render ×2 + syntect) は重いので、
        // ロックは複製を取るまでに留める — reload (保存) を /spell がブロックしない。
        let (source, entry, all) = {
            let good = lock_or_recover(&self.good);
            let entry = good
                .functions
                .iter()
                .find(|entry| entry.qualified == qualified)?
                .clone();
            let all = if with_neighbors {
                good.functions.clone()
            } else {
                Vec::new()
            };
            (good.source.clone(), entry, all)
        };
        Some(render_spell(
            &source,
            &entry,
            with_neighbors.then_some(all.as_slice()),
        ))
    }
}

/// ファイルを読み、関数索引つきのスナップショットを作る。
fn load_snapshot(file: &Path) -> Result<GoodSnapshot, ServeError> {
    let source = std::fs::read_to_string(file).map_err(|e| ServeError {
        message: format!("ファイルを読み込めません: {}: {e}", file.display()),
        line: None,
    })?;
    let functions = function_index(&source).map_err(|e| ServeError {
        line: syntax_error_line(&e),
        message: e.to_string(),
    })?;
    Ok(GoodSnapshot { source, functions })
}

/// 構文エラーの行番号 (ソースペインの案内に使う)。
fn syntax_error_line(error: &magia_rust::Error) -> Option<usize> {
    match error {
        magia_rust::Error::Syntax(syntax) => Some(syntax.span().start().line),
        magia_rust::Error::FunctionNotFound { .. } => None,
    }
}

/// 関数1つ分の応答 JSON を組み立てる。Err は内部不整合 (500 相当)。
///
/// `source_html` は Phase 4.0.5 (Vue のソースペイン) が使う先行実装で、
/// 現行 UI (薄い継ぎ目 JS) はまだ表示しない — API 契約として先に固定しておく。
fn render_spell(
    source: &str,
    entry: &FunctionEntry,
    neighbors_from: Option<&[FunctionEntry]>,
) -> Result<String, String> {
    let graph = parse_function(source, &entry.qualified).map_err(|e| e.to_string())?;
    let placed = layout(&graph);
    let snippet = source_lines(source, entry.start_line, entry.end_line);
    // Phase 4.0.9: ミッドチルダ式は配置済み IR (JSON) で返し、Vue が描画する。
    // SVG 文字列は出さない (v1.0 前は旧を消す)。ベルカ式の Vue 移植は Phase 4.3 で
    // 行うため、svg_belka のみ SVG 文字列を温存する。
    let spell = spell_ir(&graph, &placed);
    // Phase 4.1: ピン中心ビューの周辺配置。近接度はスタブ (proximity.rs、4.2 で本実装)。
    let focus_layout = neighbors_from.map(|all| {
        let seed = |e: &FunctionEntry| NeighborSeed {
            qualified: e.qualified.clone(),
            impl_context: e.impl_context.clone(),
        };
        let classified =
            classify_neighbors(&seed(entry), &all.iter().map(seed).collect::<Vec<_>>());
        let with_meta: Vec<_> = classified
            .into_iter()
            .filter_map(|neighbor| {
                let meta = all.iter().find(|e| e.qualified == neighbor.qualified)?;
                Some((
                    neighbor,
                    NeighborMeta {
                        name: meta.name.clone(),
                        signature: meta.signature.clone(),
                    },
                ))
            })
            .collect();
        focus_layout(spell.view_box, &with_meta)
    });
    let ir = serde_json::to_value(spell).map_err(|e| e.to_string())?;
    let mut response = serde_json::json!({
        "qualified": entry.qualified,
        "signature": entry.signature,
        "ir": ir,
        "svg_belka": render(&graph, &placed, RenderStyle::Belka),
        "source_html": highlight_rust(&snippet),
        "transcript": magia_core::transcript::transcribe(&graph),
        "start_line": entry.start_line,
    });
    if let Some(layout) = focus_layout {
        response["focus_layout"] = serde_json::to_value(layout).map_err(|e| e.to_string())?;
    }
    Ok(response.to_string())
}

/// 1始まり・両端含みの行範囲を切り出す (関数スニペット)。
fn source_lines(source: &str, start_line: usize, end_line: usize) -> String {
    // span が正常なら end >= start。壊れた入力でも最低1行は返す防御。
    let end_line = end_line.max(start_line);
    let mut snippet: String = source
        .lines()
        .skip(start_line.saturating_sub(1))
        .take(end_line.saturating_sub(start_line) + 1)
        .collect::<Vec<_>>()
        .join("\n");
    snippet.push('\n');
    snippet
}

/// dev-server を起動する。Ctrl-C で終了するまで戻らない。
pub(crate) fn run(file: &Path, port: u16) -> Result<()> {
    let file = file.to_path_buf();
    let label = file.display().to_string();
    let shared = Arc::new(Shared::new(label));

    // 初回読み込み (エラーでもサーバは起動し、画面にエラーを出す)。
    shared.reload(&file);

    spawn_watcher(Arc::clone(&shared), file.clone())?;

    let server = tiny_http::Server::http(("127.0.0.1", port))
        .map_err(|e| anyhow::anyhow!("サーバを起動できません: {e}"))?;
    let actual_port = match server.server_addr() {
        tiny_http::ListenAddr::IP(addr) => addr.port(),
        // 本実装は IP しか bind しないため到達しない (防御)。
        tiny_http::ListenAddr::Unix(_) => port,
    };
    println!(
        "serving http://127.0.0.1:{actual_port}/  ({})",
        file.display()
    );
    // パイプ接続時はブロックバッファリングされるため明示 flush (テストが URL 行を待つ)。
    std::io::stdout().flush().ok();

    for request in server.incoming_requests() {
        let shared = Arc::clone(&shared);
        std::thread::spawn(move || handle_request(request, &shared));
    }
    Ok(())
}

/// ファイル監視スレッドを起動する。
///
/// エディタの rename 保存 (テンポラリ書き込み → 置換) でも検知できるよう、
/// ファイルそのものではなく**親ディレクトリ**を非再帰で監視し、
/// イベントのパスをファイル名で照合する。
fn spawn_watcher(shared: Arc<Shared>, file: PathBuf) -> Result<()> {
    let parent = file
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let watched_name = file
        .file_name()
        .context("監視対象のファイル名を特定できません")?
        .to_os_string();

    let (tx, rx) = channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = tx.send(event);
    })
    .context("ファイルウォッチャを作成できません")?;
    watcher
        .watch(&parent, RecursiveMode::NonRecursive)
        .with_context(|| format!("監視を開始できません: {}", parent.display()))?;

    std::thread::spawn(move || {
        // watcher はこのスレッドが握り続ける (drop すると監視が止まる)。
        let _watcher = watcher;
        while let Ok(event) = rx.recv() {
            if !event_touches(&event, &watched_name) {
                continue;
            }
            // デバウンス: 静まるまで追加イベントを飲み込む。
            while rx.recv_timeout(DEBOUNCE).is_ok() {}
            shared.reload(&file);
        }
    });
    Ok(())
}

fn event_touches(event: &notify::Result<notify::Event>, watched_name: &std::ffi::OsStr) -> bool {
    match event {
        Ok(event) => event
            .paths
            .iter()
            .any(|path| path.file_name() == Some(watched_name)),
        // 監視エラーは「変化したかもしれない」側に倒して再解析する。
        Err(_) => true,
    }
}

// ===== HTTP =====

/// Vue SPA (web/dist)。build.rs が bun でビルドして同梱する (Phase 4.0.5 M5)。
/// 開発時は vite+ dev (HMR) を使い、この埋め込みは本番配信専用。
#[derive(rust_embed::Embed)]
#[folder = "../../web/dist"]
struct WebDist;

fn handle_request(request: tiny_http::Request, shared: &Shared) {
    let url = request.url().to_string();
    // ルーティングはパス部分のみで行う (`?fn=...` 等の状態クエリが付いても返す)。
    // split は常に1要素以上返すため unwrap_or は不達 (防御のみ)。
    let path = url.split('?').next().unwrap_or("/").to_string();
    let result = match (request.method(), path.as_str()) {
        (tiny_http::Method::Get, "/") => request.respond(embedded_response("index.html")),
        (tiny_http::Method::Get, "/state") => request.respond(
            tiny_http::Response::from_string(shared.state_json())
                .with_header(header("Content-Type", "application/json; charset=utf-8")),
        ),
        (tiny_http::Method::Get, spell) if spell.starts_with("/spell/") => {
            let qualified = percent_decode(&spell["/spell/".len()..]);
            // ?with=neighbors でピン中心ビューの周辺配置 (focus_layout) を併載する
            // (Phase 4.1)。なしは従来契約のまま (静的レンダ・テスト互換)。
            let with_neighbors = url
                .split_once('?')
                .is_some_and(|(_, query)| query.split('&').any(|kv| kv == "with=neighbors"));
            request.respond(spell_response(shared, &qualified, with_neighbors))
        }
        (tiny_http::Method::Get, "/events") => {
            let (tx, rx) = channel::<u64>();
            {
                // 新規接続のたびに切断済みクライアントを掃除する (publish が無い間の
                // Sender リーク防止)。現世代を送って生存判定を兼ねる (受信側は
                // 冪等な refresh をするだけなので余分な1イベントは無害)。
                // 既知の制約: 切断済み Sender は次の reload か新規接続まで残る
                // (ローカル開発ツールの接続数では許容)。
                let version = shared.version.load(Ordering::SeqCst);
                let mut clients = lock_or_recover(&shared.clients);
                clients.retain(|client| client.send(version).is_ok());
                clients.push(tx);
            }
            // SSE は tiny_http の Response 経路を使わない。チャンク転送経路は
            // chunked_transfer::Encoder (8KB) と BufWriter (1KB) の二重バッファが
            // flush されず、イベントがクライアントに届かないため (Phase 4.0.5 M2 で
            // 発見した Phase 2.1 からの潜在バグ)。生 writer に自前でヘッダを書き、
            // イベントごとに flush する。
            stream_sse(request.into_writer(), &rx);
            return;
        }
        // API 以外は SPA の静的ファイル (/assets/*.js, /favicon.svg, ...)。
        // 注: SPA のルートは現状 `/` 1本のため index.html フォールバックは持たない。
        // Vue Router にパスベースのビューを足すときは 404 → index.html のフォールバックが要る。
        (tiny_http::Method::Get, file) => {
            request.respond(embedded_response(file.trim_start_matches('/')))
        }
        _ => request.respond(tiny_http::Response::from_string("not found").with_status_code(404)),
    };
    // クライアント切断は正常系 (SSE はブラウザを閉じれば必ずここに来る)。
    let _ = result;
}

/// 埋め込み済み SPA ファイルの応答 (200 / 404)。
fn embedded_response(file: &str) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    match WebDist::get(file) {
        Some(content) => {
            // 拡張子 → Content-Type の最小マップ (dist に現れる種類だけで足りる)。
            let mime = match file.rsplit('.').next() {
                Some("html") => "text/html; charset=utf-8",
                Some("js") => "text/javascript; charset=utf-8",
                Some("css") => "text/css; charset=utf-8",
                Some("svg") => "image/svg+xml",
                Some("png") => "image/png",
                Some("json") => "application/json; charset=utf-8",
                _ => "application/octet-stream",
            };
            tiny_http::Response::from_data(content.data.into_owned())
                .with_header(header("Content-Type", mime))
        }
        None => tiny_http::Response::from_data(b"not found".to_vec()).with_status_code(404),
    }
}

/// SSE ストリームを生 writer へ書き続ける。クライアント切断 (write/flush エラー)
/// またはサーバ終了 (Sender 全 drop) まで返らない。
/// `Connection: close` でこの接続をイベント専用にする (keep-alive の次リクエストを
/// 同一接続に載せさせない — レスポンスは EOF 終端のストリームのため)。
fn stream_sse(mut writer: Box<dyn Write + Send>, rx: &Receiver<u64>) {
    let send = |writer: &mut Box<dyn Write + Send>, payload: &str| -> std::io::Result<()> {
        writer.write_all(payload.as_bytes())?;
        writer.flush()
    };
    // 接続直後に1イベント流し、EventSource の onmessage → refresh を促す。
    // ペイロードの「0」に意味はない (クライアントは値を見ず refresh するだけ)。
    // 将来 version を差分処理に使う場合は接続直後イベントを実世代に変えること。
    if send(
        &mut writer,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\ndata: 0\n\n",
    )
    .is_err()
    {
        return;
    }
    while let Ok(version) = rx.recv() {
        if send(&mut writer, &format!("data: {version}\n\n")).is_err() {
            return; // クライアント切断 (正常系)
        }
    }
    // サーバ側終了 → writer drop で接続クローズ
}

/// `/spell/<fn>` の応答 (200 / 404 / 500)。
fn spell_response(
    shared: &Shared,
    qualified: &str,
    with_neighbors: bool,
) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    let json_header = header("Content-Type", "application/json; charset=utf-8");
    match shared.spell_json(qualified, with_neighbors) {
        Some(Ok(json)) => tiny_http::Response::from_string(json).with_header(json_header),
        Some(Err(message)) => {
            tiny_http::Response::from_string(serde_json::json!({ "error": message }).to_string())
                .with_status_code(500)
                .with_header(json_header)
        }
        None => tiny_http::Response::from_string(
            serde_json::json!({ "error": format!("関数 `{qualified}` は索引にありません") })
                .to_string(),
        )
        .with_status_code(404)
        .with_header(json_header),
    }
}

fn header(field: &str, value: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(field.as_bytes(), value.as_bytes()).expect("静的ヘッダは常に有効")
}

/// URL パスセグメントの最小パーセントデコード (`Foo%3A%3Abar` → `Foo::bar`)。
/// 不正なエンコードは文字をそのまま通す (防御的、未知名は 404 側で吸収される)。
fn percent_decode(segment: &str) -> String {
    let bytes = segment.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = bytes.get(i + 1..i + 3)
            && let Ok(hex_str) = std::str::from_utf8(hex)
            && let Ok(value) = u8::from_str_radix(hex_str, 16)
        {
            out.push(value);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_decode_handles_qualified_names() {
        assert_eq!(percent_decode("Foo%3A%3Abar"), "Foo::bar");
        assert_eq!(percent_decode("plain_name"), "plain_name");
    }

    #[test]
    fn percent_decode_passes_incomplete_sequences_through() {
        // 不正・不完全なエンコードは文字をそのまま通す (404 側で吸収される)。
        assert_eq!(percent_decode("Foo%3"), "Foo%3");
        assert_eq!(percent_decode("Foo%"), "Foo%");
        assert_eq!(percent_decode("Foo%ZZ"), "Foo%ZZ");
    }

    #[test]
    fn source_lines_clamps_reversed_range() {
        assert_eq!(source_lines("a\nb\nc\n", 2, 1), "b\n");
    }
}
