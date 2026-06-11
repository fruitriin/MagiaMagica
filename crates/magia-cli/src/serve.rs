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

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use notify::{RecursiveMode, Watcher};

use magia_core::layout::layout;
use magia_core::render::{RenderStyle, render};
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
    fn spell_json(&self, qualified: &str) -> Option<Result<String, String>> {
        // レンダリング (parse + layout + render ×2 + syntect) は重いので、
        // ロックは複製を取るまでに留める — reload (保存) を /spell がブロックしない。
        let (source, entry) = {
            let good = lock_or_recover(&self.good);
            let entry = good
                .functions
                .iter()
                .find(|entry| entry.qualified == qualified)?
                .clone();
            (good.source.clone(), entry)
        };
        Some(render_spell(&source, &entry))
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
fn render_spell(source: &str, entry: &FunctionEntry) -> Result<String, String> {
    let graph = parse_function(source, &entry.qualified).map_err(|e| e.to_string())?;
    let placed = layout(&graph);
    let snippet = source_lines(source, entry.start_line, entry.end_line);
    Ok(serde_json::json!({
        "qualified": entry.qualified,
        "signature": entry.signature,
        "svg": render(&graph, &placed, RenderStyle::MidchildaConcentric),
        "svg_belka": render(&graph, &placed, RenderStyle::Belka),
        "source_html": highlight_rust(&snippet),
        "transcript": magia_core::transcript::transcribe(&graph),
        "start_line": entry.start_line,
    })
    .to_string())
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

fn handle_request(request: tiny_http::Request, shared: &Shared) {
    let url = request.url().to_string();
    // ルーティングはパス部分のみで行う (`?fn=...` 等の状態クエリが付いても返す)。
    let path = url.split('?').next().unwrap_or("/").to_string();
    let result = match (request.method(), path.as_str()) {
        (tiny_http::Method::Get, "/") => request.respond(
            tiny_http::Response::from_string(INDEX_HTML)
                .with_header(header("Content-Type", "text/html; charset=utf-8")),
        ),
        (tiny_http::Method::Get, "/state") => request.respond(
            tiny_http::Response::from_string(shared.state_json())
                .with_header(header("Content-Type", "application/json; charset=utf-8")),
        ),
        (tiny_http::Method::Get, spell) if spell.starts_with("/spell/") => {
            let qualified = percent_decode(&spell["/spell/".len()..]);
            request.respond(spell_response(shared, &qualified))
        }
        (tiny_http::Method::Get, "/events") => {
            let (tx, rx) = channel::<u64>();
            {
                // 新規接続のたびに切断済みクライアントを掃除する (publish が無い間の
                // Sender リーク防止)。現世代を送って生存判定を兼ねる (受信側は
                // 冪等な refresh をするだけなので余分な1イベントは無害)。
                let version = shared.version.load(Ordering::SeqCst);
                let mut clients = lock_or_recover(&shared.clients);
                clients.retain(|client| client.send(version).is_ok());
                clients.push(tx);
            }
            let response = tiny_http::Response::new(
                tiny_http::StatusCode(200),
                vec![
                    header("Content-Type", "text/event-stream"),
                    header("Cache-Control", "no-cache"),
                ],
                SseStream::new(rx),
                None, // 長さ不定 = チャンク転送
                None,
            );
            request.respond(response)
        }
        _ => request.respond(tiny_http::Response::from_string("not found").with_status_code(404)),
    };
    // クライアント切断は正常系 (SSE はブラウザを閉じれば必ずここに来る)。
    let _ = result;
}

/// `/spell/<fn>` の応答 (200 / 404 / 500)。
fn spell_response(
    shared: &Shared,
    qualified: &str,
) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    let json_header = header("Content-Type", "application/json; charset=utf-8");
    match shared.spell_json(qualified) {
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

/// SSE のイベント列を `Read` として供給するアダプタ。
/// 更新通知が来るたびに `data: <version>` イベントを1つ吐く。
/// 送信側が drop されたら EOF (接続終了)。
struct SseStream {
    rx: Receiver<u64>,
    pending: Vec<u8>,
}

impl SseStream {
    fn new(rx: Receiver<u64>) -> Self {
        // 接続直後に1イベント流し、EventSource の onmessage → refresh を促す。
        // ペイロードの「0」に意味はない (クライアントは値を見ず refresh するだけ)。
        // 将来 version を差分処理に使う場合は接続直後イベントを実世代に変えること。
        Self {
            rx,
            pending: b"data: 0\n\n".to_vec(),
        }
    }
}

impl Read for SseStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pending.is_empty() {
            match self.rx.recv() {
                Ok(version) => self.pending = format!("data: {version}\n\n").into_bytes(),
                Err(_) => return Ok(0), // サーバ側終了 → EOF
            }
        }
        let n = self.pending.len().min(buf.len());
        buf[..n].copy_from_slice(&self.pending[..n]);
        self.pending.drain(..n);
        Ok(n)
    }
}

// ===== フロントエンド =====

// Phase 4.0 注記: ペアビュー UI (関数一覧・ソースペイン) は Phase 4.0.5 (Vue 基盤) で
// 実装する (オーナー方針 2026-06-11、素 JS の二度書き回避)。本テンプレートは Phase 2.x
// の UI を維持し、新 API (/state = メタ, /spell/<fn> = 描画素材) への薄い継ぎ目のみ持つ。
// 注意: raw string の都合で本テンプレートに `"` + `#` の連接を書かないこと。
const INDEX_HTML: &str = r#"<!doctype html>
<html lang="ja"><head><meta charset="utf-8"><title>MagiaMagica dev-server</title>
<style>
  body { margin: 0; font-family: system-ui, sans-serif; }
  #status { color: #d92626; padding: 4px 12px; min-height: 1.2em; font-size: 14px; white-space: pre-wrap; }
  #magia { display: flex; justify-content: center; }
  #magia svg { max-width: 100vw; max-height: 88vh; }
  #palette { position: fixed; top: 12px; right: 12px; background: #fffd; border: 1px solid #ccc;
             border-radius: 8px; padding: 10px 14px; font-size: 13px; box-shadow: 0 2px 8px #0002; }
  #palette .layer-row { display: flex; align-items: center; gap: 6px; margin: 6px 0; }
  #palette input[type="range"] { width: 80px; }
  #palette .buttons { display: flex; gap: 6px; margin-top: 8px; }
  #palette textarea { width: 200px; font-family: ui-monospace, monospace; font-size: 12px; }
  #dsl-note { color: #7a4a1c; font-size: 11px; min-height: 1em; white-space: pre-wrap; }
  #dsl-box { margin-top: 8px; }
  /* スクリーンリーダーにのみ露出する書き起こし (spec v0.2 §15)。
     clip (旧) と clip-path (新) を併記し、margin/padding/border を明示する完全形。 */
  .visually-hidden { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
                     overflow: hidden; clip: rect(0, 0, 0, 0); clip-path: inset(50%);
                     white-space: nowrap; border: 0; }
</style></head>
<body>
<div id="status"></div>
<div id="transcript" class="visually-hidden" role="region" aria-label="呪文書き起こし"></div>
<div id="palette">
  <strong>式</strong>
  <div class="layer-row">
    <label><input type="radio" name="style" data-style="midchilda" checked> ミッドチルダ</label>
    <label><input type="radio" name="style" data-style="belka"> ベルカ</label>
  </div>
  <strong>レイヤー</strong>
  <div class="layer-row"><label><input type="checkbox" data-layer="control_flow" checked> 制御フロー</label>
    <input type="range" data-opacity="control_flow" min="0" max="1" step="0.05" value="1"></div>
  <div class="layer-row"><label><input type="checkbox" data-layer="effects" checked> 効果</label>
    <input type="range" data-opacity="effects" min="0" max="1" step="0.05" value="1"></div>
  <div class="layer-row"><label><input type="checkbox" data-layer="type_info" checked> 型情報</label>
    <input type="range" data-opacity="type_info" min="0" max="1" step="0.05" value="1"></div>
  <div class="buttons"><button id="all-on">全表示</button><button id="all-off">全非表示</button></div>
  <details id="dsl-box"><summary>.magia (spec §8)</summary>
    <textarea id="dsl" rows="3" spellcheck="false"></textarea>
    <div class="buttons"><button id="dsl-export">エクスポート</button><button id="dsl-apply">適用</button></div>
    <div id="dsl-note"></div>
  </details>
</div>
<div id="magia"></div>
<script>
const LAYERS = ['control_flow', 'effects', 'type_info'];
const cssClass = (layer) => 'layer-' + layer.replace(/_/g, '-');
// 状態: visible = 表示中レイヤーの集合、opacity = レイヤー別透明度 (既定 1)
let visible = new Set(LAYERS);
let opacity = {};
// 式 (spec v0.3 §14.4)。切替は表示する SVG の選択のみで、両式ともサーバが常に生成する。
let style = 'midchilda';
let lastSpell = null;   // /spell/<fn> の最新応答
let currentFn = null;   // qualified 名。URL ?fn= が真実 (Phase 4.0 の薄い継ぎ目)

function readQuery() {
  const params = new URLSearchParams(location.search);
  style = params.get('style') === 'belka' ? 'belka' : 'midchilda';
  currentFn = params.get('fn');
  const layers = params.get('layers');
  visible = layers === null ? new Set(LAYERS)
                            : new Set(layers.split(',').filter(l => LAYERS.includes(l)));
  opacity = {};
  for (const pair of (params.get('op') || '').split(',').filter(Boolean)) {
    const sep = pair.indexOf(':');
    if (sep === -1) { continue; }
    const layer = pair.slice(0, sep);
    const value = parseFloat(pair.slice(sep + 1));
    if (LAYERS.includes(layer) && !isNaN(value)) { opacity[layer] = value; }
  }
}

function writeQuery() {
  const params = new URLSearchParams();
  if (currentFn) { params.set('fn', currentFn); }
  if (style === 'belka') { params.set('style', 'belka'); }
  if (visible.size < LAYERS.length) { params.set('layers', LAYERS.filter(l => visible.has(l)).join(',')); }
  const ops = LAYERS.filter(l => opacity[l] !== undefined && Math.abs(opacity[l] - 1) > 1e-9)
                    .map(l => l + ':' + opacity[l]).join(',');
  if (ops) { params.set('op', ops); }
  const query = params.toString();
  history.replaceState(null, '', query ? '?' + query : location.pathname);
}

// SVG (差し替え後も含む) とパレット UI に状態を反映する。位置は一切変えない
// (spec §5.4 位置共有制約: 切替は CSS のみで、レイアウトに影響しない)。
function apply() {
  for (const layer of LAYERS) {
    for (const group of document.querySelectorAll('#magia g.' + cssClass(layer))) {
      group.style.display = visible.has(layer) ? '' : 'none';
      group.style.opacity = opacity[layer] ?? 1;
    }
    const checkbox = document.querySelector(`input[data-layer="${layer}"]`);
    const slider = document.querySelector(`input[data-opacity="${layer}"]`);
    if (checkbox) { checkbox.checked = visible.has(layer); }
    if (slider) { slider.value = opacity[layer] ?? 1; }
  }
  for (const radio of document.querySelectorAll('input[data-style]')) {
    radio.checked = radio.dataset.style === style;
  }
}

// 選択中の式の SVG を描画する (レイヤー切替はミッドチルダ式の <g> にのみ効く)。
function show() {
  if (!lastSpell) { return; }
  const svg = style === 'belka' ? lastSpell.svg_belka : lastSpell.svg;
  if (svg) {
    const doc = new DOMParser().parseFromString(svg, 'image/svg+xml');
    document.getElementById('magia').replaceChildren(doc.documentElement);
  }
  apply();
}

document.getElementById('palette').addEventListener('input', (event) => {
  const layer = event.target.dataset.layer;
  const opacityLayer = event.target.dataset.opacity;
  const styleChoice = event.target.dataset.style;
  if (!layer && !opacityLayer && !styleChoice) { return; } // 将来の入力欄でも誤発火しない
  if (layer) { event.target.checked ? visible.add(layer) : visible.delete(layer); }
  if (opacityLayer) { opacity[opacityLayer] = parseFloat(event.target.value); }
  if (styleChoice) { style = styleChoice; }
  writeQuery();
  if (styleChoice) { show(); } else { apply(); }
});
document.getElementById('all-on').addEventListener('click', () => { visible = new Set(LAYERS); writeQuery(); apply(); });
document.getElementById('all-off').addEventListener('click', () => { visible = new Set(); writeQuery(); apply(); });

// .magia (spec §8) との往復。パレットが扱うのは可視性のみで、effects[カテゴリ] の
// 絞り込みは render 時適用のため CLI (`magia render --filter`) を案内する。
document.getElementById('dsl-export').addEventListener('click', () => {
  const shown = LAYERS.filter(l => visible.has(l));
  document.getElementById('dsl').value =
    shown.length ? 'show: ' + shown.join(' + ') : '# 全レイヤー非表示\nhide: ' + LAYERS.join(' + ');
  document.getElementById('dsl-note').textContent = '';
});
document.getElementById('dsl-apply').addEventListener('click', () => {
  const note = document.getElementById('dsl-note');
  note.textContent = '';
  let show = null; const hide = new Set(); let hasCategories = false;
  for (const [i, raw] of document.getElementById('dsl').value.split('\n').entries()) {
    const line = raw.trim();
    if (!line || line.startsWith('#')) { continue; }
    const directive = line.startsWith('show:') ? 'show' : line.startsWith('hide:') ? 'hide' : null;
    if (!directive) { note.textContent = (i + 1) + '行目: show: / hide: のみ使用できます'; return; }
    for (const part of line.slice(5).split('+').map(s => s.trim()).filter(Boolean)) {
      if (part.includes('[')) {
        if (directive === 'hide') { note.textContent = (i + 1) + '行目: hide にカテゴリ指定 [...] はできません'; return; }
        hasCategories = true;
      }
      const name = part.split('[')[0].trim();
      if (!LAYERS.includes(name)) { note.textContent = (i + 1) + '行目: 未知のレイヤー名 `' + name + '`'; return; }
      if (directive === 'show') { (show ??= new Set()).add(name); } else { hide.add(name); }
    }
  }
  visible = new Set((show ? [...show] : LAYERS).filter(l => !hide.has(l)));
  if (hasCategories) { note.textContent = 'effects[カテゴリ] の絞り込みは magia render --filter で適用されます'; }
  writeQuery(); apply();
});

// Phase 4.0 の薄い継ぎ目: /state はメタ (関数一覧 + エラー)、描画素材は /spell/<fn>。
// ペアビュー UI (関数一覧・ソースペイン) は Phase 4.0.5 (Vue) で実装する。
async function refresh() {
  const state = await (await fetch('/state')).json();
  const message = state.error
    ? state.error.message + (state.error.line ? ' (' + state.error.line + '行目)' : '')
    : '';
  document.getElementById('status').textContent = message;
  const names = (state.functions || []).map(f => f.qualified);
  if (!currentFn || !names.includes(currentFn)) {
    currentFn = names[0] || null;
    writeQuery();
  }
  if (!currentFn) { return; }
  const response = await fetch('/spell/' + encodeURIComponent(currentFn));
  if (!response.ok) { return; }
  lastSpell = await response.json();
  document.getElementById('transcript').textContent = lastSpell.transcript || '';
  // innerHTML でなく DOMParser を使う (show 内): SVG 内に万一スクリプト類が
  // 混入しても実行されない (レンダラは XML エスケープ済みだが多層防御として)。
  show();
}
readQuery();
refresh();
new EventSource('/events').onmessage = refresh;
</script>
</body></html>
"#;

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
