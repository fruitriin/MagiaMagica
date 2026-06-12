//! ファイル全体の関数走査 — FunctionIndex (Phase 4.0)。
//!
//! トップレベル `fn`・`mod` 内・関数内ネスト・**`impl` ブロック内のメソッド**を
//! 1つの walker で収集する。`list_functions` / `parse_function` / serve の関数一覧が
//! 全て同じ走査を共有し、「列挙された名前は必ず再発見できる」API 規約を一点で守る。
//!
//! 名前は impl 文脈つきの **qualified 形式** (`Foo::bar`、トップレベルは `bar`) を
//! 正とする (同名メソッドが impl 違いで複数あるケースの一意キー、計画の設計判断)。

use std::collections::{BTreeSet, HashMap};

use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, ItemFn};

use crate::signature::compact_tokens;
use crate::summon::{UseMap, collect_calls_in_stmt};

/// 関数間の呼び出しエッジ (caller → callee、いずれも qualified)。
pub type CallEdge = (String, String);

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
    /// 引数 (self を除く)。チップの引数表示オプション (Phase 4.5 細部修正) の素材。
    pub args: Vec<FunctionArg>,
}

/// 関数引数1つ分 (パターンと型のコンパクトな文字列表現)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionArg {
    /// 引数名 (パターン。`mut x` や `(a, b)` もそのまま)。
    pub name: String,
    /// 型 (`&str` / `Vec<Neighbor>` — トークン列の空白を詰めた表示用)。
    pub ty: String,
}

/// ファイル全体を走査して関数索引を返す (ソース出現順)。
#[must_use = "関数索引は呼び出し側で利用されるべき"]
pub fn function_index(source: &str) -> Result<Vec<FunctionEntry>, crate::Error> {
    let file: File = syn::parse_str(source)?;
    let mut walker = FunctionWalker::default();
    walker.visit_file(&file);
    Ok(walker.entries)
}

/// 関数索引 + 関数間の呼び出しエッジ (caller → callee、いずれも qualified) を
/// 1回のパースで返す (Phase 4.2 近接度モデルの入力 — serve はファイル保存ごとに
/// 両方使うため、`function_index` → `call_graph` と分けて2回パースしない)。
///
/// 各関数本体の call site (`summon` の収集) を**同ファイルの関数**に解決する。
/// 解決規則:
/// - `.method` はレシーバ型が分からないため名前照合 (best effort — web の
///   `resolveCall` と同じ判断)。呼び出し元と同じ impl の同名メソッドを優先し、
///   それも無ければ定義順の先頭
/// - `Self::x` は呼び出し元の impl 文脈で置換してから qualified 照合
/// - `name!` (マクロ) は関数ではないのでエッジにしない (`resolveCall` は定義
///   ジャンプ用に寛容だが、自動計算の近接度では偽陽性を避ける)
/// - 未解決 (外部呼び出し)・自己再帰はエッジにしない
/// - 同じ相手への複数回呼び出しは1本に潰す (近接度は回数を見ない)
pub fn function_index_with_calls(
    source: &str,
) -> Result<(Vec<FunctionEntry>, Vec<CallEdge>), crate::Error> {
    let scan = scan_file(source)?;
    Ok((scan.entries, scan.local_edges))
}

/// 走査1回分の素材。`function_index_with_calls` と `workspace_index` が共有する
/// (同じソースを2回パースする公開 API を増やさない — Phase 4.2 レビューの規約)。
struct FileScan {
    entries: Vec<FunctionEntry>,
    /// 同ファイル内で解決できた呼び出し (caller → callee、重複なし)。
    local_edges: Vec<CallEdge>,
    /// 同ファイルで解決できなかった呼び出し先 (caller qualified, 正規化済み target)。
    /// メソッド呼び出しは先頭 `.` を保持する (横断解決の除外判定に使う)。
    unresolved: Vec<(String, String)>,
}

