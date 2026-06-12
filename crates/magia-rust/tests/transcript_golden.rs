//! 呪文書き起こしのゴールデンテスト (Phase 2.4 受け入れ基準)。

use magia_core::transcript::transcribe;
use magia_rust::parse_function;

fn transcript_of(source: &str, fn_name: &str) -> String {
    let graph = parse_function(source, fn_name).expect("fixture は必ずパースできる");
    transcribe(&graph)
}

fn assert_transcript_snapshot(name: &str, text: &str) {
    insta::with_settings!({ snapshot_path => "fixtures/snapshots", prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(name, text);
    });
}

#[test]
fn golden_transcript_async_await() {
    // fixture はワークスペースルートに統一 (CLI / README と同じものを使う)。
    let text = transcript_of(
        include_str!("../../../fixtures/async_await.rs"),
        "async_await",
    );
    // notes §9.1 の必須情報が揃っている。
    assert!(text.contains("関数 async_await:"));
    assert!(text.contains("async 関数 (await 2点)。"));
    assert!(text.contains("外部呼び出し"));
    assert_transcript_snapshot("transcript_async_await", &text);
}

#[test]
fn golden_transcript_dense_dispatch() {
    let text = transcript_of(
        include_str!("../../../fixtures/dense_dispatch.rs"),
        "dense_dispatch",
    );
    // Phase 4.8 M2: `map_or(0, |_| reload())` のクロージャが補助リングに数えられる。
    assert!(text.contains("補助リングを10個持つ (分岐7、ループ2、クロージャ1。入れ子を含む)。"));
    assert!(text.contains("ファイルシステム副作用"));
    assert_transcript_snapshot("transcript_dense_dispatch", &text);
}

#[test]
fn golden_transcript_medium() {
    let text = transcript_of(
        include_str!("../../../fixtures/medium_render_doc.rs"),
        "medium_render_doc",
    );
    assert_transcript_snapshot("transcript_medium_render_doc", &text);
}

#[test]
fn transcript_is_deterministic() {
    let source = include_str!("../../../fixtures/dense_dispatch.rs");
    let first = transcript_of(source, "dense_dispatch");
    for _ in 0..4 {
        assert_eq!(transcript_of(source, "dense_dispatch"), first);
    }
}
