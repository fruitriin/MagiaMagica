//! MagiaMagica Rust language adapter.
//!
//! Phase 1.2 (M2): syn 2.x ベースで単一 Rust 関数を MagiaIR に変換する。
//! Phase 1.3 (M3): 制御構造 (if/match/ループ) を AuxRing に切り出し、ControlFlow Edge で
//! 親リングと接続する。呼び出し先 (SummonGlyph) と効果判定は Phase 1.4 で追加する。

mod allocator;
mod error;
mod ring;
mod signature;
mod statement;

pub use error::Error;

use magia_core::ir::{ConcurrencyInfo, MagiaGraph, Module, ModuleId, ProjectMetadata};
use syn::visit::Visit;
use syn::{File, ItemFn};

use crate::allocator::SigilIdAllocator;
use crate::ring::build_rings;
use crate::signature::extract_type_info;
use crate::statement::ParseContext;

/// Rust ソースから関数定義の名前一覧を返す。
///
/// トップレベル・`mod { ... }` 内・関数内のネスト `fn` を再帰的に収集する。
/// `impl` ブロック内のメソッドは Phase 1.2 のスコープ外 (Phase 1.4 以降で検討)。
///
/// 規約: 本関数が返す任意の名前は `parse_function` で必ず発見できる
/// (両者の探索範囲は意図的に一致させてある)。
#[must_use = "関数一覧は呼び出し側で利用されるべき"]
pub fn list_functions(source: &str) -> Result<Vec<String>, Error> {
    let file: File = syn::parse_str(source)?;
    let mut collector = FunctionNameCollector { names: Vec::new() };
    collector.visit_file(&file);
    Ok(collector.names)
}

/// 指定された名前の関数を MagiaIR に変換する。
///
/// Phase 1.3 の出力は `MainRing` 1個 + 制御構造ごとの `AuxRing` 群 + 両者を結ぶ
/// `ControlFlow` Edge。SummonGlyph (呼び出し先) は Phase 1.4 で追加する。
#[must_use = "IR は呼び出し側で利用されるべき"]
pub fn parse_function(source: &str, fn_name: &str) -> Result<MagiaGraph, Error> {
    let file: File = syn::parse_str(source)?;
    let item_fn = find_function(&file, fn_name)?;
    let mut allocator = SigilIdAllocator::new();
    let ctx = ParseContext {
        fn_is_unsafe: item_fn.sig.unsafety.is_some(),
    };
    let mut forest = build_rings(item_fn, &mut allocator, ctx);

    // 関数レベルのレイヤー (シグネチャ・並行性) は MainRing にのみ載せる。
    // AuxRing は制御フロー情報 (`control_flow.role`) だけを持つ。
    let main_ring = &mut forest.sigils[0];
    main_ring.layers.type_info = Some(extract_type_info(&item_fn.sig));
    main_ring.layers.concurrency = Some(ConcurrencyInfo {
        is_async: item_fn.sig.asyncness.is_some(),
        await_points: count_await_points(item_fn),
    });

    // Phase 1.2 ではファイル/プロジェクト解析が無いため、`Module.name` と
    // `ProjectMetadata.name` の両方に関数名をプレースホルダーとして入れる。
    // ファイル粒度の解析 (Phase 1.5 以降) で実ファイルパス・プロジェクト名で上書きする。
    let module = Module {
        id: ModuleId(0),
        name: fn_name.to_string(),
        sigils: forest.sigils,
        edges: forest.edges,
    };
    Ok(MagiaGraph {
        modules: vec![module],
        cross_module_edges: Vec::new(),
        metadata: ProjectMetadata {
            name: fn_name.to_string(),
            version: None,
            root_path: None,
        },
    })
}

fn find_function<'a>(file: &'a File, fn_name: &str) -> Result<&'a ItemFn, Error> {
    let mut collector = FunctionRefCollector {
        target: fn_name.to_string(),
        all_names: Vec::new(),
        found: None,
    };
    collector.visit_file(file);
    if let Some(item) = collector.found {
        return Ok(item);
    }
    Err(Error::FunctionNotFound {
        name: fn_name.to_string(),
        candidates: collector.all_names,
    })
}

