//! dev-server (Phase 2.1, spec v0.2 §7)。
//!
//! `magia serve <FILE> --fn <NAME>` で常駐し、ブラウザに魔法陣を表示する。
//! ファイル保存を検知して再レンダリングし、SSE でブラウザへ更新を push する
//! (spec §7 は WebSocket または SSE を許容。同期サーバで完結する SSE を採用)。
//! 解析エラー中は**直前の正常な SVG を保持**したままエラーメッセージを重ねる
//! (コードとの会話を切らない、notes §1.1)。
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
use magia_rust::parse_function;

/// エディタの「テンポラリ書き込み → rename」保存で複数イベントが連射されるため、
/// 最後のイベントからこの時間だけ静まるのを待ってから1回だけ再レンダリングする。
const DEBOUNCE: Duration = Duration::from_millis(120);

/// ブラウザに配る共有状態。
struct Shared {
    /// 直近の正常な SVG (エラー中も保持する)。
    svg: Mutex<String>,
    /// 現在の解析エラー (正常時は None)。
    error: Mutex<Option<String>>,
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
    fn new() -> Self {
        Self {
            svg: Mutex::new(String::new()),
            error: Mutex::new(None),
            version: AtomicU64::new(0),
            clients: Mutex::new(Vec::new()),
        }
    }

    /// 状態を更新して世代を進め、SSE クライアントへ通知する。
    fn publish(&self, rendered: Result<String, String>) {
        match rendered {
            Ok(svg) => {
                *lock_or_recover(&self.svg) = svg;
                *lock_or_recover(&self.error) = None;
            }
            Err(message) => {
                // SVG は触らない: 直前の正常な図を保持する (spec §7)。
                *lock_or_recover(&self.error) = Some(message);
            }
        }
        let version = self.version.fetch_add(1, Ordering::SeqCst) + 1;
        lock_or_recover(&self.clients).retain(|client| client.send(version).is_ok());
    }

    fn state_json(&self) -> String {
        serde_json::json!({
            "version": self.version.load(Ordering::SeqCst),
            "svg": *lock_or_recover(&self.svg),
            "error": *lock_or_recover(&self.error),
        })
        .to_string()
    }
}

/// dev-server を起動する。Ctrl-C で終了するまで戻らない。
pub(crate) fn run(file: &Path, fn_name: &str, port: u16) -> Result<()> {
    let file = file.to_path_buf();
    let shared = Arc::new(Shared::new());

    // 初回レンダリング (エラーでもサーバは起動し、画面にエラーを出す)。
    shared.publish(render_once(&file, fn_name));

    spawn_watcher(Arc::clone(&shared), file.clone(), fn_name.to_string())?;

    let server = tiny_http::Server::http(("127.0.0.1", port))
        .map_err(|e| anyhow::anyhow!("サーバを起動できません: {e}"))?;
    let actual_port = match server.server_addr() {
        tiny_http::ListenAddr::IP(addr) => addr.port(),
        // 本実装は IP しか bind しないため到達しない (防御)。
        tiny_http::ListenAddr::Unix(_) => port,
    };
    println!(
        "serving http://127.0.0.1:{actual_port}/  ({} --fn {fn_name})",
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

fn render_once(file: &Path, fn_name: &str) -> Result<String, String> {
    let source = std::fs::read_to_string(file)
        .map_err(|e| format!("ファイルを読み込めません: {}: {e}", file.display()))?;
    let graph = parse_function(&source, fn_name).map_err(|e| e.to_string())?;
    let placed = layout(&graph);
    Ok(render(&graph, &placed, RenderStyle::MidchildaConcentric))
}

/// ファイル監視スレッドを起動する。
///
/// エディタの rename 保存 (テンポラリ書き込み → 置換) でも検知できるよう、
/// ファイルそのものではなく**親ディレクトリ**を非再帰で監視し、
/// イベントのパスをファイル名で照合する。
fn spawn_watcher(shared: Arc<Shared>, file: PathBuf, fn_name: String) -> Result<()> {
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
            shared.publish(render_once(&file, &fn_name));
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
        // 監視エラーは「変化したかもしれない」側に倒して再レンダリングする。
        Err(_) => true,
    }
}

// ===== HTTP =====

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="ja"><head><meta charset="utf-8"><title>MagiaMagica dev-server</title>
<style>
  body { margin: 0; font-family: system-ui, sans-serif; }
  #status { color: #d92626; padding: 4px 12px; min-height: 1.2em; font-size: 14px; white-space: pre-wrap; }
  #magia { display: flex; justify-content: center; }
  #magia svg { max-width: 100vw; max-height: 94vh; }
</style></head>
<body>
<div id="status"></div>
<div id="magia"></div>
<script>
async function refresh() {
  const response = await fetch('/state');
  const state = await response.json();
  if (state.svg) { document.getElementById('magia').innerHTML = state.svg; }
  document.getElementById('status').textContent = state.error || '';
}
refresh();
new EventSource('/events').onmessage = refresh;
</script>
</body></html>
"#;

fn handle_request(request: tiny_http::Request, shared: &Shared) {
    let url = request.url().to_string();
    let result = match (request.method(), url.as_str()) {
        (tiny_http::Method::Get, "/") => request.respond(
            tiny_http::Response::from_string(INDEX_HTML)
                .with_header(header("Content-Type", "text/html; charset=utf-8")),
        ),
        (tiny_http::Method::Get, "/state") => request.respond(
            tiny_http::Response::from_string(shared.state_json())
                .with_header(header("Content-Type", "application/json; charset=utf-8")),
        ),
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

fn header(field: &str, value: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(field.as_bytes(), value.as_bytes()).expect("静的ヘッダは常に有効")
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
