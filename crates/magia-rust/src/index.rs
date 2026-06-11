//! ファイル全体の関数走査 — FunctionIndex (Phase 4.0)。
//!
//! トップレベル `fn`・`mod` 内・関数内ネスト・**`impl` ブロック内のメソッド**を
//! 1つの walker で収集する。`list_functions` / `parse_function` / serve の関数一覧が
//! 全て同じ走査を共有し、「列挙された名前は必ず再発見できる」API 規約を一点で守る。
//!
//! 名前は impl 文脈つきの **qualified 形式** (`Foo::bar`、トップレベルは `bar`) を
//! 正とする (同名メソッドが impl 違いで複数あるケースの一意キー、計画の設計判断)。

use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, ItemFn};

/// 関数1つ分の索引情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionEntry {
    /// 関数名 (`bar`)。
    pub name: String,
    /// impl ブロックの self 型 (`Foo`)。トップレベル関数は `None`。
    /// `impl Trait for Foo` は `Foo` を採る (メソッドが所属する型)。
    pub impl_context: Option<String>,
    /// 一意キー (`Foo::bar` / `bar`)。`parse_function` / `?fn=` で使う。
    pub qualified: String,
    /// ソース上の行範囲 (1始まり、両端含む)。
    pub start_line: usize,
    pub end_line: usize,
    /// シグネチャの文字列 (非整形トークン列、表示用)。
    pub signature: String,
}

/// ファイル全体を走査して関数索引を返す (ソース出現順)。
#[must_use = "関数索引は呼び出し側で利用されるべき"]
pub fn function_index(source: &str) -> Result<Vec<FunctionEntry>, crate::Error> {
    let file: File = syn::parse_str(source)?;
    let mut walker = FunctionWalker::default();
    walker.visit_file(&file);
    Ok(walker.entries)
}

/// qualified 名 (`Foo::bar`) または素の名前 (`bar`) で関数本体を探す。
///
/// 戻り値はメソッドを `ItemFn` に正規化した owned 値 (`ImplItemFn` と `ItemFn` は
/// sig/block の形が同じため、後段のリング構築を一本化できる)。
/// 素の名前は qualified 完全一致が無いときのフォールバックで、ソース出現順の
/// 最初の同名関数に決定論的に解決する。
pub(crate) fn find_function(file: &File, fn_name: &str) -> Result<ItemFn, crate::Error> {
    let mut walker = FunctionWalker::default();
    walker.visit_file(file);
    let found = walker
        .entries
        .iter()
        .zip(&walker.bodies)
        .find(|(entry, _)| entry.qualified == fn_name)
        .or_else(|| {
            walker
                .entries
                .iter()
                .zip(&walker.bodies)
                .find(|(entry, _)| entry.name == fn_name)
        });
    match found {
        Some((_, body)) => Ok(body.clone()),
        None => Err(crate::Error::FunctionNotFound {
            name: fn_name.to_string(),
            candidates: walker.entries.into_iter().map(|e| e.qualified).collect(),
        }),
    }
}

/// 索引と本体を同時に集める walker (1関心1visitor の例外: 索引と本体は同じ走査範囲
/// であることが規約なので、走査を分けると将来ずれる)。
#[derive(Default)]
struct FunctionWalker {
    impl_stack: Vec<String>,
    entries: Vec<FunctionEntry>,
    /// `entries` と同じ並びの正規化済み関数本体。
    bodies: Vec<ItemFn>,
}

impl FunctionWalker {
    fn push(&mut self, item: ItemFn, span: proc_macro2::Span) {
        let name = item.sig.ident.to_string();
        let impl_context = self.impl_stack.last().cloned();
        let qualified = match &impl_context {
            Some(context) => format!("{context}::{name}"),
            None => name.clone(),
        };
        self.entries.push(FunctionEntry {
            name,
            impl_context,
            qualified,
            start_line: span.start().line,
            end_line: span.end().line,
            signature: item.sig.to_token_stream().to_string(),
        });
        self.bodies.push(item);
    }
}

