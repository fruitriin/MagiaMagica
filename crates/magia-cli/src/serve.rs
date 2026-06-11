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
    /// 直近の正常な SVG — ミッドチルダ式 (エラー中も保持する)。
    svg: Mutex<String>,
    /// 直近の正常な SVG — ベルカ式 (Phase 3.5)。同じ IR の別射影なので対で更新する。
    svg_belka: Mutex<String>,
    /// 直近の正常な呪文書き起こし (spec §15。SVG と同じ IR の射影なので対で更新する)。
    transcript: Mutex<String>,
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
            svg_belka: Mutex::new(String::new()),
            transcript: Mutex::new(String::new()),
            error: Mutex::new(None),
            version: AtomicU64::new(0),
            clients: Mutex::new(Vec::new()),
        }
    }

    /// 状態を更新して世代を進め、SSE クライアントへ通知する。
    fn publish(&self, rendered: Result<Rendered, String>) {
        match rendered {
            Ok(rendered) => {
                *lock_or_recover(&self.svg) = rendered.svg;
                *lock_or_recover(&self.svg_belka) = rendered.svg_belka;
                *lock_or_recover(&self.transcript) = rendered.transcript;
                *lock_or_recover(&self.error) = None;
            }
            Err(message) => {
                // SVG・書き起こしは触らない: 直前の正常な内容を保持する (spec §7)。
                // スクリーンリーダー利用者にもエラーは #status (テキスト) で伝わり、
                // 書き起こしが「最後に解析できた状態」を指すのは視覚側と同じ扱い。
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
            "svg_belka": *lock_or_recover(&self.svg_belka),
            "transcript": *lock_or_recover(&self.transcript),
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

/// 1回分の解析結果 (両式の SVG と書き起こしは同じ IR の射影なので必ず対で作る)。
struct Rendered {
    svg: String,
    svg_belka: String,
    transcript: String,
}

// serve は常にフィルタなしで描き、レイヤー切替はブラウザ側 CSS で行う (spec §5.3)。
// 将来 serve が `--filter` を受けるときは render_with に切り替える (Phase 3 拡張点)。
fn render_once(file: &Path, fn_name: &str) -> Result<Rendered, String> {
    let source = std::fs::read_to_string(file)
        .map_err(|e| format!("ファイルを読み込めません: {}: {e}", file.display()))?;
    let graph = parse_function(&source, fn_name).map_err(|e| e.to_string())?;
    let placed = layout(&graph);
    Ok(Rendered {
        svg: render(&graph, &placed, RenderStyle::MidchildaConcentric),
        // ベルカ式は三角配置を内部で決める (placed は使われないが API は共通)。
        svg_belka: render(&graph, &placed, RenderStyle::Belka),
        transcript: magia_core::transcript::transcribe(&graph),
    })
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

// レイヤーパレット (Phase 2.2, spec v0.2 §5.5):
// - 切替は CSS (display / opacity) のみで行い、SVG は再生成しない (spec §5.3)
// - 状態は URL クエリ `?layers=a,b&op=a:0.5` に反映し、リロード・共有で再現できる
// - SSE による SVG 差し替え (innerHTML) 後はスタイルが消えるため毎回再適用する
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
let lastState = null;

function readQuery() {
  const params = new URLSearchParams(location.search);
  style = params.get('style') === 'belka' ? 'belka' : 'midchilda';
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
  if (!lastState) { return; }
  const svg = style === 'belka' ? lastState.svg_belka : lastState.svg;
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

async function refresh() {
  const response = await fetch('/state');
  lastState = await response.json();
  document.getElementById('status').textContent = lastState.error || '';
  document.getElementById('transcript').textContent = lastState.transcript || '';
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

fn handle_request(request: tiny_http::Request, shared: &Shared) {
    let url = request.url().to_string();
    // ルーティングはパス部分のみで行う (`/?layers=...` のようにレイヤーパレットの
    // 状態クエリが付いてもトップページを返す)。
    let path = url.split('?').next().unwrap_or("/");
    let result = match (request.method(), path) {
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