/// 関数本体内の `.await` 数を数える。
///
/// ネストした `async { ... }` ブロック内部の `.await` も合算する (Phase 1 の近似)。
/// 実装上「外側関数の await」と「内側 async ブロックの await」を区別する解析は重く、
/// Phase 1 の "await の重み付け表現" レベルでは合計値で十分。
fn count_await_points(item_fn: &ItemFn) -> u32 {
    struct Counter {
        count: u32,
    }
    impl<'ast> Visit<'ast> for Counter {
        fn visit_expr_await(&mut self, node: &'ast syn::ExprAwait) {
            self.count += 1;
            syn::visit::visit_expr_await(self, node);
        }
    }
    let mut counter = Counter { count: 0 };
    counter.visit_block(&item_fn.block);
    counter.count
}

struct FunctionNameCollector {
    names: Vec<String>,
}

impl<'ast> Visit<'ast> for FunctionNameCollector {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.names.push(node.sig.ident.to_string());
        // 関数内の `fn` ネストや `mod` 内宣言も拾う。`FunctionRefCollector` と探索範囲を揃え、
        // 列挙された名前を `parse_function` で必ず再発見できる API 規約を保つ。
        syn::visit::visit_item_fn(self, node);
    }
}

struct FunctionRefCollector<'ast> {
    target: String,
    all_names: Vec<String>,
    found: Option<&'ast ItemFn>,
}

