//! 関数シグネチャを `TypeInfo` に変換する。
//!
//! Phase 1.2 のスコープ: 引数型と戻り値型を文字列として書き起こす + Result/Option 判定。
//! ジェネリクス・lifetime・dyn Trait の意味的解決は spec §10.3 後段で Phase 1 除外項目。

use magia_core::ir::TypeInfo;
use quote::ToTokens;
use syn::{ReturnType, Signature, Type};

/// 関数シグネチャから [`TypeInfo`] を構築する。
pub(crate) fn extract_type_info(sig: &Signature) -> TypeInfo {
    let signature = render_signature(sig);
    let (returns_result, returns_option) = classify_return_type(&sig.output);
    let args = sig
        .inputs
        .iter()
        .filter_map(|input| match input {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(typed) => Some((
                compact_tokens(&typed.pat.to_token_stream().to_string()),
                compact_tokens(&typed.ty.to_token_stream().to_string()),
            )),
        })
        .collect();
    let ret = match &sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => {
            let text = compact_tokens(&ty.to_token_stream().to_string());
            // 明示的な `-> ()` も省略表記なしと同じ扱い (表示で `-> ()` を出さない)。
            (text != "()").then_some(text)
        }
    };
    TypeInfo {
        signature: Some(signature),
        fn_name: Some(sig.ident.to_string()),
        args,
        ret,
        returns_result,
        returns_option,
        reducer_shape: is_reducer_shape(sig),
    }
}

/// トークン列文字列の表示用コンパクト化 (`Vec < Neighbor >` → `Vec<Neighbor>`)。
/// 引数表示 (チップ・メイン円) と FunctionEntry.args が同じ整形を共有する。
pub(crate) fn compact_tokens(tokens: &str) -> String {
    tokens
        .replace(" :: ", "::")
        .replace(" < ", "<")
        .replace(" >", ">")
        .replace("& ", "&")
        .replace(" ,", ",")
        .replace("' ", "'")
}

/// Reducer 形 `(A, B, ...) -> A` の構文的判定 (Phase 3.5, spec §14.3)。
///
/// 第1引数の型と戻り値型のトークン列が一致するか見るだけ (型別名・参照の剥がしは
/// しない)。引数が2個未満なら Reducer とは呼ばない (単項の恒等変換を除外)。
fn is_reducer_shape(sig: &Signature) -> bool {
    if sig.inputs.len() < 2 {
        return false;
    }
    let Some(syn::FnArg::Typed(first)) = sig.inputs.first() else {
        return false; // self レシーバ始まりはメソッド畳み込みの将来課題に残す
    };
    let ReturnType::Type(_, output) = &sig.output else {
        return false;
    };
    first.ty.to_token_stream().to_string() == output.to_token_stream().to_string()
}

/// 元コードに近い形でシグネチャを書き起こす (装飾的に使う想定)。
///
/// 出力は `quote::ToTokens` の `to_string()` で得る非整形トークン列のため、
/// `fn add (a : i32 , b : i32) -> i32` のような体裁になる。レンダラ (Phase 1.6 / M6)
/// で外周ラベルに使う際に rustfmt 風の整形を当て直す責務はそちらに置く。
fn render_signature(sig: &Signature) -> String {
    sig.to_token_stream().to_string()
}

/// 戻り値型が `Result` / `Option` のいずれかを名前ベースで判定する。
///
/// 構文上の最後のパスセグメント名を見るだけの構文的判定。意味解決は行わない
/// (例: `type MyResult = Result<T, E>;` のような別名は検出できない)。
fn classify_return_type(output: &ReturnType) -> (bool, bool) {
    let ReturnType::Type(_, ty) = output else {
        return (false, false);
    };
    let Type::Path(path) = ty.as_ref() else {
        return (false, false);
    };
    let Some(last) = path.path.segments.last() else {
        return (false, false);
    };
    let name = last.ident.to_string();
    (name == "Result", name == "Option")
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn detects_result_return() {
        let item: syn::ItemFn = parse_quote! { fn f() -> Result<(), Error> { todo!() } };
        let info = extract_type_info(&item.sig);
        assert!(info.returns_result);
        assert!(!info.returns_option);
    }

    #[test]
    fn detects_option_return() {
        let item: syn::ItemFn = parse_quote! { fn f() -> Option<i32> { None } };
        let info = extract_type_info(&item.sig);
        assert!(!info.returns_result);
        assert!(info.returns_option);
    }

    #[test]
    fn extracts_structured_parts_for_display_assembly() {
        // 引数表示オプション (細部修正 2026-06-12) の部品: name / args / ret。
        let item: syn::ItemFn =
            parse_quote! { fn cast(power: u32, target: &Wand) -> Result<(), Error> { todo!() } };
        let info = extract_type_info(&item.sig);
        assert_eq!(info.fn_name.as_deref(), Some("cast"));
        assert_eq!(
            info.args,
            [
                ("power".to_string(), "u32".to_string()),
                ("target".to_string(), "&Wand".to_string()),
            ]
        );
        assert_eq!(info.ret.as_deref(), Some("Result<(), Error>"));
        // unit 戻り値は None (表示で `-> ()` を出さない)。明示的 `-> ()` も同じ。
        let unit: syn::ItemFn = parse_quote! { fn ping() {} };
        assert_eq!(extract_type_info(&unit.sig).ret, None);
        let explicit_unit: syn::ItemFn = parse_quote! { fn pong() -> () {} };
        assert_eq!(extract_type_info(&explicit_unit.sig).ret, None);
    }

    #[test]
    fn captures_signature_string() {
        let item: syn::ItemFn = parse_quote! { fn add(a: i32, b: i32) -> i32 { a + b } };
        let info = extract_type_info(&item.sig);
        let sig = info.signature.expect("signature recorded");
        assert!(sig.contains("add"));
        assert!(sig.contains("i32"));
    }

    #[test]
    fn unit_return_is_neither_result_nor_option() {
        let item: syn::ItemFn = parse_quote! { fn f() { } };
        let info = extract_type_info(&item.sig);
        assert!(!info.returns_result);
        assert!(!info.returns_option);
    }

    #[test]
    fn reducer_shape_requires_first_param_matching_return() {
        let fold: syn::ItemFn =
            parse_quote! { fn fold(acc: u32, item: u8) -> u32 { acc + u32::from(item) } };
        assert!(extract_type_info(&fold.sig).reducer_shape);
        // 戻り値型が第1引数と異なれば Reducer ではない。
        let map: syn::ItemFn =
            parse_quote! { fn map(a: u32, b: u8) -> String { format!("{a}{b}") } };
        assert!(!extract_type_info(&map.sig).reducer_shape);
        // 単項は恒等変換でも Reducer と呼ばない。
        let id: syn::ItemFn = parse_quote! { fn id(a: u32) -> u32 { a } };
        assert!(!extract_type_info(&id.sig).reducer_shape);
        // 複合型でもトークン一致で判定できる。
        let extend: syn::ItemFn =
            parse_quote! { fn extend(acc: Vec<String>, item: String) -> Vec<String> { acc } };
        assert!(extract_type_info(&extend.sig).reducer_shape);
    }
}
