//! 図形ノード (リング・記号) の定義。

use serde::{Deserialize, Serialize};

use super::layers::LayerData;
use super::operation::Operation;
use super::source::SourceSpan;

/// Sigil を一意に識別する ID。決定論的に採番する (乱数禁止)。
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct SigilId(pub u32);

/// Sigil の種別 (spec §6.1.2)。
///
/// - [`MainRing`](SigilKind::MainRing): 関数本体を表すリング (図の中心)
/// - [`AuxRing`](SigilKind::AuxRing): 補助リング (制御構造ブロック等)
/// - [`SummonGlyph`](SigilKind::SummonGlyph): 外部関数呼び出しの召喚記号
/// - [`GateGlyph`](SigilKind::GateGlyph): エントリポイント・公開境界の門記号
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SigilKind {
    #[default]
    MainRing,
    AuxRing,
    SummonGlyph,
    GateGlyph,
}

/// 図形ノード = リング・記号の単位。
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sigil {
    pub id: SigilId,
    pub kind: SigilKind,
    /// リング内の処理列。SummonGlyph 等では空でよい。
    pub content: Vec<Operation>,
    /// 多次元レイヤー情報。
    pub layers: LayerData,
    /// 対応するソース位置。
    pub source_location: SourceSpan,
    /// 重み・情報量 (直径・太さに反映される)。
    pub cardinality: Cardinality,
}

/// Sigil の重み・情報量。spec §13 用語集の Sigil Cardinality に対応。
///
/// レンダリング時に直径・太さ・装飾度合いへ反映する基礎値。
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Cardinality {
    /// 基本重み (0.0〜)。
    pub weight: f64,
    /// 情報密度の補助メトリクス (任意)。
    pub density: Option<f64>,
}