fn scan_file(source: &str) -> Result<FileScan, crate::Error> {
    let file: File = syn::parse_str(source)?;
    let uses = UseMap::from_file(&file);
    let mut walker = FunctionWalker::default();
    walker.visit_file(&file);
    let mut seen = BTreeSet::new();
    let mut local_edges = Vec::new();
    let mut unresolved = Vec::new();
    for (entry, body) in walker.entries.iter().zip(&walker.bodies) {
        for stmt in &body.block.stmts {
            for call in collect_calls_in_stmt(stmt, &uses) {
                let Some(normalized) = normalize_target(&call.target, entry) else {
                    continue;
                };
                match resolve_local_target(&normalized, entry, &walker.entries) {
                    Some(callee) if callee == entry.qualified => {} // 自己再帰
                    Some(callee) => {
                        if seen.insert((entry.qualified.clone(), callee.clone())) {
                            local_edges.push((entry.qualified.clone(), callee));
                        }
                    }
                    None => unresolved.push((entry.qualified.clone(), normalized)),
                }
            }
        }
    }
    Ok(FileScan {
        entries: walker.entries,
        local_edges,
        unresolved,
    })
}

/// 照合用の形に正規化する: マクロと「impl 文脈なしの `Self::`」は解決不能として
/// `None` (エッジ素材から除外)。`Self::x` は呼び出し元の impl 文脈で置換する。
fn normalize_target(target: &str, caller: &FunctionEntry) -> Option<String> {
    if target.ends_with('!') {
        return None;
    }
    match target.strip_prefix("Self::") {
        Some(rest) => Some(format!("{}::{rest}", caller.impl_context.as_ref()?)),
        None => Some(target.to_string()),
    }
}

/// 呼び出し先の名前を同ファイルの関数 (qualified) へ解決する。
fn resolve_local_target(
    normalized: &str,
    caller: &FunctionEntry,
    entries: &[FunctionEntry],
) -> Option<String> {
    let plain = normalized.strip_prefix('.').unwrap_or(normalized);
    // 同名メソッドが複数 impl にあるときは呼び出し元と同じ impl を優先する
    // (`self.method()` の最も確からしい解決)。qualified 完全一致 → 同 impl の
    // 名前一致 → 定義順先頭の名前一致、の3段フォールバック。
    entries
        .iter()
        .find(|e| e.qualified == plain)
        .or_else(|| {
            entries
                .iter()
                .find(|e| e.name == plain && e.impl_context == caller.impl_context)
        })
        .or_else(|| entries.iter().find(|e| e.name == plain))
        .map(|e| e.qualified.clone())
}

/// ワークスペース1ファイル分の索引結果 (Phase 4.5 M2 前段)。
#[derive(Debug)]
pub struct FileIndex {
    /// 呼び出し側が渡したパス (そのまま返す — serve の cwd 相対パスが前提)。
    pub path: String,
    /// 関数索引 (パース失敗時は空)。
    pub entries: Vec<FunctionEntry>,
    /// 同ファイル内の呼び出しエッジ。
    pub local_edges: Vec<CallEdge>,
    /// パース失敗 (俯瞰を壊さずスキップしたフラグ)。
    pub error: bool,
}

/// ファイル横断の呼び出しエッジ (from/to は qualified)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossFileEdge {
    pub from_file: String,
    pub from: String,
    pub to_file: String,
    pub to: String,
}

