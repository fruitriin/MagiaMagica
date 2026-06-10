//! Spell Diff の構造テスト (Phase 3.1 受け入れ基準, spec v0.3 §9.2) と
//! 視覚的 Spell Diff = overlay-diff チャネルのテスト (Phase 3.2, spec v0.3 §8)。
//!
//! fixtures/diff/ の before/after ペアは1組で4象限 (追加/削除/変更/不変) を
//! 全て踏むよう設計している (fixture 側のコメント参照)。

use magia_core::diff::{SpellDiff, diff};
use magia_core::filter::FilterSpec;
use magia_core::ir::MagiaGraph;
use magia_core::render::{RenderStyle, render_diff};
use magia_rust::parse_function;

const BEFORE: &str = include_str!("../../../fixtures/diff/before.rs");
const AFTER: &str = include_str!("../../../fixtures/diff/after.rs");

fn graph_of(source: &str) -> MagiaGraph {
    parse_function(source, "process_order").expect("fixture は必ずパースできる")
}

fn fixture_diff() -> SpellDiff {
    diff(&graph_of(BEFORE), &graph_of(AFTER))
}

#[test]
fn added_nodes_are_reported_with_paths() {
    let result = fixture_diff();
    let paths: Vec<&str> = result.added.iter().map(|n| n.path.as_str()).collect();
    // else 分岐の新設は部分木ごと (リング + 配下の召喚) 追加として平坦化される。
    assert!(paths.iter().any(|p| p.contains("召喚 audit")));
    assert!(
        paths
            .iter()
            .any(|p| p.contains("else分岐") && !p.contains("召喚"))
    );
    assert!(
        paths
            .iter()
            .any(|p| p.contains("else分岐") && p.contains("召喚 log_small"))
    );
}

#[test]
fn removed_nodes_are_reported_with_paths() {
    let result = fixture_diff();
    assert_eq!(result.removed.len(), 1);
    assert_eq!(result.removed[0].path, "main > 召喚 notify");
}

#[test]
fn node_ids_resolve_in_their_own_revision() {
    // added の ID は after 側、removed の ID は before 側で実在する (overlay 描画の前提)。
    let before = graph_of(BEFORE);
    let after = graph_of(AFTER);
    let result = diff(&before, &after);
    let ids_of = |graph: &MagiaGraph| -> Vec<_> {
        graph
            .modules
            .iter()
            .flat_map(|m| &m.sigils)
            .map(|s| s.id)
            .collect()
    };
    let before_ids = ids_of(&before);
    let after_ids = ids_of(&after);
    assert!(result.added.iter().all(|n| after_ids.contains(&n.sigil)));
    assert!(result.removed.iter().all(|n| before_ids.contains(&n.sigil)));
    assert!(
        result
            .changed
            .iter()
            .all(|c| before_ids.contains(&c.before) && after_ids.contains(&c.after))
    );
}

#[test]
fn changed_nodes_carry_japanese_details() {
    let result = fixture_diff();
    // main は notify 削除で操作数が減り、if 分岐は audit 追加で操作数が増える。
    let main = result
        .changed
        .iter()
        .find(|c| c.path == "main")
        .expect("main の変更が報告される");
    assert!(main.details.iter().any(|d| d.starts_with("操作数")));
    let if_branch = result
        .changed
        .iter()
        .find(|c| c.path.contains("if分岐"))
        .expect("if 分岐の変更が報告される");
    assert!(if_branch.details.contains(&"操作数 1 → 2".to_string()));
}

#[test]
fn unchanged_subtrees_stay_silent() {
    let result = fixture_diff();
    // for ループ本体と step 召喚は両リビジョンで不変 — どの象限にも現れない。
    let all_paths: Vec<&str> = result
        .added
        .iter()
        .chain(&result.removed)
        .map(|n| n.path.as_str())
        .chain(result.changed.iter().map(|c| c.path.as_str()))
        .collect();
    assert!(!all_paths.iter().any(|p| p.contains("ループ本体")));
    assert!(!all_paths.iter().any(|p| p.contains("召喚 step")));
}

#[test]
fn metrics_delta_uses_shared_measure() {
    let result = fixture_diff();
    // リング数 3 → 4 (else 追加)、召喚 4 → 5 (notify 削除 + audit/log_small 追加)。
    assert_eq!(result.metrics.before.rings, 3);
    assert_eq!(result.metrics.after.rings, 4);
    assert_eq!(result.metrics.before.glyphs, 4);
    assert_eq!(result.metrics.after.glyphs, 5);
}

#[test]
fn identical_input_yields_empty_diff() {
    let result = diff(&graph_of(BEFORE), &graph_of(BEFORE));
    assert!(result.is_empty());
    assert_eq!(result.metrics.before, result.metrics.after);
    assert!(result.to_report("process_order").contains("構造の変化なし"));
}

