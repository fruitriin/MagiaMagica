//! ソース位置情報。

use serde::{Deserialize, Serialize};

/// ソースコード上の位置範囲。
///
/// Phase 1 では関数1つを描画するため、ファイルパスと行範囲を持つ。
/// 列情報は将来のホバーインタラクションのために `Option` で確保する。
/// `None` は「解析器から列情報が得られなかった」を意味し、`Some(0)` 等の
/// センチネル値とは区別する。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SourceSpan {
    /// ファイルの絶対または相対パス。
    pub file: String,
    /// 開始行 (1-based)。0 はファイル未確定 (default) を表す。
    pub start_line: u32,
    /// 終了行 (1-based, inclusive)。
    pub end_line: u32,
    /// 開始列 (1-based)。`None` は解析器から列情報が得られなかったことを示す。
    pub start_column: Option<u32>,
    /// 終了列 (1-based)。`None` は解析器から列情報が得られなかったことを示す。
    pub end_column: Option<u32>,
}
