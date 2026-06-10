//! Spell Diff の構造テスト (Phase 3.1 受け入れ基準, spec v0.3 §9.2)。
//!
//! fixtures/diff/ の before/after ペアは1組で4象限 (追加/削除/変更/不変) を
//! 全て踏むよう設計している (fixture 側のコメント参照)。

use magia_core::diff::{SpellDiff, diff};
use magia_core::ir::MagiaGraph;
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
    // else 分岐の新設は部分木ごと (リング + 配下の召喚) 追加として平坦化される。
    assert!(result.added.iter().any(|p| p.contains("召喚 audit")));
    assert!(
        result
            .added
            .iter()
            .any(|p| p.contains("else分岐") && !p.contains("召喚"))
    );
    assert!(
        result
            .added
            .iter()
            .any(|p| p.contains("else分岐") && p.contains("召喚 log_small"))
    );
}

#[test]
fn removed_nodes_are_reported_with_paths() {
    let result = fixture_diff();
    assert_eq!(result.removed, vec!["main > 召喚 notify".to_string()]);
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
        .map(String::as_str)
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
