//! レイアウト定数 (Phase 1.5)。
//!
//! M6 (SVG レンダラ) がリングの描画半径として同じ値を参照するため `pub` で公開する。
//! 値は Phase 1 の仮置き。実レンダリングの目視 (Phase 1.6) で調整する。

/// MainRing の半径。
pub const MAIN_RING_RADIUS: f64 = 120.0;

/// AuxRing の半径 (入れ子でも同一。縮小は視覚検証後に判断)。
pub const AUX_RING_RADIUS: f64 = 48.0;

/// リング外周どうしの隙間 (親リング外周 → 子リング外周)。
pub const RING_GAP: f64 = 36.0;

/// SummonGlyph (召喚記号) の半径。
pub const SUMMON_GLYPH_RADIUS: f64 = 14.0;

/// リング外周から SummonGlyph 外周までの隙間。
pub const GLYPH_GAP: f64 = 28.0;

/// 同一 anchor に複数の AuxRing が並ぶとき (if 連鎖・match アーム) の
/// 扇状の角度ステップ (ラジアン)。
pub const SIBLING_FAN_STEP_RAD: f64 = 0.35;

/// キャンバス bounding box に足すマージン。
pub const CANVAS_MARGIN: f64 = 24.0;

/// 交差最小化 hill-climbing の最大パス数 (spec §6.1.4 の決定論要件のため固定)。
pub const CROSSING_OPT_MAX_PASSES: usize = 50;

/// 交差最小化でファン全体を回転させる角度ステップ (ラジアン)。
pub const CROSSING_OPT_ROTATION_STEP_RAD: f64 = 0.2;