#[test]
fn diff_is_deterministic_across_runs() {
    let first = fixture_diff();
    for _ in 0..4 {
        assert_eq!(fixture_diff(), first);
    }
}

#[test]
fn report_lists_only_changed_metrics() {
    let report = fixture_diff().to_report("process_order");
    // 複雑度は 3 → 3 で不変 — 変化のないメトリクスは行に出さない。
    assert!(report.contains("リング数 3 → 4"));
    assert!(!report.contains("複雑度"));
}

// ===== Phase 3.2: overlay-diff チャネル =====

fn fixture_diff_svg(filter: &FilterSpec) -> String {
    let before = graph_of(BEFORE);
    let after = graph_of(AFTER);
    let result = diff(&before, &after);
    render_diff(
        &before,
        &after,
        &result,
        RenderStyle::MidchildaConcentric,
        filter,
    )
}

#[test]
fn overlay_draws_all_three_quadrants() {
    let svg = fixture_diff_svg(&FilterSpec::default());
    assert!(svg.contains(r#"<g class="overlay-diff">"#));
    // fixture 設計: 追加3 (audit + else リング + log_small)、変更2 (main + if 分岐)、削除1 (notify)。
    assert_eq!(svg.matches(r#"class="diff-added""#).count(), 3);
    assert_eq!(svg.matches(r#"class="diff-changed""#).count(), 2);
    assert_eq!(svg.matches(r#"class="diff-removed""#).count(), 1);
    // ゴーストは破線 (本体と見間違えない)。
    assert!(svg.contains("stroke-dasharray=\"5 4\""));
}

#[test]
fn overlay_ignores_layer_gating() {
    // spec v0.3 §8: 強調チャネルはレイヤーの show/hide の影響を受けない。
    let filter = FilterSpec::parse("show: control_flow\nhighlight: changed\n").unwrap();
    let svg = fixture_diff_svg(&filter);
    assert!(!svg.contains("layer-effects"), "effects 層は隠れている");
    assert_eq!(
        svg.matches(r#"class="diff-added""#).count(),
        3,
        "召喚記号 (effects 層由来) の強調も残る"
    );
}

#[test]
fn render_diff_is_deterministic() {
    let first = fixture_diff_svg(&FilterSpec::default());
    assert_eq!(fixture_diff_svg(&FilterSpec::default()), first);
}

#[test]
fn empty_diff_renders_empty_overlay() {
    let graph = graph_of(BEFORE);
    let result = diff(&graph, &graph);
    let svg = render_diff(
        &graph,
        &graph,
        &result,
        RenderStyle::MidchildaConcentric,
        &FilterSpec::default(),
    );
    assert!(
        svg.contains("overlay-diff"),
        "チャネル自体は diff モードの印として出る"
    );
    assert!(!svg.contains("diff-added"));
    assert!(!svg.contains("diff-removed"));
    assert!(!svg.contains("diff-changed"));
}

#[test]
fn ghost_extends_viewbox_only_when_needed() {
    // 削除ゴーストの座標が after のキャンバスに収まるよう viewBox が拡張される。
    // ゴースト円 (cx, cy, r) が viewBox の矩形内に完全に入ることを数値で検証する。
    // 拡張側の reach (sigil_radius + DIFF_HALO_OFFSET + DIFF_HALO_STROKE) はゴーストの
    // r = sigil_radius より広いため、この内包判定には線幅分の余白が織り込まれている。
    let svg = fixture_diff_svg(&FilterSpec::default());
    let view_box = svg
        .split("viewBox=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("viewBox がある");
    let numbers: Vec<f64> = view_box
        .split(' ')
        .map(|token| token.parse().expect("viewBox は数値4つ"))
        .collect();
    let (x0, y0, w, h) = (numbers[0], numbers[1], numbers[2], numbers[3]);
    let ghost_line = svg
        .lines()
        .find(|line| line.contains("diff-removed"))
        .expect("ゴーストが出ている");
    let attr = |name: &str| -> f64 {
        ghost_line
            .split(&format!("{name}=\""))
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("属性がある")
            .parse()
            .expect("数値")
    };
    let (cx, cy, r) = (attr("cx"), attr("cy"), attr("r"));
    assert!(cx - r >= x0 && cx + r <= x0 + w);
    assert!(cy - r >= y0 && cy + r <= y0 + h);
}

#[test]
fn golden_diff_overlay() {
    let svg = fixture_diff_svg(&FilterSpec::default());
    insta::with_settings!({ snapshot_path => "fixtures/snapshots", prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!("diff_overlay_process_order", svg);
    });
}
