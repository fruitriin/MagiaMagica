//! ベルカ式レンダラのゴールデンテスト (Phase 3.5 受け入れ基準, spec v0.3 §14)。

use magia_core::layout::layout;
use magia_core::render::{RenderStyle, render};
use magia_rust::parse_function;

const LOOP_ACCUMULATE: &str = include_str!("../../../fixtures/loop_accumulate.rs");

fn render_belka(source: &str, fn_name: &str) -> String {
    let graph = parse_function(source, fn_name).expect("fixture は必ずパースできる");
    // ベルカ式は三角配置を内部で決めるため placed は使われない (API が共通なだけ)。
    let placed = layout(&graph);
    render(&graph, &placed, RenderStyle::Belka)
}

#[test]
fn belka_draws_three_poles_with_fields() {
    let svg = render_belka(LOOP_ACCUMULATE, "loop_accumulate");
    assert!(svg.starts_with("<svg "));
    assert!(svg.trim_end().ends_with("</svg>"));
    // 三極 (生成/変換/消費) と力場・フロー線の構成要素が揃う。
    assert_eq!(svg.matches(r#"class="belka-pole""#).count(), 3);
    assert_eq!(svg.matches("<radialGradient").count(), 3);
    for label in ["生成", "変換", "消費"] {
        assert!(svg.contains(label), "{label} のラベルが出る");
    }
    assert!(svg.contains("belka-field"));
    assert!(svg.contains("belka-flow"), "極間のフロー線が出る");
}

#[test]
fn belka_flow_follows_execution_order() {
    // loop_accumulate: items (生成) → ループで変換 → 末尾の total が消費へ。
    // 実行順走査により「変換 → 消費」の還流が出る (深さ優先だと取り逃がす形)。
    let svg = render_belka(LOOP_ACCUMULATE, "loop_accumulate");
    let flow_lines = svg
        .lines()
        .filter(|line| line.contains(r#"class="belka-flow""#))
        .count();
    assert!(
        flow_lines >= 2,
        "生成→変換 と 変換→消費 の2本以上: {flow_lines}"
    );
}

#[test]
fn belka_is_deterministic() {
    let first = render_belka(LOOP_ACCUMULATE, "loop_accumulate");
    for _ in 0..4 {
        assert_eq!(render_belka(LOOP_ACCUMULATE, "loop_accumulate"), first);
    }
}

#[test]
fn golden_belka_loop_accumulate() {
    let svg = render_belka(LOOP_ACCUMULATE, "loop_accumulate");
    insta::with_settings!({ snapshot_path => "fixtures/snapshots", prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!("belka_loop_accumulate", svg);
    });
}

#[test]
fn reducer_fixture_is_flagged_in_type_info() {
    let graph = parse_function(
        include_str!("../../../fixtures/reduce_brightness.rs"),
        "reduce_brightness",
    )
    .expect("fixture は必ずパースできる");
    let main = &graph.modules[0].sigils[0];
    assert!(
        main.layers
            .type_info
            .as_ref()
            .is_some_and(|t| t.reducer_shape),
        "(A, B) -> A は Reducer 形として検出される"
    );
}
