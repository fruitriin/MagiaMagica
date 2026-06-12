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
    spawn_server_in(file.parent().expect("fixture は親を持つ"), file)
}

/// cwd を指定して起動する (Phase 4.4.5 — /files・/file は cwd がワークスペース境界)。
fn spawn_server_in(cwd: &std::path::Path, file: &std::path::Path) -> DevServer {
    let binary = assert_cmd::cargo::cargo_bin("magia");
    let mut child = Command::new(binary)
        .current_dir(cwd)
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

    // Phase 4.0.5 M5: UI は rust-embed 同梱の Vue SPA (web/dist)。
    // UI の振る舞い (パレット・DSL・書き起こし) は Playwright (M6) が担い、
    // ここでは「SPA シェルと静的アセットが配信される」契約だけを見る。
    let index = http_get(server.port, "/");
    assert!(index.contains("200"));
    assert!(index.contains(r#"<div id="app">"#));
    let asset = index
        .split(r#"src="/"#)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("index.html がバンドル JS を参照している");
    let js = http_get(server.port, &format!("/{asset}"));
    assert!(js.contains("200") && js.contains("text/javascript"));
    // 存在しない静的ファイルは 404。
    assert!(http_get(server.port, "/no_such_file.js").contains("404"));

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
    assert!(with_fn.contains("200") && with_fn.contains(r#"<div id="app">"#));
}

#[test]
fn spell_endpoint_renders_function_on_demand() {
    let file = temp_fixture("spell.rs", INITIAL);
    let server = spawn_server(&file);

    let spell = body_json_at(server.port, "/spell/watched");
    // Phase 4.0.9: ミッドチルダ式は配置済み IR (JSON)。メインリングと配置済み
    // 操作ドット・シグネチャ円弧を持つ (Vue がこれを描画する契約)。
    let ir = &spell["ir"];
    let rings = ir["rings"].as_array().unwrap();
    assert!(rings.iter().any(|r| r["role"] == "main"));
    let main = rings.iter().find(|r| r["role"] == "main").unwrap();
    assert!(main["radius"].as_f64().unwrap() > 0.0);
    assert!(!main["operations"].as_array().unwrap().is_empty());
    assert!(main["operations"][0]["x"].is_number());
    assert!(
        ir["signature"]["arc_path"]
            .as_str()
            .unwrap()
            .starts_with('M')
    );
    assert_eq!(ir["view_box"].as_array().unwrap().len(), 4);
    // ベルカ式も配置済み IR (Phase 4.3 — Vue の BelkaCircle が描く)。
    assert_eq!(spell["belka_ir"]["poles"].as_array().unwrap().len(), 3);
    assert!(spell["source_html"].as_str().unwrap().contains("<pre"));
    assert!(spell["source_html"].as_str().unwrap().contains("watched"));
    assert!(spell["transcript"].as_str().unwrap().contains("関数 "));
    assert!(spell["signature"].as_str().unwrap().contains("fn watched"));
    // Phase 4.1 追加要望3: 操作ドットの原文断片 (`<ring_id>-<出現順>` →
    // ハイライト済み HTML、改行・列込みの原文切り出し)。
    let main_id = main["id"].as_u64().unwrap();
    let op_excerpt = spell["op_excerpts"][format!("{main_id}-0")]
        .as_str()
        .expect("メインリング先頭の操作に断片が付く");
    // 切り出し内容の正確さは span_excerpt の unit テストが担保する。
    // ここでは「ハイライト済み HTML が付く」契約だけ見る (syntect はトークン
    // ごとに span 分割するため連続文字列では照合できない)。
    assert!(op_excerpt.contains("<pre"));
    // 操作 IR にも原文位置が載る (1-based・列あり)。
    assert!(main["operations"][0]["source_span"]["start_column"].as_u64() > Some(0));

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

/// Phase 4.1: `?with=neighbors` でピン中心ビューの周辺配置が併載される。
/// 配置は決定論 (名前ソート → 等角度) で、focus 自身は周辺に含まれない。
#[test]
fn spell_with_neighbors_returns_focus_layout() {
    let file = temp_fixture("neighbors.rs", INITIAL);
    let server = spawn_server(&file);

    // 既定 (with なし) は従来契約 — focus_layout を持たない。
    let plain = body_json_at(server.port, "/spell/watched");
    assert!(plain.get("focus_layout").is_none());

    let spell = body_json_at(server.port, "/spell/watched?with=neighbors");
    let layout = &spell["focus_layout"];
    let neighbors = layout["neighbors"].as_array().unwrap();
    // INITIAL は watched + Caster::cast の2関数 — 周辺は cast のみ
    // (呼び出し関係なし・impl 違いの同ファイル = 外リング 3、Phase 4.2)。
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0]["qualified"], "Caster::cast");
    assert_eq!(neighbors[0]["distance"], 3);
    assert!(neighbors[0]["scale"].as_f64().unwrap() < 1.0);
    assert!(neighbors[0]["opacity"].as_f64().unwrap() < 1.0);
    assert!(neighbors[0]["x"].is_number() && neighbors[0]["y"].is_number());
    // 全体 viewBox は focus 単体 (ir.view_box) より広い。
    let focus_w = spell["ir"]["view_box"][2].as_f64().unwrap();
    assert!(layout["view_box"][2].as_f64().unwrap() > focus_w);

    // 逆方向 (Caster::cast を focus) でも同じ分類 — 近接度は無向。
    let cast = body_json_at(server.port, "/spell/Caster%3A%3Acast?with=neighbors");
    assert_eq!(cast["focus_layout"]["neighbors"][0]["distance"], 3);
}

/// Phase 4.2: 近接度の本実装 — 同 impl = 内リング (1)、呼び出し関係 = 中リング (2)、
/// その他の同ファイル = 外リング (3)。呼び出しは無向 (caller 側も近い)。
#[test]
fn proximity_rings_reflect_impl_and_call_relations() {
    let source = "fn entry(v: i32) -> i32 { helper(v) }\n\
fn helper(v: i32) -> i32 { v }\n\
fn loner() {}\n\
struct Wand;\n\
impl Wand {\n    fn cast(&self) -> i32 { self.charge() }\n    fn charge(&self) -> i32 { 1 }\n}\n";
    let file = temp_fixture("proximity.rs", source);
    let server = spawn_server(&file);

    let ring_of = |spell: &serde_json::Value, name: &str| -> u64 {
        spell["focus_layout"]["neighbors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["qualified"] == name)
            .unwrap_or_else(|| panic!("{name} が周辺にいる"))["distance"]
            .as_u64()
            .unwrap()
    };

    // entry → helper の呼び出しで helper は中リング、無関係な loner は外リング。
    let entry = body_json_at(server.port, "/spell/entry?with=neighbors");
    assert_eq!(ring_of(&entry, "helper"), 2);
    assert_eq!(ring_of(&entry, "loner"), 3);

    // 無向: helper を focus にしても entry は中リング。
    let helper = body_json_at(server.port, "/spell/helper?with=neighbors");
    assert_eq!(ring_of(&helper, "entry"), 2);

    // 同 impl (かつ `.charge()` 呼び出し) は min で内リング。
    let cast = body_json_at(server.port, "/spell/Wand%3A%3Acast?with=neighbors");
    assert_eq!(ring_of(&cast, "Wand::charge"), 1);
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
    assert_ne!(before_spell["ir"], after_spell["ir"]);
    assert_ne!(
        before_spell["source_html"].as_str().unwrap(),
        after_spell["source_html"].as_str().unwrap()
    );
    assert!(after["error"].is_null());
}

/// SSE (/events) はヘッダ + 接続直後イベントが**即座に**届き、ファイル変更で
/// 追加イベントが流れる。Phase 4.0.5 M2 で発見した「tiny_http のチャンク転送経路は
/// chunked_transfer::Encoder (8KB) + BufWriter (1KB) の二重バッファが flush されず、
/// イベントがクライアントへ永遠に届かない」バグ (Phase 2.1 から潜在) の回帰テスト。
/// read_line のタイムアウト (5秒) = 滞留の再発。
#[test]
fn sse_events_stream_immediately() {
    let file = temp_fixture("sse.rs", INITIAL);
    let server = spawn_server(&file);

    let mut stream = TcpStream::connect(("127.0.0.1", server.port)).expect("接続できる");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(stream, "GET /events HTTP/1.0\r\nHost: localhost\r\n\r\n").unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    // ステータス行 → text/event-stream ヘッダ → 接続直後イベントの順で届く。
    reader.read_line(&mut line).expect("ステータス行が届く");
    assert!(line.contains("200"), "SSE のステータス行: {line}");
    let mut saw_content_type = false;
    loop {
        line.clear();
        reader
            .read_line(&mut line)
            .expect("接続直後イベントまで滞留なく読める");
        let trimmed = line.trim();
        saw_content_type |= trimmed.eq_ignore_ascii_case("content-type: text/event-stream");
        if trimmed == "data: 0" {
            break;
        }
    }
    assert!(saw_content_type, "Content-Type が SSE になっている");

    // ファイル変更 → 新しい version のイベントが流れる。
    std::fs::write(&file, CHANGED).unwrap();
    loop {
        line.clear();
        reader.read_line(&mut line).expect("更新イベントが届く");
        if let Some(version) = line.trim().strip_prefix("data: ") {
            assert!(
                version.parse::<u64>().is_ok(),
                "イベントは version 数値: {version}"
            );
            break;
        }
    }
}

#[test]
fn syntax_error_keeps_last_good_snapshot() {
    let file = temp_fixture("broken.rs", INITIAL);
    let server = spawn_server(&file);
    let good_spell = body_json_at(server.port, "/spell/watched");
    let good_ir = good_spell["ir"].clone();

    std::fs::write(&file, BROKEN).unwrap();
    let state = wait_for(server.port, |s| !s["error"].is_null());
    // エラーには行番号が付く (ソースペインの案内に使う)。
    assert!(state["error"]["line"].as_u64().is_some());
    // 関数一覧も /spell も直前の正常スナップショットから配信し続ける。
    assert!(!state["functions"].as_array().unwrap().is_empty());
    let kept = body_json_at(server.port, "/spell/watched");
    assert_eq!(
        kept["ir"], good_ir,
        "直前の正常な魔法陣を保持する (会話を切らない)"
    );

    // 直してまた描けることも確認する。
    std::fs::write(&file, CHANGED).unwrap();
    let recovered = wait_for(server.port, |s| s["error"].is_null());
    assert!(recovered["error"].is_null());
    let fresh = body_json_at(server.port, "/spell/watched");
    assert_ne!(fresh["ir"], good_ir);
}

/// Phase 4.3.7: `?diff=<REV>` で Spell Diff overlay が併載される (web 上の live diff)。
#[test]
fn spell_diff_query_returns_overlay_and_notes() {
    // git リポジトリ化した fixture: 初期内容を commit してから変更を書く。
    let file = temp_fixture("diffweb.rs", INITIAL);
    let dir = file.parent().unwrap();
    let git = |args: &[&str]| {
        let ok = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("git を起動できる")
            .success();
        assert!(ok, "git {args:?} が成功する");
    };
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "initial"]);
    std::fs::write(&file, CHANGED).unwrap();

    let server = spawn_server(&file);
    wait_for(server.port, |s| {
        s["functions"]
            .as_array()
            .is_some_and(|fns| fns.iter().any(|f| f["qualified"] == "fresh_fn"))
    });

    // 正常系: HEAD (= INITIAL) との diff。watched は helper 呼び出しが増えた。
    let spell = body_json_at(server.port, "/spell/watched?with=neighbors&diff=HEAD");
    let marks = spell["diff_overlay"]
        .as_array()
        .expect("overlay が併載される");
    assert!(!marks.is_empty());
    assert!(
        marks
            .iter()
            .all(|m| m["x"].is_number() && m["radius"].is_number())
    );
    assert!(
        spell["diff_report"]
            .as_str()
            .expect("メトリクス要約が併載される")
            .contains("watched")
    );
    assert!(spell.get("diff_note").is_none());
    // focus_layout は diff 版 viewBox (ゴースト拡張込み) を入力にしている。
    assert!(spell["focus_layout"]["view_box"].is_array());

    // 新規関数 (HEAD に無い) は案内文 — UI を壊さない。
    let fresh = body_json_at(server.port, "/spell/fresh_fn?diff=HEAD");
    assert!(fresh.get("diff_overlay").is_none());
    assert!(
        fresh["diff_note"]
            .as_str()
            .expect("案内が出る")
            .contains("新規関数")
    );

    // 不正な rev も案内文で受ける。
    let bad = body_json_at(server.port, "/spell/watched?diff=no_such_rev");
    assert!(bad.get("diff_overlay").is_none());
    assert!(bad["diff_note"].as_str().unwrap().contains("no_such_rev"));

    // ?diff なしは従来契約と完全互換 (diff 系フィールドが無い)。
    let plain = body_json_at(server.port, "/spell/watched");
    assert!(plain.get("diff_overlay").is_none());
    assert!(plain.get("diff_report").is_none());
    assert!(plain.get("diff_note").is_none());
}

/// Phase 4.4.5: 監視ファイルの一覧 (`GET /files`) と切替 (`POST /file`)。
#[test]
fn file_listing_and_switching() {
    let file = temp_fixture("switch.rs", INITIAL);
    let dir = file.parent().unwrap().to_path_buf();
    std::fs::write(dir.join("second.rs"), "fn beta(x: u8) -> u8 { x + 1 }\n").unwrap();
    std::fs::create_dir_all(dir.join("target")).unwrap();
    std::fs::write(dir.join("target/generated.rs"), "fn hidden() {}\n").unwrap();
    let server = spawn_server_in(&dir, &file);

    // 一覧: cwd 配下の .rs (target/ は除外、ソート済み)。
    let files = body_json_at(server.port, "/files");
    let listed: Vec<&str> = files["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap())
        .collect();
    assert_eq!(listed, ["second.rs", "switch.rs"]);

    // 切替: /state と /spell が新ファイルへ追従する (watcher スレッド経由なので待つ)。
    let response = http_post_json(server.port, "/file", r#"{"path":"second.rs"}"#);
    assert!(response.contains("200"), "{response}");
    assert!(response.contains(r#""file":"second.rs""#));
    let state = wait_for(server.port, |s| s["file"] == "second.rs");
    let names: Vec<&str> = state["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["qualified"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["beta"]);
    let spell = body_json_at(server.port, "/spell/beta");
    assert!(
        spell["ir"]["rings"]
            .as_array()
            .is_some_and(|r| !r.is_empty())
    );

    // 切替後はファイル保存の live 更新も新ファイルで動く (watch 張り替えの検証)。
    std::fs::write(
        dir.join("second.rs"),
        "fn beta(x: u8) -> u8 { x + 1 }\nfn gamma() {}\n",
    )
    .unwrap();
    wait_for(server.port, |s| {
        s["functions"]
            .as_array()
            .is_some_and(|fns| fns.iter().any(|f| f["qualified"] == "gamma"))
    });

    // 拒否系: ワークスペース外 / 非 .rs / 存在しない / path なし → 400 (状態は不変)。
    for (body, needle) in [
        (r#"{"path":"../outside.rs"}"#, "400"),
        (r#"{"path":"second.txt"}"#, "400"),
        (r#"{"path":"no_such.rs"}"#, "400"),
        (r#"{"nope":1}"#, "400"),
    ] {
        let rejected = http_post_json(server.port, "/file", body);
        assert!(rejected.contains(needle), "{body} → {rejected}");
    }
    assert_eq!(state_json(server.port)["file"], "second.rs");
}

/// 素朴な HTTP/1.0 POST (JSON ボディ)。
fn http_post_json(port: u16, path: &str, body: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("接続できる");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(
        stream,
        "POST {path} HTTP/1.0\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("応答を読める");
    response
}

/// Phase 4.4: 周辺チップにフォーカス基準の呼び出し関係 (relation) が載る。
#[test]
fn neighbor_chips_carry_call_relation() {
    let source = "fn entry(v: i32) -> i32 { helper(v) }\n\
fn helper(v: i32) -> i32 { back(v) }\n\
fn back(v: i32) -> i32 { if v > 0 { helper(v - 1) } else { v } }\n\
fn loner() {}\n";
    let file = temp_fixture("relation.rs", source);
    let server = spawn_server(&file);

    let relation_of = |spell: &serde_json::Value, name: &str| -> serde_json::Value {
        spell["focus_layout"]["neighbors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["qualified"] == name)
            .unwrap_or_else(|| panic!("{name} が周辺にいる"))["relation"]
            .clone()
    };

    // entry から見て: helper は呼ぶ先 / loner は無関係 (フィールド省略 = null)。
    let entry = body_json_at(server.port, "/spell/entry?with=neighbors");
    assert_eq!(relation_of(&entry, "helper"), "calls");
    assert_eq!(relation_of(&entry, "loner"), serde_json::Value::Null);

    // helper から見て: entry は呼んでくる側 / back は相互 (helper→back→helper)。
    let helper = body_json_at(server.port, "/spell/helper?with=neighbors");
    assert_eq!(relation_of(&helper, "entry"), "called_by");
    assert_eq!(relation_of(&helper, "back"), "mutual");
}

/// Phase 4.5 M1: `GET /workspace` — 全 .rs の関数一覧 (俯瞰の入力)。
#[test]
fn workspace_endpoint_lists_functions_per_file() {
    let file = temp_fixture("overview.rs", INITIAL);
    let dir = file.parent().unwrap().to_path_buf();
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::write(dir.join("sub/extra.rs"), "fn deep() {}\n").unwrap();
    std::fs::write(dir.join("broken.rs"), "fn broken( {\n").unwrap();
    let server = spawn_server_in(&dir, &file);

    let workspace = body_json_at(server.port, "/workspace");
    let files = workspace["files"].as_array().unwrap();
    let by_path = |p: &str| -> &serde_json::Value {
        files
            .iter()
            .find(|f| f["path"] == p)
            .unwrap_or_else(|| panic!("{p} が一覧にいない"))
    };

    // 正常ファイル: 関数一覧 (qualified + signature) が載る。
    let overview = by_path("overview.rs");
    let names: Vec<&str> = overview["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["qualified"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["watched", "Caster::cast"]);
    assert!(overview.get("error").is_none());

    // サブディレクトリは dir で区別できる。
    assert_eq!(by_path("sub/extra.rs")["dir"], "sub");
    assert_eq!(by_path("overview.rs")["dir"], "");

    // 壊れたファイルは俯瞰を壊さず error フラグでスキップ。
    let broken = by_path("broken.rs");
    assert_eq!(broken["error"], true);
    assert_eq!(broken["functions"].as_array().unwrap().len(), 0);
}
