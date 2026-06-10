//! 関数本体の `syn::Stmt` を `Operation` に変換する。
//!
//! Phase 1.2 のスコープでは関数本体の**トップレベル statement** のみを Operation 列に
//! 平坦化する。内部の `if` / `match` / `loop` を AuxRing に切り出すのは Phase 1.3 で行う。

use magia_core::ir::{EffectSet, Operation, OperationKind, OperationPayload};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, ExprUnsafe, Stmt};

/// 単一の statement を Operation に変換する。
///
/// `fn_is_unsafe` が true なら、すべての Operation の `EffectSet.unsafe_block` を立てる
/// (関数全体が unsafe コンテキストにあるため)。
///
/// `OperationKind::Await` は Phase 1.2 ではトップレベル statement の `Await` 式しか
/// 区別できないため、`await_points` カウントを `ConcurrencyInfo` 側に集約し、本関数では
/// `Compute` 扱いで通す。Operation 単位の Await/Yield/Throw への展開は Phase 1.3 以降。
pub(crate) fn statement_to_operation(stmt: &Stmt, fn_is_unsafe: bool) -> Operation {
    let scan = scan_statement(stmt);
    let mut effects = EffectSet::default();
    if fn_is_unsafe || scan.has_unsafe {
        effects.unsafe_block = true;
    }
    Operation {
        kind: scan.kind,
        effects,
        payload: OperationPayload {
            source_excerpt: Some(excerpt(stmt)),
            call_target: None,
            early_return: scan.early_return,
        },
    }
}

/// statement を1回だけ走査して必要な情報をまとめて取り出す。
///
/// 別々のヘルパーを連続で呼ぶと AST スキャンが重複し、Phase 1.3 でビジターが増えたとき
/// 計算量が線形に膨らむため、単一のヘルパーに集約する。
fn scan_statement(stmt: &Stmt) -> StatementScan {
    let is_return_stmt = expression_is_return(stmt);
    let mut visitor = StatementVisitor::default();
    visitor.visit_stmt(stmt);

    let early_return = is_return_stmt || visitor.has_try;
    let kind = if early_return {
        OperationKind::Return
    } else {
        OperationKind::Compute
    };

    StatementScan {
        kind,
        early_return,
        has_unsafe: visitor.has_unsafe,
    }
}

struct StatementScan {
    kind: OperationKind,
    early_return: bool,
    has_unsafe: bool,
}

/// statement のトップレベル式が `return` か (visitor を回す前の高速判定)。
fn expression_is_return(stmt: &Stmt) -> bool {
    matches!(stmt, Stmt::Expr(Expr::Return(_), _))
}

fn excerpt(stmt: &Stmt) -> String {
    // ToTokens の `to_string()` は空白の正規化までで十分。整形は表示側 (M6) の責務。
    stmt.to_token_stream().to_string()
}

/// statement の長さ (行数) を概算する。`SourceSpan` 充填の補助。
pub(crate) fn statement_line_range(stmt: &Stmt) -> (u32, u32) {
    let span = stmt.span();
    let start = u32::try_from(span.start().line).unwrap_or(0);
    let end = u32::try_from(span.end().line).unwrap_or(start);
    (start, end)
}

#[derive(Default)]
struct StatementVisitor {
    has_try: bool,
    has_unsafe: bool,
}

impl<'ast> Visit<'ast> for StatementVisitor {
    fn visit_expr_try(&mut self, node: &'ast syn::ExprTry) {
        self.has_try = true;
        syn::visit::visit_expr_try(self, node);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast ExprUnsafe) {
        self.has_unsafe = true;
        syn::visit::visit_expr_unsafe(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn parse_stmt(src: &str) -> Stmt {
        syn::parse_str::<Stmt>(src).expect("parse stmt")
    }

    #[test]
    fn compute_for_plain_expression() {
        let stmt = parse_stmt("let x = 1 + 2;");
        let op = statement_to_operation(&stmt, false);
        assert_eq!(op.kind, OperationKind::Compute);
        assert!(!op.payload.early_return);
    }

    #[test]
    fn return_for_explicit_return_expression() {
        let stmt: Stmt = parse_quote! { return; };
        let op = statement_to_operation(&stmt, false);
        assert_eq!(op.kind, OperationKind::Return);
        assert!(op.payload.early_return);
    }

    #[test]
    fn try_operator_is_early_return() {
        let stmt: Stmt = parse_quote! { let v = f()?; };
        let op = statement_to_operation(&stmt, false);
        assert_eq!(op.kind, OperationKind::Return);
        assert!(op.payload.early_return);
    }

    #[test]
    fn unsafe_block_sets_effect_flag() {
        let stmt: Stmt = parse_quote! { unsafe { do_thing(); } };
        let op = statement_to_operation(&stmt, false);
        assert!(op.effects.unsafe_block);
    }

    #[test]
    fn unsafe_fn_marks_all_operations() {
        let stmt = parse_stmt("let x = 1;");
        let op = statement_to_operation(&stmt, true);
        assert!(op.effects.unsafe_block);
    }
}
