//! 関数本体の `syn::Stmt` / `syn::Expr` を `Operation` に変換する。
//!
//! Phase 1.3 以降、制御構造 (`if` / `match` / ループ) は `ring` モジュールが AuxRing へ
//! 切り出すため、本モジュールが受け取るのは「制御構造ではない」statement / 式に限られる。
//! 制御構造そのものに対応する Operation (Branch/Match/Loop) の組み立ても `ring` 側の責務。

use magia_core::ir::{EffectSet, Operation, OperationKind, OperationPayload};
use quote::ToTokens;
use syn::visit::Visit;
use syn::{Expr, ExprUnsafe, Stmt};

/// 関数解析全体で引き回すコンテキスト。
///
/// Phase 1.2 では `fn_is_unsafe: bool` を直接引数で渡していたが、AuxRing の再帰展開で
/// 呼び出し連鎖が深くなるため構造体に集約した (Phase 1.2 レビュー持ち越し対応)。
/// Phase 1.4 (call site) 以降のコンテキスト追加もここに足す。
#[derive(Debug, Clone, Copy)]
pub(crate) struct ParseContext {
    /// 関数全体が `unsafe fn` か。true なら全 Operation の `unsafe_block` を立てる。
    pub(crate) fn_is_unsafe: bool,
}

/// 単一の statement を Operation に変換する。
///
/// `OperationKind::Await` は Phase 1.2〜1.3 ではトップレベル statement の `Await` 式しか
/// 区別できないため、`await_points` カウントを `ConcurrencyInfo` 側に集約し、本関数では
/// `Compute` 扱いで通す。Operation 単位の Await/Yield/Throw への展開はレンダリング
/// (Phase 1.6) で必要になったときに判断する。
pub(crate) fn statement_to_operation(stmt: &Stmt, ctx: ParseContext) -> Operation {
    let is_return_stmt = expression_is_return(stmt);
    let mut visitor = StatementVisitor::default();
    visitor.visit_stmt(stmt);
    build_operation(
        is_return_stmt || visitor.has_try,
        visitor.has_unsafe,
        excerpt(stmt),
        ctx,
    )
}

/// 式のサブツリーを1回走査し、`?` と `unsafe` の有無を返す。
///
/// `ring` モジュールが制御構造の条件式 (if の条件、match の被検査式等) だけを
/// スキャンするために公開する。本体ブロックは AuxRing 側で別途処理されるため、
/// ここに本体を渡すと二重計上になる。
pub(crate) fn scan_expression(expr: &Expr) -> ExprScan {
    let mut visitor = StatementVisitor::default();
    visitor.visit_expr(expr);
    ExprScan {
        has_try: visitor.has_try,
        has_unsafe: visitor.has_unsafe,
    }
}

/// `scan_expression` の結果。
pub(crate) struct ExprScan {
    pub(crate) has_try: bool,
    pub(crate) has_unsafe: bool,
}

fn build_operation(
    early_return: bool,
    has_unsafe: bool,
    excerpt: String,
    ctx: ParseContext,
) -> Operation {
    let kind = if early_return {
        OperationKind::Return
    } else {
        OperationKind::Compute
    };
    let mut effects = EffectSet::default();
    if ctx.fn_is_unsafe || has_unsafe {
        effects.unsafe_block = true;
    }
    Operation {
        kind,
        effects,
        payload: OperationPayload {
            source_excerpt: Some(excerpt),
            call_target: None,
            early_return,
            // defs/uses はスコープを知る ring 側 (dataflow) が後から充填する。
            ..OperationPayload::default()
        },
    }
}

/// statement のトップレベル式が `return` か (visitor を回す前の高速判定)。
fn expression_is_return(stmt: &Stmt) -> bool {
    matches!(stmt, Stmt::Expr(Expr::Return(_), _))
}

fn excerpt(stmt: &Stmt) -> String {
    // ToTokens の `to_string()` は空白の正規化までで十分。整形は表示側 (M6) の責務。
    stmt.to_token_stream().to_string()
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

    const SAFE_FN: ParseContext = ParseContext {
        fn_is_unsafe: false,
    };
    const UNSAFE_FN: ParseContext = ParseContext { fn_is_unsafe: true };

    fn parse_stmt(src: &str) -> Stmt {
        syn::parse_str::<Stmt>(src).expect("parse stmt")
    }

    #[test]
    fn compute_for_plain_expression() {
        let stmt = parse_stmt("let x = 1 + 2;");
        let op = statement_to_operation(&stmt, SAFE_FN);
        assert_eq!(op.kind, OperationKind::Compute);
        assert!(!op.payload.early_return);
    }

    #[test]
    fn return_for_explicit_return_expression() {
        let stmt: Stmt = parse_quote! { return; };
        let op = statement_to_operation(&stmt, SAFE_FN);
        assert_eq!(op.kind, OperationKind::Return);
        assert!(op.payload.early_return);
    }

    #[test]
    fn try_operator_is_early_return() {
        let stmt: Stmt = parse_quote! { let v = f()?; };
        let op = statement_to_operation(&stmt, SAFE_FN);
        assert_eq!(op.kind, OperationKind::Return);
        assert!(op.payload.early_return);
    }

    #[test]
    fn unsafe_block_sets_effect_flag() {
        let stmt: Stmt = parse_quote! { unsafe { do_thing(); } };
        let op = statement_to_operation(&stmt, SAFE_FN);
        assert!(op.effects.unsafe_block);
    }

    #[test]
    fn unsafe_fn_marks_all_operations() {
        let stmt = parse_stmt("let x = 1;");
        let op = statement_to_operation(&stmt, UNSAFE_FN);
        assert!(op.effects.unsafe_block);
    }

    #[test]
    fn scan_expression_finds_try_and_unsafe() {
        let expr: Expr = parse_quote! { unsafe { f()? } };
        let scan = scan_expression(&expr);
        assert!(scan.has_try);
        assert!(scan.has_unsafe);
    }
}
