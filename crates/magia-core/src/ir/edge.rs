//! Sigil 間・記号間の接続を表す Edge。

use serde::{Deserialize, Serialize};

use super::sigil::SigilId;

/// 線 (リング間・記号間の接続)。
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Edge {
    pub source: SigilId,
    pub target: SigilId,
    pub kind: EdgeKind,
    /// 接点の太さ・情報量。
    pub cardinality: f64,
    /// 多次元レイヤー情報。
    pub layers: EdgeLayerData,
}

/// Edge の種別 (spec §4.2)。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    #[default]
    ControlFlow,
    DataFlow,
    Dependency,
    Inheritance,
    Implementation,
}

/// Edge に対する多次元レイヤー情報。
///
/// Phase 1 では各フィールドが空コレクションまたは `None` のままで構わない。
/// 立体ビュー (Phase 6) で Z 軸方向の積層に使う。
///
/// `data_volume` が `f64` のため `Eq` は派生しない (`Option<f64>` が `Eq` ではない)。
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EdgeLayerData {
    /// 動的解析でトレースされた呼び出し頻度 (Phase 5+)。
    pub call_frequency: Option<u64>,
    /// データフロー量の指標 (Phase 3+)。
    pub data_volume: Option<f64>,
    /// 任意のラベル列 (拡張用)。
    pub labels: Vec<String>,
}
