//! call site の抽出と近似パス解決 (Phase 1.4, spec §6.1.2 の召喚記号)。
//!
//! 解決戦略は tech-selection §2.1 の Phase 1a: 意味解決はせず、
//! 「記述されたままのパス + 同ファイル内 `use` 文の機械的展開」で近似する。
//! 解決できないものは記述されたまま保持し、効果判定は `effects` モジュールの
//! ヒューリスティックに委ねる (未知パスはサイレントに pure)。

use magia_core::ir::EffectSet;
use quote::ToTokens;
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, File, Stmt, UseTree};

use crate::effects::{effects_for_macro, effects_for_path};

/// `use` 文の機械的展開表。先頭セグメント名 → フルパス。
///
/// 例: `use std::collections::HashMap;` ⇒ `"HashMap" → "std::collections::HashMap"`。
/// モジュール境界は無視してファイル全体から収集する (Phase 1a の近似)。
/// 同名の `use` が複数あるときは後勝ち (ソース順で決定論的)。
pub(crate) struct UseMap {
    map: HashMap<String, String>,
}

impl UseMap {
    pub(crate) fn from_file(file: &File) -> Self {
        struct Collector {
            map: HashMap<String, String>,
        }
        impl Collector {
            fn walk(&mut self, prefix: &str, tree: &UseTree) {
                match tree {
                    UseTree::Path(path) => {
                        let next = if prefix.is_empty() {
                            path.ident.to_string()
                        } else {
                            format!("{prefix}::{}", path.ident)
                        };
                        self.walk(&next, &path.tree);
                    }
                    UseTree::Name(name) => {
                        let ident = name.ident.to_string();
                        let full = if prefix.is_empty() {
                            ident.clone()
                        } else {
                            format!("{prefix}::{ident}")
                        };
                        self.map.insert(ident, full);
                    }
                    UseTree::Rename(rename) => {
                        let full = if prefix.is_empty() {
                            rename.ident.to_string()
                        } else {
                            format!("{prefix}::{}", rename.ident)
                        };
                        self.map.insert(rename.rename.to_string(), full);
                    }
                    UseTree::Group(group) => {
                        for item in &group.items {
                            self.walk(prefix, item);
                        }
                    }
                    // glob は展開先が分からないため無視 (Phase 1a)。
                    UseTree::Glob(_) => {}
                }
            }
        }
        impl<'ast> Visit<'ast> for Collector {
            fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
                self.walk("", &node.tree);
                syn::visit::visit_item_use(self, node);
            }
        }
        let mut collector = Collector {
            map: HashMap::new(),
        };
        collector.visit_file(file);
        Self { map: collector.map }
    }

    /// パスの先頭セグメントを `use` 展開してフルパスを近似する。
    /// 展開できなければ記述されたままを返す。
    fn expand(&self, path: &str) -> String {
        let (first, rest) = path.split_once("::").unwrap_or((path, ""));
        match self.map.get(first) {
            Some(full) if rest.is_empty() => full.clone(),
            Some(full) => format!("{full}::{rest}"),
            None => path.to_string(),
        }
    }
}

/// 抽出された call site 1件。SummonGlyph の素材。
pub(crate) struct CallSite {
    /// 近似解決後の呼び出し先 (`std::fs::read_to_string`, `println!`,
    /// メソッド呼び出しは `.method` 形式)。
    pub(crate) target: String,
    /// 元ソースの抜粋。
    pub(crate) excerpt: String,
    /// 効果カテゴリ (ヒューリスティック判定済み)。
    pub(crate) effects: EffectSet,
    /// 呼び出し位置。
    pub(crate) span: proc_macro2::Span,
}

/// statement 全体から call site を収集する (非制御 statement 用)。
///
/// `let x = if ...` のような式内制御構造の本体に含まれる call も、その statement を
/// 持つリングに係留される (制御構造を切り出さない判断と整合)。
pub(crate) fn collect_calls_in_stmt(stmt: &Stmt, uses: &UseMap) -> Vec<CallSite> {
    let mut visitor = CallVisitor {
        uses,
        calls: Vec::new(),
    };
    visitor.visit_stmt(stmt);
    visitor.calls
}

/// 式から call site を収集する (制御構造のガード式・被検査式・イテレータ式用)。
///
/// 本体ブロックは AuxRing 側のリング構築時に収集されるため、ここに渡すと二重計上になる。
pub(crate) fn collect_calls_in_expr(expr: &Expr, uses: &UseMap) -> Vec<CallSite> {
    let mut visitor = CallVisitor {
        uses,
        calls: Vec::new(),
    };
    visitor.visit_expr(expr);
    visitor.calls
}

