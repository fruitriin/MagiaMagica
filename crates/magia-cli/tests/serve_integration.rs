//! dev-server の統合テスト (Phase 2.1 受け入れ基準 + Phase 4.0 ソース連動ビュー)。
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

/// Phase 4.0: `--fn` は廃止 — ファイルだけで起動する。
fn spawn_server(file: &std::path::Path) -> DevServer {
    let binary = assert_cmd::cargo::cargo_bin("magia");
    let mut child = Command::new(binary)
        .arg("serve")
        .arg(file)
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

fn body_json_at(port: u16, path: &str) -> serde_json::Value {
    let response = http_get(port, path);
    let body = response
        .split("\r\n\r\n")
        .nth(1)
        .expect("ヘッダとボディの区切りがある");
    serde_json::from_str(body).unwrap_or_else(|e| panic!("{path} は有効な JSON: {e}\n{body}"))
}

fn state_json(port: u16) -> serde_json::Value {
    body_json_at(port, "/state")
}

/// 条件が満たされるまで /state をポーリングする (上限 5 秒)。
fn wait_for(port: u16, predicate: impl Fn(&serde_json::Value) -> bool) -> serde_json::Value {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let state = state_json(port);
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

const INITIAL: &str = "fn watched(a: i32) -> i32 { a + 1 }\n\
struct Caster;\n\
impl Caster {\n    fn cast(&self) -> i32 { 42 }\n}\n";
const CHANGED: &str = "fn watched(a: i32) -> i32 { let b = a * 2; helper(b) }\n\
fn fresh_fn() {}\n\
struct Caster;\n\
impl Caster {\n    fn cast(&self) -> i32 { 42 }\n}\n";
const BROKEN: &str = "fn watched(a: i32) -> i32 { let b = \n";

#[test]
fn serves_function_index_api() {
    let file = temp_fixture("initial.rs", INITIAL);
    let server = spawn_server(&file);

    // Phase 4.0 はサーバ側 API まで (ペアビュー UI は Phase 4.0.5 / Vue)。
    // 既存 UI (パレット・式トグル・書き起こし・DSL 往復) は維持される。
    let index = http_get(server.port, "/");
    assert!(index.contains("200"));
    assert!(index.contains(r#"<div id="magia">"#));
    assert!(index.contains("EventSource"));
    for layer in ["control_flow", "effects", "type_info"] {
        assert!(index.contains(&format!(r#"data-layer="{layer}""#)));
    }
    for style in ["midchilda", "belka"] {
        assert!(index.contains(&format!(r#"data-style="{style}""#)));
    }
    assert!(index.contains(r#"id="dsl""#) && index.contains("visually-hidden"));
    // 薄い継ぎ目: 旧 UI が新 API を呼ぶ。
    assert!(index.contains("/spell/"));

    // /state はメタ + 関数一覧 (impl メソッドは qualified 名)。
    let state = state_json(server.port);
    assert!(state["error"].is_null());
    let names: Vec<&str> = state["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["qualified"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["watched", "Caster::cast"]);
    let cast = &state["functions"][1];
    assert_eq!(cast["impl_context"], "Caster");
    assert!(cast["start_line"].as_u64().unwrap() > 0);

    // ?fn= 付き URL でもトップページが返る (リロード・共有経路)。
    let with_fn = http_get(server.port, "/?fn=Caster%3A%3Acast&style=belka");
    assert!(with_fn.contains("200") && with_fn.contains(r#"<div id="magia">"#));
}

#[test]
fn spell_endpoint_renders_function_on_demand() {
    let file = temp_fixture("spell.rs", INITIAL);
    let server = spawn_server(&file);

    let spell = body_json_at(server.port, "/spell/watched");
    assert!(
        spell["svg"]
            .as_str()
            .unwrap()
            .contains("layer-control-flow")
    );
    assert!(spell["svg_belka"].as_str().unwrap().contains("belka-pole"));
    assert!(spell["source_html"].as_str().unwrap().contains("<pre"));
    assert!(spell["source_html"].as_str().unwrap().contains("watched"));
    assert!(spell["transcript"].as_str().unwrap().contains("関数 "));
    assert!(spell["signature"].as_str().unwrap().contains("fn watched"));

    // impl メソッドは qualified 名 (URL エンコード) で引ける。
    let method = body_json_at(server.port, "/spell/Caster%3A%3Acast");
    assert!(method["signature"].as_str().unwrap().contains("fn cast"));
    assert!(
        method["source_html"].as_str().unwrap().contains("42"),
        "メソッド本体のスニペットが入る"
    );

    // 未知の関数は 404。
    let missing = http_get(server.port, "/spell/no_such_fn");
    assert!(missing.contains("404"));
    assert!(missing.contains("索引にありません"));
}

#[test]
fn file_change_updates_function_index_and_spells() {
    let file = temp_fixture("change.rs", INITIAL);
    let server = spawn_server(&file);
    let before_spell = body_json_at(server.port, "/spell/watched");

    std::fs::write(&file, CHANGED).unwrap();
    // 述語は version でなく意味 (新関数の出現) で待つ: truncate→write の途中状態を
    // 読んだ中間 reload の version 加算を最終状態と誤認しない。
    let after = wait_for(server.port, |s| {
        s["functions"]
            .as_array()
            .is_some_and(|fns| fns.iter().any(|f| f["qualified"] == "fresh_fn"))
    });
    // 既存関数の魔法陣・ソースも新内容でオンデマンド再レンダリングされる。
    let after_spell = body_json_at(server.port, "/spell/watched");
    assert_ne!(
        before_spell["svg"].as_str().unwrap(),
        after_spell["svg"].as_str().unwrap()
    );
    assert_ne!(
        before_spell["source_html"].as_str().unwrap(),
        after_spell["source_html"].as_str().unwrap()
    );
    assert!(after["error"].is_null());
}

#[test]
fn syntax_error_keeps_last_good_snapshot() {
    let file = temp_fixture("broken.rs", INITIAL);
    let server = spawn_server(&file);
    let good_spell = body_json_at(server.port, "/spell/watched");
    let good_svg = good_spell["svg"].as_str().unwrap().to_string();

    std::fs::write(&file, BROKEN).unwrap();
    let state = wait_for(server.port, |s| !s["error"].is_null());
    // エラーには行番号が付く (ソースペインの案内に使う)。
    assert!(state["error"]["line"].as_u64().is_some());
    // 関数一覧も /spell も直前の正常スナップショットから配信し続ける。
    assert!(!state["functions"].as_array().unwrap().is_empty());
    let kept = body_json_at(server.port, "/spell/watched");
    assert_eq!(
        kept["svg"].as_str().unwrap(),
        good_svg,
        "直前の正常な魔法陣を保持する (会話を切らない)"
    );

    // 直してまた描けることも確認する。
    std::fs::write(&file, CHANGED).unwrap();
    let recovered = wait_for(server.port, |s| s["error"].is_null());
    assert!(recovered["error"].is_null());
    let fresh = body_json_at(server.port, "/spell/watched");
    assert_ne!(fresh["svg"].as_str().unwrap(), good_svg);
}