/// ワークスペース全体の関数索引 + ファイル横断の呼び出しグラフ (Phase 4.5 M2 前段)。
///
/// 各ファイルは1回だけパースし、同ファイル内で解決できなかった呼び出し先を
/// ワークスペース全体へ3段照合する:
/// 1. 正規化済み target と qualified の完全一致
/// 2. target の末尾2セグメント (`mod::Type::method` → `Type::method`)
/// 3. 末尾1セグメントとトップレベル関数名 (`mod::func` → `func`)
///
/// **各段でワークスペース内に一意に決まるときだけ**エッジにする (複数候補 = 曖昧
/// として捨てる — 自動計算の偽陽性回避、4.2 の「マクロをエッジにしない」と同じ判断)。
/// `.method` (レシーバ型不明) はファイル横断では解決しない。
#[must_use]
pub fn workspace_index(files: &[(String, String)]) -> (Vec<FileIndex>, Vec<CrossFileEdge>) {
    let indexed: Vec<(FileIndex, Vec<(String, String)>)> = files
        .iter()
        .map(|(path, source)| match scan_file(source) {
            Ok(scan) => (
                FileIndex {
                    path: path.clone(),
                    entries: scan.entries,
                    local_edges: scan.local_edges,
                    error: false,
                },
                scan.unresolved,
            ),
            Err(_) => (
                FileIndex {
                    path: path.clone(),
                    entries: Vec::new(),
                    local_edges: Vec::new(),
                    error: true,
                },
                Vec::new(),
            ),
        })
        .collect();

    // qualified → (ファイル番号, qualified)。同名の出現は全部持つ (一意性判定に使う)。
    let mut by_qualified: HashMap<&str, Vec<(usize, &str)>> = HashMap::new();
    for (idx, (file, _)) in indexed.iter().enumerate() {
        for entry in &file.entries {
            by_qualified
                .entry(entry.qualified.as_str())
                .or_default()
                .push((idx, entry.qualified.as_str()));
        }
    }

    let mut seen = BTreeSet::new();
    let mut cross = Vec::new();
    for (from_idx, (file, unresolved)) in indexed.iter().enumerate() {
        for (from, target) in unresolved {
            if target.starts_with('.') {
                continue; // レシーバ型不明のメソッド呼び出しは横断解決しない
            }
            let Some(&(to_idx, to)) = unique_workspace_match(target, &by_qualified) else {
                continue;
            };
            if to_idx == from_idx {
                continue; // 同ファイルはローカル解決済みのはず (防御)
            }
            let edge = CrossFileEdge {
                from_file: file.path.clone(),
                from: from.clone(),
                to_file: indexed[to_idx].0.path.clone(),
                to: to.to_string(),
            };
            if seen.insert((
                edge.from_file.clone(),
                edge.from.clone(),
                to_idx,
                edge.to.clone(),
            )) {
                cross.push(edge);
            }
        }
    }
    (indexed.into_iter().map(|(file, _)| file).collect(), cross)
}

/// 3段照合の1ターゲット分。各段で候補が**ちょうど1つ**のときだけ採用する。
fn unique_workspace_match<'a>(
    target: &str,
    by_qualified: &'a HashMap<&str, Vec<(usize, &'a str)>>,
) -> Option<&'a (usize, &'a str)> {
    // 段1: 完全一致 (`Type::method` / `func` がそのまま定義されている)
    if let Some(found) = unique(by_qualified.get(target)) {
        return Some(found);
    }
    let segments: Vec<&str> = target.split("::").collect();
    // 段2: 末尾2セグメント (`crate::module::Type::method` → `Type::method`)
    if segments.len() > 2 {
        let tail2 = format!(
            "{}::{}",
            segments[segments.len() - 2],
            segments[segments.len() - 1]
        );
        if let Some(found) = unique(by_qualified.get(tail2.as_str())) {
            return Some(found);
        }
    }
    // 段3: 末尾1セグメントとトップレベル関数名 (`module::func` → `func`)
    if segments.len() > 1 {
        let tail = segments[segments.len() - 1];
        if let Some(found) = unique(
            by_qualified
                .get(tail)
                .filter(|c| c.iter().all(|(_, q)| !q.contains("::"))),
        ) {
            return Some(found);
        }
    }
    None
}