/// `ExprCall` / `ExprMethodCall` / マクロ呼び出しを1回の走査でまとめて拾う visitor。
///
/// 収集順は syn の走査順 (= ソース出現順) なので決定論的。
/// マクロのトークン列内部 (`println!("{}", foo())` の `foo()`) は syn が式として
/// 走査しないため拾えない (Phase 1 の既知の限界)。
/// 引数位置のネスト呼び出し (`outer(inner())`) は独立した glyph として同一の
/// 所属リングに平坦化され、「inner が outer の引数である」関係は IR 上では失われる。
struct CallVisitor<'a> {
    uses: &'a UseMap,
    calls: Vec<CallSite>,
}

impl<'ast> Visit<'ast> for CallVisitor<'_> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        // 関数位置がパス式 (`foo(..)`, `HashMap::new(..)`) のときのみ解決を試みる。
        // クロージャ呼び出し等のパスでない式は記述のまま保持する。
        let target = match node.func.as_ref() {
            Expr::Path(path) => self.uses.expand(&path_to_string(&path.path)),
            other => other.to_token_stream().to_string(),
        };
        self.calls.push(CallSite {
            effects: effects_for_path(&target),
            excerpt: node.to_token_stream().to_string(),
            span: node.span(),
            target,
        });
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        // レシーバ型が分からないため解決不能 (Phase 1a)。`.method` 形式で保持し pure 扱い。
        let target = format!(".{}", node.method);
        self.calls.push(CallSite {
            effects: effects_for_path(&target),
            excerpt: node.to_token_stream().to_string(),
            span: node.span(),
            target,
        });
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        // `Stmt::Macro` (`println!("x");`) と `Expr::Macro` の両方がここを通る。
        // 名前ベースの白リスト判定のみで、トークン列は展開しない。
        let name = node
            .path
            .segments
            .last()
            .map_or_else(String::new, |seg| seg.ident.to_string());
        self.calls.push(CallSite {
            effects: effects_for_macro(&name),
            excerpt: node.to_token_stream().to_string(),
            span: node.span(),
            target: format!("{name}!"),
        });
        syn::visit::visit_macro(self, node);
    }
}

/// パスをセグメント名のみで文字列化する。ターボフィッシュの型引数
/// (`HashMap::<K, V>::new` の `::<K, V>`) は意図的に落とす (効果判定には不要)。
fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn use_map(src: &str) -> UseMap {
        UseMap::from_file(&syn::parse_str::<File>(src).expect("parse file"))
    }

    fn calls(stmt_src: &str, uses: &UseMap) -> Vec<CallSite> {
        let stmt = syn::parse_str::<Stmt>(stmt_src).expect("parse stmt");
        collect_calls_in_stmt(&stmt, uses)
    }

    #[test]
    fn use_map_expands_first_segment() {
        let uses = use_map("use std::collections::HashMap;");
        assert_eq!(
            uses.expand("HashMap::new"),
            "std::collections::HashMap::new"
        );
        assert_eq!(uses.expand("Unknown::f"), "Unknown::f");
    }

    #[test]
    fn use_map_handles_group_and_rename() {
        let uses = use_map("use std::io::{Read, Write as W};");
        assert_eq!(uses.expand("Read"), "std::io::Read");
        assert_eq!(uses.expand("W"), "std::io::Write");
    }

    #[test]
    fn use_map_rename_without_prefix_has_no_leading_colons() {
        // `use Bar as B;` (extern prelude 由来) で `::Bar` にならないこと。
        let uses = use_map("use Bar as B;");
        assert_eq!(uses.expand("B::f"), "Bar::f");
    }

    #[test]
    fn call_paths_are_collected_in_source_order() {
        let uses = use_map("");
        let sites = calls("let x = outer(inner());", &uses);
        let targets: Vec<_> = sites.iter().map(|c| c.target.as_str()).collect();
        assert_eq!(targets, vec!["outer", "inner"]);
    }

    #[test]
    fn macro_statement_is_collected() {
        let uses = use_map("");
        let sites = calls("println!(\"x\");", &uses);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].target, "println!");
        assert!(sites[0].effects.io);
    }

    #[test]
    fn method_call_is_pure_with_dot_prefix() {
        let uses = use_map("");
        let sites = calls("file.read_to_string(&mut buf);", &uses);
        assert_eq!(sites[0].target, ".read_to_string");
        assert!(sites[0].effects.pure);
    }
}
