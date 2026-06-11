//! データフロー抽出のテスト (Phase 3.4 受け入れ基準, spec §5.1)。
//!
//! loop_accumulate fixture は def/use の代表形を1本で踏む:
//! 親で def → 子で use (下り)、子で再定義 → 親で use (上り)、
//! for パターン束縛 → 入れ子 if での use (兄弟スコープ間)。

use magia_core::ir::{EdgeKind, MagiaGraph, Module};
use magia_rust::parse_function;

const LOOP_ACCUMULATE: &str = include_str!("../../../fixtures/loop_accumulate.rs");

fn module_of(source: &str, fn_name: &str) -> Module {
    let graph: MagiaGraph = parse_function(source, fn_name).expect("fixture は必ずパースできる");
    graph.modules.into_iter().next().expect("モジュールがある")
}

/// (source, target) → variables の組で DataFlow Edge を取り出す。
fn dataflow_edges(module: &Module) -> Vec<(u32, u32, Vec<String>)> {
    module
        .edges
        .iter()
        .filter(|e| e.kind == EdgeKind::DataFlow)
        .map(|e| {
            (
                e.source.0,
                e.target.0,
                e.layers
                    .data_flow
                    .as_ref()
                    .map(|d| d.variables.clone())
                    .unwrap_or_default(),
            )
        })
        .collect()
}

#[test]
fn defs_and_uses_are_recorded_on_operations() {
    let module = module_of(LOOP_ACCUMULATE, "loop_accumulate");
    let main = &module.sigils[0];
    // let mut total = 0; → total の def。
    assert_eq!(main.content[0].payload.defs, vec!["total"]);
    // for item in items → items は引数の use、item は構文上ここで生まれる。
    assert_eq!(main.content[1].payload.defs, vec!["item"]);
    assert_eq!(main.content[1].payload.uses, vec!["items"]);
    // 末尾の total (返却) は use。
    let last = main.content.last().expect("content がある");
    assert_eq!(last.payload.uses, vec!["total"]);
}

#[test]
fn cross_ring_flows_become_dataflow_edges() {
    let module = module_of(LOOP_ACCUMULATE, "loop_accumulate");
    let edges = dataflow_edges(&module);
    // 注: SigilId の生値は「深さ優先・ソース出現順」の採番規約に依存する
    // (0=main, 1=forループ本体, 2=if分岐)。fixture か採番規約を変えたら要更新。
    // 下り: main(0) の total → 複合代入リング(2)。
    assert!(edges.contains(&(0, 2, vec!["total".to_string()])));
    // 上り: ループ内で再定義された total が main へ還流する (ベルカ式の「変換→消費」)。
    assert!(edges.contains(&(2, 0, vec!["total".to_string()])));
    // 兄弟スコープ: for パターンの item がループ本体(1) から if 分岐(2) へ。
    assert!(edges.contains(&(1, 2, vec!["item".to_string()])));
}

#[test]
fn aux_ring_payload_explains_defs_and_uses() {
    // 説明可能性の核心: AuxRing 内の Operation にも defs/uses が残る。
    let module = module_of(LOOP_ACCUMULATE, "loop_accumulate");
    let compound = module
        .sigils
        .iter()
        .flat_map(|s| &s.content)
        .find(|op| {
            op.payload
                .source_excerpt
                .as_deref()
                .is_some_and(|e| e.contains("total +="))
        })
        .expect("total += の Operation がある");
    assert_eq!(compound.payload.defs, vec!["total"]);
    assert_eq!(compound.payload.uses, vec!["item", "total"]);
    // while ガードは制御 Operation の uses に載る。
    let while_op = module
        .sigils
        .iter()
        .flat_map(|s| &s.content)
        .find(|op| {
            op.payload
                .source_excerpt
                .as_deref()
                .is_some_and(|e| e.starts_with("while"))
        })
        .expect("while の Operation がある");
    assert_eq!(while_op.payload.uses, vec!["total"]);
}

#[test]
fn ring_stats_aggregate_chains() {
    let module = module_of(LOOP_ACCUMULATE, "loop_accumulate");
    let main = &module.sigils[0];
    let info = main
        .layers
        .data_flow
        .as_ref()
        .expect("リングには DataFlowInfo が入る");
    // main で def: total (使われる) + items は引数 (使われる) = 2 チェーン。
    assert_eq!(info.use_def_chains, 2);
    assert!(info.longest_chain >= 2);
    // glyph には DataFlowInfo を入れない。
    let glyph = module
        .sigils
        .iter()
        .find(|s| s.kind == magia_core::ir::SigilKind::SummonGlyph)
        .expect("召喚記号がある");
    assert!(glyph.layers.data_flow.is_none());
}

#[test]
fn extraction_is_deterministic() {
    let first = module_of(LOOP_ACCUMULATE, "loop_accumulate");
    for _ in 0..4 {
        assert_eq!(module_of(LOOP_ACCUMULATE, "loop_accumulate"), first);
    }
}

#[test]
fn shadowing_and_match_bindings_are_tracked() {
    let source = r"
fn rebind(input: u32) -> u32 {
    let value = input + 1;
    let value = value * 2;
    match classify(value) {
        Some(extracted) => {
            let result = extracted + value;
            result
        }
        None => 0,
    }
}
";
    let module = module_of(source, "rebind");
    let main = &module.sigils[0];
    // シャドーイング: 2行目の let value は旧 value を use しつつ新 def。
    assert_eq!(main.content[1].payload.defs, vec!["value"]);
    assert_eq!(main.content[1].payload.uses, vec!["value"]);
    // match アームの束縛 extracted は Some アーム内で use される (同リング → Edge なし)。
    // value は main → Some アームへ流れる。
    let edges = dataflow_edges(&module);
    assert!(
        edges
            .iter()
            .any(|(s, _, vars)| *s == 0 && vars.contains(&"value".to_string())),
        "value が main からアームへ流れる: {edges:?}"
    );
}

#[test]
fn closures_and_unresolved_names_are_ignored() {
    let source = r"
fn capture(base: u32) -> u32 {
    let add = |x: u32| x + base;
    add(base)
}
";
    let module = module_of(source, "capture");
    let main = &module.sigils[0];
    // クロージャ内の base は追わない (let add の uses は空)。
    assert_eq!(main.content[0].payload.defs, vec!["add"]);
    assert!(main.content[0].payload.uses.is_empty());
    // add(base) は add (ローカルのクロージャ変数) と base の use。
    assert_eq!(main.content[1].payload.uses, vec!["add", "base"]);
}