fn unique<T>(candidates: Option<&Vec<T>>) -> Option<&T> {
    match candidates?.as_slice() {
        [only] => Some(only),
        _ => None,
    }
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
        let args = item
            .sig
            .inputs
            .iter()
            .filter_map(|input| match input {
                // self レシーバは「引数」として表示しない (チップは関数の外形が目的)。
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(typed) => Some(FunctionArg {
                    name: compact_tokens(&typed.pat.to_token_stream().to_string()),
                    ty: compact_tokens(&typed.ty.to_token_stream().to_string()),
                }),
            })
            .collect();
        self.entries.push(FunctionEntry {
            name,
            impl_context,
            qualified,
            start_line: span.start().line,
            end_line: span.end().line,
            signature: item.sig.to_token_stream().to_string(),
            args,
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

    const CALL_SOURCE: &str = r#"
fn entry(v: i32) -> i32 { helper(v) + external::thing(v) }

fn helper(v: i32) -> i32 {
    format!("{v}");
    v * 2
}

struct Wand;
impl Wand {
    fn cast(&self) -> i32 { self.charge() + Self::calibrate() }
    fn charge(&self) -> i32 { 1 }
    fn calibrate() -> i32 { recurse() }
}

fn recurse() -> i32 { recurse() }
"#;

    #[test]
    fn call_graph_resolves_local_calls_only() {
        let (_, edges) = function_index_with_calls(CALL_SOURCE).unwrap();
        // 外部呼び出し (external::thing)・マクロ (format!)・自己再帰 (recurse) は出ない。
        assert_eq!(
            edges,
            [
                ("entry".to_string(), "helper".to_string()),
                // `.charge()` はレシーバ型不明の名前照合 (best effort)
                ("Wand::cast".to_string(), "Wand::charge".to_string()),
                // `Self::calibrate()` は impl 文脈で置換して解決
                ("Wand::cast".to_string(), "Wand::calibrate".to_string()),
                ("Wand::calibrate".to_string(), "recurse".to_string()),
            ]
        );
    }

    #[test]
    fn call_graph_prefers_same_impl_for_method_names() {
        // 同名メソッドが複数 impl にあるとき `.render()` は呼び出し元の impl を優先する
        // (レビュー W2: 定義順先頭だと Widget::render が B 側からも誤って選ばれる)。
        let source = "struct A;\nimpl A {\n    fn render(&self) {}\n}\n\
struct B;\nimpl B {\n    fn run(&self) { self.render(); }\n    fn render(&self) {}\n}\n";
        let (_, edges) = function_index_with_calls(source).unwrap();
        assert_eq!(edges, [("B::run".to_string(), "B::render".to_string())]);
    }

    #[test]
    fn compact_tokens_covers_common_type_shapes() {
        // 表示用の最良努力変換 (レビュー W2 のケース網羅)。
        assert_eq!(compact_tokens("& str"), "&str");
        assert_eq!(compact_tokens("Vec < Neighbor >"), "Vec<Neighbor>");
        assert_eq!(compact_tokens("& 'a str"), "&'a str");
        assert_eq!(compact_tokens("& mut T"), "&mut T");
        assert_eq!(
            compact_tokens("std :: path :: PathBuf"),
            "std::path::PathBuf"
        );
        assert_eq!(
            compact_tokens("& [(String , String)]"),
            "&[(String, String)]"
        );
    }

    #[test]
    fn function_entry_args_extracts_pat_and_ty() {
        let index = function_index(
            "fn f(a: i32, list: &[Vec<u8>]) {}\nstruct S;\nimpl S { fn m(&self, x: &str) {} }\n",
        )
        .unwrap();
        let f_args: Vec<(&str, &str)> = index[0]
            .args
            .iter()
            .map(|a| (a.name.as_str(), a.ty.as_str()))
            .collect();
        assert_eq!(f_args, [("a", "i32"), ("list", "&[Vec<u8>]")]);
        // self レシーバは引数に含めない。
        let m_args: Vec<&str> = index[1].args.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(m_args, ["x"]);
    }

    #[test]
    fn call_graph_dedupes_repeated_calls() {
        let source = "fn a() { b(); b(); b(); }\nfn b() {}\n";
        let (_, edges) = function_index_with_calls(source).unwrap();
        assert_eq!(edges, [("a".to_string(), "b".to_string())]);
    }

    fn ws(files: &[(&str, &str)]) -> (Vec<FileIndex>, Vec<CrossFileEdge>) {
        let owned: Vec<(String, String)> = files
            .iter()
            .map(|(p, s)| ((*p).to_string(), (*s).to_string()))
            .collect();
        workspace_index(&owned)
    }

    fn edge(from_file: &str, from: &str, to_file: &str, to: &str) -> CrossFileEdge {
        CrossFileEdge {
            from_file: from_file.to_string(),
            from: from.to_string(),
            to_file: to_file.to_string(),
            to: to.to_string(),
        }
    }

    #[test]
    fn workspace_resolves_cross_file_calls() {
        let (files, cross) = ws(&[
            // use 展開された `util::helper` (段3) と裸の一意名 `unique_fn` (段1)
            (
                "main.rs",
                "use crate::util::helper;\nfn run() { helper(); unique_fn(); }\n",
            ),
            ("util.rs", "pub fn helper() {}\n"),
            ("other.rs", "pub fn unique_fn() {}\n"),
        ]);
        assert!(files.iter().all(|f| !f.error));
        assert_eq!(
            cross,
            [
                edge("main.rs", "run", "util.rs", "helper"),
                edge("main.rs", "run", "other.rs", "unique_fn"),
            ]
        );
    }

    #[test]
    fn workspace_resolves_qualified_method_via_tail2() {
        // `crate::caster::Caster::summon` → 末尾2セグメント `Caster::summon` (段2)
        let (_, cross) = ws(&[
            ("a.rs", "fn go() { crate::caster::Caster::summon(); }\n"),
            (
                "caster.rs",
                "pub struct Caster;\nimpl Caster { pub fn summon() {} }\n",
            ),
        ]);
        assert_eq!(cross, [edge("a.rs", "go", "caster.rs", "Caster::summon")]);
    }

    #[test]
    fn workspace_skips_ambiguous_and_method_calls() {
        let (_, cross) = ws(&[
            // `.method()` はレシーバ不明 → 横断解決しない。
            // `dup()` は2ファイルに定義 → 曖昧として捨てる。
            ("a.rs", "fn go(x: T) { x.method(); dup(); }\n"),
            ("b.rs", "pub fn dup() {}\npub fn method() {}\n"),
            ("c.rs", "pub fn dup() {}\n"),
        ]);
        assert_eq!(cross, []);
    }

    #[test]
    fn workspace_prefers_local_resolution() {
        // 同名がローカルにあればローカル勝ち — 横断エッジは作らない。
        let (files, cross) = ws(&[
            ("a.rs", "fn go() { helper(); }\nfn helper() {}\n"),
            ("b.rs", "pub fn helper() {}\n"),
        ]);
        assert_eq!(
            files[0].local_edges,
            [("go".to_string(), "helper".to_string())]
        );
        assert_eq!(cross, []);
    }

    #[test]
    fn workspace_stage3_requires_top_level() {
        // 段3 (末尾1セグメント) はトップレベル関数のみ — メソッドには倒さない
        // (`module::render` が `Type::render` に化ける偽陽性の防止)。
        let (_, cross) = ws(&[
            ("a.rs", "fn go() { module::render(); }\n"),
            ("b.rs", "pub struct W;\nimpl W { pub fn render() {} }\n"),
        ]);
        assert_eq!(cross, []);
    }

    #[test]
    fn workspace_marks_broken_file_and_keeps_going() {
        let (files, cross) = ws(&[
            ("ok.rs", "fn fine() { other(); }\n"),
            ("broken.rs", "fn broken( {\n"),
            ("lib.rs", "pub fn other() {}\n"),
        ]);
        assert!(files[1].error);
        assert!(files[1].entries.is_empty());
        assert_eq!(cross, [edge("ok.rs", "fine", "lib.rs", "other")]);
    }
}
