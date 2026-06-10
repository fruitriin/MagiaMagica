//! ゴールデンテスト (Phase 1.6 受け入れ基準)。
//!
//! 合成 fixture 5本を `parse_function → layout → render` に通し、SVG を insta
//! スナップショットとして確定する。スナップショットは `tests/fixtures/snapshots/`
//! に格納し、レビュー時は SVG diff を目視で確認する (計画の設計判断)。

use magia_core::layout::layout;
use magia_core::render::{RenderStyle, render};
use magia_rust::parse_function;

fn render_fixture(source: &str, fn_name: &str) -> String {
    let graph = parse_function(source, fn_name).expect("fixture は必ずパースできる");
    let placed = layout(&graph);
    render(&graph, &placed, RenderStyle::MidchildaConcentric)
}

fn assert_svg_snapshot(name: &str, svg: &str) {
    insta::with_settings!({ snapshot_path => "fixtures/snapshots", prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(name, svg);
    });
}

#[test]
fn golden_simple_compute() {
    let svg = render_fixture(include_str!("fixtures/simple_compute.rs"), "simple_compute");
    assert_svg_snapshot("simple_compute", &svg);
}

#[test]
fn golden_if_branch() {
    let svg = render_fixture(include_str!("fixtures/if_branch.rs"), "if_branch");
    // 分岐 = AuxRing 2個 + 分岐記号。
    assert!(svg.contains(r#"class="aux-ring""#));
    assert!(svg.contains(r#"class="sym-branch""#));
    assert_svg_snapshot("if_branch", &svg);
}

#[test]
fn golden_match_arms() {
    let svg = render_fixture(include_str!("fixtures/match_arms.rs"), "match_arms");
    assert_svg_snapshot("match_arms", &svg);
}

#[test]
fn golden_async_io() {
    let svg = render_fixture(include_str!("fixtures/async_io.rs"), "async_io");
    // async 二重線・io 色 (println!)・Result 分岐線・早期リターン (`?`)。
    assert!(svg.contains(r#"class="main-ring-async""#));
    assert!(svg.contains("#1f4dff"), "io 効果の青が出る");
    assert!(svg.contains(r#"class="return-path-err""#));
    assert!(svg.contains(r#"class="sym-early-return""#));
    assert_svg_snapshot("async_io", &svg);
}

#[test]
fn golden_for_loop() {
    let svg = render_fixture(include_str!("fixtures/for_loop.rs"), "for_loop");
    // ループ本体 AuxRing + ループ記号 (内側の小さな矢印)。
    assert!(svg.contains(r#"class="aux-ring""#));
    assert!(svg.contains(r#"class="sym-loop""#));
    assert_svg_snapshot("for_loop", &svg);
}

#[test]
fn golden_unsafe_block() {
    let svg = render_fixture(include_str!("fixtures/unsafe_block.rs"), "unsafe_block");
    // unsafe fn のコンテキストが赤で出る。
    assert!(svg.contains("#d92626"), "unsafe 効果の赤が出る");
    assert_svg_snapshot("unsafe_block", &svg);
}

#[test]
fn golden_dense_dispatch() {
    // Phase 1.8 の衝突回避を通った過密ケースのスナップショット。
    let svg = render_fixture(
        include_str!("../../../fixtures/dense_dispatch.rs"),
        "dense_dispatch",
    );
    assert_svg_snapshot("dense_dispatch", &svg);
}

#[test]
fn golden_render_is_deterministic_end_to_end() {
    let source = include_str!("fixtures/async_io.rs");
    let first = render_fixture(source, "async_io");
    let second = render_fixture(source, "async_io");
    assert_eq!(first, second, "parse → layout → render の全段が決定論的");
}
