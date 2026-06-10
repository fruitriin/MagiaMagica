//! MagiaMagica Rust language adapter.
//!
//! Phase 1.2 (M2): syn 2.x ベースで単一 Rust 関数を MagiaIR に変換する。
//! 制御構造 (AuxRing) の切り出しは Phase 1.3、呼び出し先 (SummonGlyph) と効果判定は
//! Phase 1.4 で追加する。

mod allocator;
mod error;
mod signature;
mod statement;

pub use error::Error;

use magia_core::ir::{
    Cardinality, ConcurrencyInfo, LayerData, MagiaGraph, Module, ModuleId, ProjectMetadata, Sigil,
    SigilKind, SourceSpan,
};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, ItemFn};

use crate::allocator::SigilIdAllocator;
use crate::signature::extract_type_info;
use crate::statement::{statement_line_range, statement_to_operation};

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
/// Phase 1.2 の出力は `MainRing` 1個 + 本体の statement を Operation 列として持つ
/// 最小構成。制御構造の AuxRing 化と SummonGlyph は後続マイルストーンで追加する。
#[must_use = "IR は呼び出し側で利用されるべき"]
pub fn parse_function(source: &str, fn_name: &str) -> Result<MagiaGraph, Error> {
    let file: File = syn::parse_str(source)?;
    let item_fn = find_function(&file, fn_name)?;
    let mut allocator = SigilIdAllocator::new();
    let main_ring = build_main_ring(item_fn, &mut allocator);
    // Phase 1.2 ではファイル/プロジェクト解析が無いため、`Module.name` と
    // `ProjectMetadata.name` の両方に関数名をプレースホルダーとして入れる。
    // ファイル粒度の解析 (Phase 1.5 以降) で実ファイルパス・プロジェクト名で上書きする。
    let module = Module {
        id: ModuleId(0),
        name: fn_name.to_string(),
        sigils: vec![main_ring],
        edges: Vec::new(),
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

fn build_main_ring(item_fn: &ItemFn, allocator: &mut SigilIdAllocator) -> Sigil {
    let fn_is_unsafe = item_fn.sig.unsafety.is_some();
    let is_async = item_fn.sig.asyncness.is_some();

    let content: Vec<_> = item_fn
        .block
        .stmts
        .iter()
        .map(|stmt| statement_to_operation(stmt, fn_is_unsafe))
        .collect();

    let await_points = count_await_points(item_fn);
    let type_info = extract_type_info(&item_fn.sig);

    let layers = LayerData {
        type_info: Some(type_info),
        concurrency: Some(ConcurrencyInfo {
            is_async,
            await_points,
        }),
        ..LayerData::default()
    };

    let span = item_fn.span();
    let start_line = u32::try_from(span.start().line).unwrap_or(0);
    let end_line = item_fn
        .block
        .stmts
        .last()
        .map_or(start_line, |stmt| statement_line_range(stmt).1);

    Sigil {
        id: allocator.allocate(),
        kind: SigilKind::MainRing,
        content,
        layers,
        source_location: SourceSpan {
            file: String::new(),
            start_line,
            end_line,
            start_column: None,
            end_column: None,
        },
        cardinality: Cardinality::default(),
    }
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
    use magia_core::ir::OperationKind;

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
}