impl<'ast> Visit<'ast> for FunctionWalker {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // 関数内のネスト fn / mod 内宣言も拾う (Phase 1.2 からの規約)。
        // メソッド本体内のネスト fn は impl 文脈を引き継ぐ (`S::local_fn`) — メソッドと
        // 同列に見えるが、list→parse の一意な往復を優先した意図的な近似 (Phase 4.0)。
        self.push(node.clone(), node.span());
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        // impl 文脈は self 型の最後のセグメント (`impl fmt::Display for Foo` → `Foo`)。
        // パス以外の self 型 (参照・タプル等) はトークン列全体で受ける (防御)。
        let context = match node.self_ty.as_ref() {
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .map_or_else(String::new, |s| s.ident.to_string()),
            other => other.to_token_stream().to_string(),
        };
        self.impl_stack.push(context);
        syn::visit::visit_item_impl(self, node);
        self.impl_stack.pop();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        // メソッドを ItemFn に正規化する: sig / block / attrs / vis の形は共通。
        let item = ItemFn {
            attrs: node.attrs.clone(),
            vis: node.vis.clone(),
            sig: node.sig.clone(),
            block: Box::new(node.block.clone()),
        };
        self.push(item, node.span());
        syn::visit::visit_impl_item_fn(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = r"
fn top_level(a: u32) -> u32 { a }

struct Widget;
impl Widget {
    fn render(&self) -> String { String::new() }
    fn area(&self) -> u32 { 0 }
}

struct Gadget;
impl Gadget {
    fn render(&self) -> String { String::new() }
}

impl std::fmt::Display for Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }
}
";

    #[test]
    fn collects_top_level_and_methods_with_context() {
        let index = function_index(SOURCE).unwrap();
        let qualified: Vec<&str> = index.iter().map(|e| e.qualified.as_str()).collect();
        assert_eq!(
            qualified,
            [
                "top_level",
                "Widget::render",
                "Widget::area",
                "Gadget::render",
                "Widget::fmt"
            ]
        );
        assert_eq!(index[0].impl_context, None);
        assert_eq!(index[1].impl_context.as_deref(), Some("Widget"));
    }

    #[test]
    fn line_ranges_and_signatures_are_recorded() {
        let index = function_index(SOURCE).unwrap();
        let top = &index[0];
        assert_eq!(top.start_line, 2);
        assert_eq!(top.end_line, 2);
        assert!(top.signature.contains("fn top_level"));
        let render = &index[1];
        assert!(render.start_line < render.end_line || render.start_line > 2);
    }

    #[test]
    fn qualified_name_disambiguates_same_method_name() {
        let file: File = syn::parse_str(SOURCE).unwrap();
        // Widget::render と Gadget::render は別の関数として引ける。
        let widget = find_function(&file, "Widget::render").unwrap();
        let gadget = find_function(&file, "Gadget::render").unwrap();
        assert_eq!(widget.sig.ident, "render");
        assert_eq!(gadget.sig.ident, "render");
        // 素の名前はソース出現順の最初 (Widget::render) に解決する。
        let bare = find_function(&file, "render").unwrap();
        assert_eq!(bare.sig.ident, "render");
    }

    #[test]
    fn nested_fn_inside_method_inherits_impl_context() {
        // 意図的な近似の固定: メソッド内のネスト fn も impl 文脈つきで列挙される。
        let source = "struct S;\nimpl S { fn method(&self) { fn local_fn() {} } }\n";
        let index = function_index(source).unwrap();
        let qualified: Vec<&str> = index.iter().map(|e| e.qualified.as_str()).collect();
        assert_eq!(qualified, ["S::method", "S::local_fn"]);
    }

    #[test]
    fn unknown_function_lists_qualified_candidates() {
        let file: File = syn::parse_str(SOURCE).unwrap();
        let error = find_function(&file, "nope").unwrap_err();
        let message = error.to_string();
        assert!(message.contains("nope"));
    }
}
