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
    TypeInfo {
        signature: Some(signature),
        returns_result,
        returns_option,
    }
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
}
