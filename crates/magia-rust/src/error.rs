//! `magia-rust` のエラー型。

use thiserror::Error;

/// Rust ソースを IR に変換するときに発生するエラー。
#[derive(Debug, Error)]
pub enum Error {
    /// 入力ソースが Rust として構文解析できなかった。
    #[error("Rust 構文エラー: {0}")]
    Syntax(#[from] syn::Error),

    /// 指定された名前の関数がソースに見つからなかった。
    #[error(
        "関数 `{name}` が見つかりません (候補: {})",
        if candidates.is_empty() { "なし".to_string() } else { candidates.join(", ") }
    )]
    FunctionNotFound {
        name: String,
        candidates: Vec<String>,
    },
}
