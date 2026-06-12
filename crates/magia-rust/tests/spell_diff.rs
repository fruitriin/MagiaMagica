//! Spell Diff の構造テスト (Phase 3.1 受け入れ基準, spec v0.3 §9.2) と
//! 視覚的 Spell Diff = overlay-diff チャネルのテスト (Phase 3.2, spec v0.3 §8)。
//!
//! fixtures/diff/ の before/after ペアは1組で4象限 (追加/削除/変更/不変) を
//! 全て踏むよう設計している (fixture 側のコメント参照)。

use magia_core::diff::{SpellDiff, diff};
use magia_core::ir::MagiaGraph;
use magia_core::render::ir_export::diff_spell_ir;
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

/// 差分強調つき IR (Phase 4.3 M5 — SVG 検証は Vue SSR 側 (vitest / cli 統合) が担い、
/// ここでは配置済みマークの構造を見る)。
fn fixture_diff_ir() -> (
    magia_core::render::ir_export::SpellIr,
    Vec<magia_core::render::ir_export::DiffMarkIr>,
) {
    let before = graph_of(BEFORE);
    let after = graph_of(AFTER);
    let result = diff(&before, &after);
    diff_spell_ir(&before, &after, &result)
}

#[test]
fn diff_ir_marks_all_three_quadrants() {
    let (_, marks) = fixture_diff_ir();
    let json = serde_json::to_value(&marks).unwrap();
    let count = |status: &str| {
        json.as_array()
            .unwrap()
            .iter()
            .filter(|m| m["status"] == status)
            .count()
    };
    // fixture 設計: 追加3 (audit + else リング + log_small)、変更2 (main + if 分岐)、削除1 (notify)。
    assert_eq!(count("added"), 3);
    assert_eq!(count("changed"), 2);
    assert_eq!(count("removed"), 1);
    // 描画順 = removed → changed → added (注目度順、配列順で固定)。
    let statuses: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["status"].as_str().unwrap())
        .collect();
    assert_eq!(
        statuses,
        ["removed", "changed", "changed", "added", "added", "added"]
    );
}

#[test]
fn diff_ir_is_deterministic() {
    let (ir_a, marks_a) = fixture_diff_ir();
    let (ir_b, marks_b) = fixture_diff_ir();
    assert_eq!(
        serde_json::to_string(&ir_a).unwrap(),
        serde_json::to_string(&ir_b).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&marks_a).unwrap(),
        serde_json::to_string(&marks_b).unwrap()
    );
}

#[test]
fn empty_diff_yields_no_marks() {
    let graph = graph_of(BEFORE);
    let result = diff(&graph, &graph);
    let (_, marks) = diff_spell_ir(&graph, &graph, &result);
    assert!(marks.is_empty());
}

#[test]
fn ghost_extends_viewbox_only_when_needed() {
    // 削除ゴーストの座標 (before レイアウト由来) が viewBox の矩形内に完全に
    // 収まるよう拡張される (はみ出すケースの取りこぼし防止)。
    let (ir, marks) = fixture_diff_ir();
    let json = serde_json::to_value(&ir).unwrap();
    let vb: Vec<f64> = json["view_box"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    let (x0, y0, w, h) = (vb[0], vb[1], vb[2], vb[3]);
    let marks_json = serde_json::to_value(&marks).unwrap();
    let ghost = marks_json
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["status"] == "removed")
        .expect("ゴーストが出ている");
    let (cx, cy, r) = (
        ghost["x"].as_f64().unwrap(),
        ghost["y"].as_f64().unwrap(),
        ghost["radius"].as_f64().unwrap(),
    );
    assert!(cx - r >= x0 && cx + r <= x0 + w);
    assert!(cy - r >= y0 && cy + r <= y0 + h);
}

#[test]
fn chain_followers_participate_in_diff() {
    // Phase 4.8 レビュー Critical 1 の回帰: 鎖の後続 glyph (Chain edge の子) も
    // diff の木に入り、伸びた鎖が追加として検出される。
    let before = parse_function("fn f(v: V) { v.iter().count(); }", "f").unwrap();
    let after = parse_function("fn f(v: V) { v.iter().filter(keep).count(); }", "f").unwrap();
    let result = diff(&before, &after);
    let added: Vec<&str> = result.added.iter().map(|n| n.path.as_str()).collect();
    assert!(
        added.iter().any(|p| p.contains(".filter")),
        "鎖に挿入された .filter が追加検出される: {added:?}"
    );
}
