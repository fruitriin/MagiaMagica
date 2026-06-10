//! dev-server の統合テスト (Phase 2.1 受け入れ基準)。
//!
//! 実バイナリを `--port 0` (空きポート自動割当) で起動し、生 TCP の HTTP/1.0 で叩く
//! (HTTP クライアント依存を増やさない)。サーバは子プロセスとして必ず kill する。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// 起動した dev-server。drop 時に必ず子プロセスを kill する。
struct DevServer {
    child: Child,
    port: u16,
}

impl Drop for DevServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn spawn_server(file: &std::path::Path, fn_name: &str) -> DevServer {
    let binary = assert_cmd::cargo::cargo_bin("magia");
    let mut child = Command::new(binary)
        .arg("serve")
        .arg(file)
        .arg("--fn")
        .arg(fn_name)
        .arg("--port")
        .arg("0")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("magia serve を起動できる");

    // 起動行 `serving http://127.0.0.1:PORT/ ...` からポートを読む。
    let stdout = child.stdout.take().expect("stdout をパイプしている");
    let mut lines = BufReader::new(stdout).lines();
    let first = lines
        .next()
        .expect("起動行が出力される")
        .expect("起動行を読める");
    let port: u16 = first
        .split("127.0.0.1:")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| panic!("起動行からポートを抽出できない: {first}"));
    DevServer { child, port }
}

/// 素朴な HTTP/1.0 GET。レスポンス全体 (ヘッダ + ボディ) を文字列で返す。
fn http_get(port: u16, path: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("接続できる");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(stream, "GET {path} HTTP/1.0\r\nHost: localhost\r\n\r\n").unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("応答を読める");
    response
}

fn body_json(port: u16) -> serde_json::Value {
    let response = http_get(port, "/state");
    let body = response
        .split("\r\n\r\n")
        .nth(1)
        .expect("ヘッダとボディの区切りがある");
    serde_json::from_str(body).expect("state は有効な JSON")
}

/// 条件が満たされるまで /state をポーリングする (上限 5 秒)。
fn wait_for(port: u16, predicate: impl Fn(&serde_json::Value) -> bool) -> serde_json::Value {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let state = body_json(port);
        if predicate(&state) {
            return state;
        }
        assert!(
            Instant::now() < deadline,
            "5秒以内に条件を満たさない: {state}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// テストごとに**専用ディレクトリ**を作って fixture を置く。
/// テストは並列実行されるため、ディレクトリを分けないと他テストのファイル変更
/// イベントが notify 監視に混入してフレークする。毎回作り直してクリーンに始める。
fn temp_fixture(name: &str, content: &str) -> std::path::PathBuf {
    let stem = name.trim_end_matches(".rs");
    let dir = std::env::temp_dir().join("magia-serve-test").join(stem);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

const INITIAL: &str = "fn watched(a: i32) -> i32 { a + 1 }\n";
const CHANGED: &str = "fn watched(a: i32) -> i32 { let b = a * 2; helper(b) }\n";
const BROKEN: &str = "fn watched(a: i32) -> i32 { let b = \n";

#[test]
fn serves_html_and_initial_state() {
    let file = temp_fixture("initial.rs", INITIAL);
    let server = spawn_server(&file, "watched");

    let index = http_get(server.port, "/");
    assert!(index.contains("200"));
    assert!(index.contains(r#"<div id="magia">"#));
    assert!(index.contains("EventSource"));

    // レイヤーパレット (Phase 2.2): 3レイヤーの切替 UI が配信される。
    for layer in ["control_flow", "effects", "type_info"] {
        assert!(
            index.contains(&format!(r#"data-layer="{layer}""#)),
            "{layer} のチェックボックスがある"
        );
        assert!(
            index.contains(&format!(r#"data-opacity="{layer}""#)),
            "{layer} の透明度スライダーがある"
        );
    }
    assert!(index.contains(r#"id="all-on""#) && index.contains(r#"id="all-off""#));

    let state = body_json(server.port);
    let svg = state["svg"].as_str().unwrap();
    assert!(svg.contains("<svg "), "初回レンダリング済み");
    assert!(state["error"].is_null());
    // パレット JS の cssClass() 変換 (snake_case → layer-kebab-case) が、
    // レンダラの出力する <g> クラス名と一致していることの契約テスト (spec §5.3)。
    for class in ["layer-control-flow", "layer-effects", "layer-type-info"] {
        assert!(
            svg.contains(&format!(r#"<g class="{class}">"#)),
            "SVG に {class} の <g> がある"
        );
    }

    let missing = http_get(server.port, "/no-such");
    assert!(missing.contains("404"));

    // レイヤー状態のクエリ付き URL (リロード・共有) でもトップページが返る。
    // 回帰経緯: ルート照合が完全一致だった頃 `/?layers=...` が 404 になった。
    let with_query = http_get(server.port, "/?layers=effects&op=effects:0.5");
    assert!(with_query.contains("200") && with_query.contains(r#"<div id="magia">"#));
    assert!(with_query.contains("text/html"), "HTML として返る");
}

#[test]
fn file_change_triggers_rerender() {
    let file = temp_fixture("change.rs", INITIAL);
    let server = spawn_server(&file, "watched");
    let before = body_json(server.port);
    let before_version = before["version"].as_u64().unwrap();

    std::fs::write(&file, CHANGED).unwrap();
    let after = wait_for(server.port, |s| {
        s["version"].as_u64().unwrap() > before_version
    });
    assert_ne!(
        before["svg"].as_str().unwrap(),
        after["svg"].as_str().unwrap(),
        "変更後の SVG は変わる (helper 呼び出しの召喚記号が増える)"
    );
    assert!(after["error"].is_null());
}

#[test]
fn syntax_error_keeps_last_good_svg() {
    let file = temp_fixture("broken.rs", INITIAL);
    let server = spawn_server(&file, "watched");
    let good = body_json(server.port);
    let good_svg = good["svg"].as_str().unwrap().to_string();
    assert!(good_svg.contains("<svg "));

    std::fs::write(&file, BROKEN).unwrap();
    let state = wait_for(server.port, |s| !s["error"].is_null());
    // メッセージ本文には依存しない (実装側の文言変更でフレークさせない)。
    assert!(!state["error"].as_str().unwrap().is_empty());
    assert_eq!(
        state["svg"].as_str().unwrap(),
        good_svg,
        "直前の正常な SVG を保持する (会話を切らない)"
    );

    // 直してまた描けることも確認する。
    std::fs::write(&file, CHANGED).unwrap();
    let recovered = wait_for(server.port, |s| s["error"].is_null());
    assert_ne!(recovered["svg"].as_str().unwrap(), good_svg);
}