impl<'ast> Visit<'ast> for FunctionRefCollector<'ast> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let name = node.sig.ident.to_string();
        self.all_names.push(name.clone());
        if self.found.is_none() && name == self.target {
            self.found = Some(node);
        }
        // `list_functions` の `FunctionNameCollector` と探索範囲を揃え、列挙された名前が
        // 必ず `parse_function` で再発見できることを保証する。
        syn::visit::visit_item_fn(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use magia_core::ir::{AuxRingKind, EdgeKind, LoopKind, OperationKind, Sigil, SigilKind};

    /// モジュール内の AuxRing 一覧 (SigilId 昇順)。
    fn aux_rings(module: &Module) -> Vec<&Sigil> {
        module
            .sigils
            .iter()
            .filter(|s| s.kind == SigilKind::AuxRing)
            .collect()
    }

    /// 受け入れ基準の不変条件: ControlFlow Edge 数 = AuxRing 数、SigilId は一意、
    /// 各 AuxRing にちょうど1本の Edge が入る。
    fn assert_ring_invariants(module: &Module) {
        let aux = aux_rings(module);
        assert_eq!(
            module
                .edges
                .iter()
                .filter(|e| e.kind == EdgeKind::ControlFlow)
                .count(),
            aux.len(),
            "ControlFlow Edge 数 = AuxRing 数"
        );
        let mut ids: Vec<_> = module.sigils.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), module.sigils.len(), "SigilId は一意");
        for ring in &aux {
            assert_eq!(
                module.edges.iter().filter(|e| e.target == ring.id).count(),
                1,
                "各 AuxRing は親と1本の Edge を持つ"
            );
        }
    }

    #[test]
    fn list_functions_returns_top_level_names() {
        let names = list_functions("fn alpha() {} fn beta() {}").unwrap();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn list_functions_traverses_modules() {
        let src = "mod inner { fn nested() {} } fn outer() {}";
        let mut names = list_functions(src).unwrap();
        names.sort();
        assert_eq!(names, vec!["nested", "outer"]);
    }

    #[test]
    fn listed_names_are_all_parseable() {
        // API 規約: list_functions の任意の戻り値で parse_function が成功する。
        let src = "
            mod inner { fn nested() -> u8 { 1 } }
            fn outer() {}
            async fn awaiter() { foo().await; }
        ";
        let names = list_functions(src).unwrap();
        assert!(!names.is_empty());
        for name in &names {
            parse_function(src, name)
                .unwrap_or_else(|e| panic!("`{name}` should be parseable: {e}"));
        }
    }

    #[test]
    fn parse_function_yields_single_main_ring_with_operations() {
        let src = "fn foo() -> i32 { let x = 1; let y = 2; x + y }";
        let graph = parse_function(src, "foo").unwrap();
        let module = &graph.modules[0];
        assert_eq!(module.sigils.len(), 1);
        let main = &module.sigils[0];
        assert_eq!(main.kind, SigilKind::MainRing);
        assert_eq!(main.content.len(), 3);
        assert!(
            main.content
                .iter()
                .all(|op| op.kind == OperationKind::Compute)
        );
    }

    #[test]
    fn parse_function_marks_return_statement() {
        let src = "fn bar() { return; }";
        let graph = parse_function(src, "bar").unwrap();
        let ops = &graph.modules[0].sigils[0].content;
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].kind, OperationKind::Return);
        assert!(ops[0].payload.early_return);
    }

    #[test]
    fn parse_function_records_async_and_await_points() {
        let src = "async fn baz() { foo().await; bar().await; }";
        let graph = parse_function(src, "baz").unwrap();
        let layers = &graph.modules[0].sigils[0].layers;
        let concurrency = layers.concurrency.as_ref().unwrap();
        assert!(concurrency.is_async);
        assert_eq!(concurrency.await_points, 2);
    }

    #[test]
    fn parse_function_records_unsafe_function() {
        let src = "unsafe fn dangerous() { core::hint::unreachable_unchecked(); }";
        let graph = parse_function(src, "dangerous").unwrap();
        let ops = &graph.modules[0].sigils[0].content;
        assert!(ops.iter().all(|op| op.effects.unsafe_block));
    }

    #[test]
    fn parse_function_records_signature_and_result_flag() {
        let src = "fn try_parse(input: &str) -> Result<i32, String> { todo!() }";
        let graph = parse_function(src, "try_parse").unwrap();
        let layers = &graph.modules[0].sigils[0].layers;
        let type_info = layers.type_info.as_ref().unwrap();
        assert!(type_info.signature.as_ref().unwrap().contains("try_parse"));
        assert!(type_info.returns_result);
        assert!(!type_info.returns_option);
    }

    #[test]
    fn parse_function_records_option_return() {
        let src = "fn find_id() -> Option<u32> { Some(42) }";
        let graph = parse_function(src, "find_id").unwrap();
        let type_info = graph.modules[0].sigils[0]
            .layers
            .type_info
            .as_ref()
            .unwrap();
        assert!(type_info.returns_option);
        assert!(!type_info.returns_result);
    }

    #[test]
    fn parse_function_not_found_reports_candidates() {
        let err = parse_function("fn alpha() {} fn beta() {}", "gamma").unwrap_err();
        match err {
            Error::FunctionNotFound { name, candidates } => {
                assert_eq!(name, "gamma");
                assert_eq!(candidates, vec!["alpha", "beta"]);
            }
            Error::Syntax(syntax) => panic!("unexpected syntax error: {syntax:?}"),
        }
    }

    #[test]
    fn parse_function_returns_syntax_error_for_invalid_source() {
        let err = parse_function("fn broken( {", "broken").unwrap_err();
        assert!(matches!(err, Error::Syntax(_)));
    }

    #[test]
    fn parse_function_is_deterministic_on_json() {
        let src = "fn deterministic() -> i32 { 7 }";
        let graph = parse_function(src, "deterministic").unwrap();
        let json_a = serde_json::to_string(&graph).unwrap();
        let json_b = serde_json::to_string(&graph).unwrap();
        assert_eq!(json_a, json_b);
    }

    // ===== Phase 1.3: 制御構造の AuxRing 化 =====

    #[test]
    fn if_else_yields_two_aux_rings_and_edges() {
        let src = "fn f(a: bool) -> i32 { if a { 1 } else { 2 } }";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        assert_ring_invariants(module);

        let aux = aux_rings(module);
        assert_eq!(aux.len(), 2);
        let roles: Vec<_> = aux
            .iter()
            .map(|s| {
                s.layers
                    .control_flow
                    .as_ref()
                    .unwrap()
                    .role
                    .as_ref()
                    .unwrap()
            })
            .collect();
        assert_eq!(roles[0].kind, AuxRingKind::IfBranch);
        assert_eq!(roles[1].kind, AuxRingKind::ElseBranch);
        assert_eq!((roles[0].ordinal, roles[1].ordinal), (0, 1));

        // MainRing 側: Branch Operation 1個、branch_count = 1 (if チェーンで1)。
        let main = &module.sigils[0];
        assert_eq!(main.kind, SigilKind::MainRing);
        assert_eq!(main.content.len(), 1);
        assert_eq!(main.content[0].kind, OperationKind::Branch);
        let info = main.layers.control_flow.as_ref().unwrap();
        assert_eq!(info.branch_count, 1);
        assert!(info.role.is_none());
    }

    #[test]
    fn match_yields_aux_ring_per_arm_with_pattern_labels() {
        let src = "fn f(x: u8) -> u8 { match x { 1 => a(), 2 => b(), _ => c(), } }";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        assert_ring_invariants(module);

        let aux = aux_rings(module);
        assert_eq!(aux.len(), 3);
        let labels: Vec<_> = aux
            .iter()
            .map(|s| {
                let role = s
                    .layers
                    .control_flow
                    .as_ref()
                    .unwrap()
                    .role
                    .as_ref()
                    .unwrap();
                assert_eq!(role.kind, AuxRingKind::MatchArm);
                role.label.clone().unwrap()
            })
            .collect();
        assert_eq!(labels, vec!["1", "2", "_"]);

        let main = &module.sigils[0];
        assert_eq!(main.content[0].kind, OperationKind::Match);
        // branch_count = アーム数 - 1。
        assert_eq!(main.layers.control_flow.as_ref().unwrap().branch_count, 2);
    }

    #[test]
    fn for_loop_yields_single_loop_aux_ring() {
        let src = "fn f() { for i in 0..10 { use_it(i); } }";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        assert_ring_invariants(module);

        let aux = aux_rings(module);
        assert_eq!(aux.len(), 1);
        let role = aux[0]
            .layers
            .control_flow
            .as_ref()
            .unwrap()
            .role
            .as_ref()
            .unwrap();
        assert_eq!(role.kind, AuxRingKind::LoopBody(LoopKind::For));
        assert_eq!(role.ordinal, 0);

        let main = &module.sigils[0];
        assert_eq!(main.content[0].kind, OperationKind::Loop);
        assert_eq!(main.layers.control_flow.as_ref().unwrap().loop_count, 1);
    }

    #[test]
    fn if_let_yields_single_aux_ring() {
        let src = "fn f(opt: Option<u8>) { if let Some(x) = opt { use_it(x); } }";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        assert_ring_invariants(module);

        let aux = aux_rings(module);
        assert_eq!(aux.len(), 1);
        let role = aux[0]
            .layers
            .control_flow
            .as_ref()
            .unwrap()
            .role
            .as_ref()
            .unwrap();
        assert_eq!(role.kind, AuxRingKind::IfBranch);
    }

    #[test]
    fn else_if_chain_yields_ring_per_branch() {
        let src = "fn f(a: bool, b: bool) { if a { x(); } else if b { y(); } else { z(); } }";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        assert_ring_invariants(module);

        let aux = aux_rings(module);
        assert_eq!(aux.len(), 3);
        let roles: Vec<_> = aux
            .iter()
            .map(|s| {
                s.layers
                    .control_flow
                    .as_ref()
                    .unwrap()
                    .role
                    .as_ref()
                    .unwrap()
            })
            .collect();
        assert_eq!(roles[0].kind, AuxRingKind::IfBranch);
        assert_eq!(roles[1].kind, AuxRingKind::IfBranch);
        assert_eq!(roles[2].kind, AuxRingKind::ElseBranch);
        assert_eq!(
            roles.iter().map(|r| r.ordinal).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        // 連鎖全体は親リング上の同一 Operation に係留される。
        assert!(roles.iter().all(|r| r.anchor_operation == 0));
        // if チェーン全体で branch_count = 1。
        let main = &module.sigils[0];
        assert_eq!(main.layers.control_flow.as_ref().unwrap().branch_count, 1);
    }

    #[test]
    fn nested_control_structures_nest_aux_rings() {
        let src = "fn f(c: bool, x: u8) { if c { match x { 1 => a(), _ => b(), } } }";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        assert_ring_invariants(module);

        // MainRing + if 分岐 + match アーム2 = 4 Sigil。
        assert_eq!(module.sigils.len(), 4);
        let aux = aux_rings(module);
        let if_ring = aux
            .iter()
            .find(|s| {
                s.layers
                    .control_flow
                    .as_ref()
                    .unwrap()
                    .role
                    .as_ref()
                    .unwrap()
                    .kind
                    == AuxRingKind::IfBranch
            })
            .unwrap();
        // if 分岐リングは MainRing から、match アームは if 分岐リングから接続される。
        let main_id = module.sigils[0].id;
        for edge in &module.edges {
            if edge.target == if_ring.id {
                assert_eq!(edge.source, main_id);
            } else {
                assert_eq!(edge.source, if_ring.id);
            }
        }
        // if 分岐リング自身の content には match の Operation が1個。
        assert_eq!(if_ring.content.len(), 1);
        assert_eq!(if_ring.content[0].kind, OperationKind::Match);
    }

    #[test]
    fn early_return_is_counted_in_owning_ring() {
        let src = "fn f(c: bool) -> u8 { if c { return 1; } 0 }";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        assert_ring_invariants(module);

        let main_info = module.sigils[0].layers.control_flow.as_ref().unwrap();
        assert_eq!(
            main_info.early_return_count, 0,
            "return は分岐リング側に計上"
        );
        let aux = aux_rings(module);
        let aux_info = aux[0].layers.control_flow.as_ref().unwrap();
        assert_eq!(aux_info.early_return_count, 1);
    }

    #[test]
    fn unsafe_fn_propagates_into_aux_rings() {
        let src = "unsafe fn f(c: bool) { if c { do_thing(); } }";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        let aux = aux_rings(module);
        assert!(
            aux[0].content.iter().all(|op| op.effects.unsafe_block),
            "unsafe fn のコンテキストは AuxRing 内の Operation にも伝播する"
        );
    }

    #[test]
    fn if_let_with_else_yields_else_branch_ring() {
        let src =
            "fn f(opt: Option<u8>) { if let Some(x) = opt { use_it(x); } else { fallback(); } }";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        assert_ring_invariants(module);

        let aux = aux_rings(module);
        assert_eq!(aux.len(), 2);
        let kinds: Vec<_> = aux
            .iter()
            .map(|s| {
                s.layers
                    .control_flow
                    .as_ref()
                    .unwrap()
                    .role
                    .as_ref()
                    .unwrap()
                    .kind
            })
            .collect();
        assert_eq!(kinds, vec![AuxRingKind::IfBranch, AuxRingKind::ElseBranch]);
    }

    #[test]
    fn single_arm_match_has_zero_branch_count() {
        // マクロ生成等で現れる1アーム match: 分岐は発生しないので branch_count = 0。
        let src = "fn f(x: u8) -> u8 { match x { _ => 0 } }";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        assert_ring_invariants(module);
        assert_eq!(aux_rings(module).len(), 1);
        assert_eq!(
            module.sigils[0]
                .layers
                .control_flow
                .as_ref()
                .unwrap()
                .branch_count,
            0
        );
    }

    #[test]
    fn let_binding_with_if_stays_compute() {
        // `let x = if ...` の式内制御構造は Phase 1.3 では意図的に切り出さない
        // (ring::classify のスコープ判断を回帰テストとして固定する)。
        let src = "fn f(a: bool) -> i32 { let x = if a { 1 } else { 2 }; x }";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        assert!(aux_rings(module).is_empty());
        assert!(
            module.sigils[0]
                .content
                .iter()
                .all(|op| op.kind == OperationKind::Compute)
        );
    }

    #[test]
    fn complex_function_is_deterministic_and_consistent() {
        let src = "
            fn f(a: bool, xs: Vec<u8>) -> u8 {
                let mut acc = 0;
                for x in xs {
                    if a {
                        acc += x;
                    } else {
                        match x {
                            0 => acc += 1,
                            _ => { while acc < 10 { acc += 2; } }
                        }
                    }
                }
                loop { break; }
                acc
            }
        ";
        let graph = parse_function(src, "f").unwrap();
        let module = &graph.modules[0];
        assert_ring_invariants(module);

        // 再パースしても同一 JSON (決定論的採番・決定論的順序)。
        let again = parse_function(src, "f").unwrap();
        assert_eq!(
            serde_json::to_string(&graph).unwrap(),
            serde_json::to_string(&again).unwrap()
        );
    }
}
