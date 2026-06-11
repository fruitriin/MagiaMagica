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
/// Phase 3.4 で Sigil 側の `LayerData` と同じ `Option<XxxInfo>` 積層構造へ破壊的に
/// 再設計した (spec v0.2 §4.3 の既定方針、v0.3 追認)。旧 `data_volume: Option<f64>` は
/// `data_flow.variables` (流れる変数の列挙) として正式化 — 量だけでなく
/// 「何が流れているか」を保持し、誤検出をデバッグ可能にする (説明可能性の優先)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EdgeLayerData {
    /// データフロー線の詳細 (Phase 3.4 で値を埋める)。
    pub data_flow: Option<EdgeDataFlowInfo>,
    /// 動的解析でトレースされたプロファイル (Phase 5+)。
    pub profile: Option<EdgeProfileInfo>,
    /// 任意のラベル列 (拡張用)。
    pub labels: Vec<String>,
}

/// DataFlow Edge の詳細 (spec §5.1 `data_flow` レイヤーの Edge 側)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EdgeDataFlowInfo {
    /// この線を流れる変数名 (辞書順)。線の太さ = `variables.len()`。
    pub variables: Vec<String>,
}

/// Edge の動的プロファイル (Phase 5+)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EdgeProfileInfo {
    /// トレースされた呼び出し頻度。
    pub call_frequency: u64,
}
