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

/// リングどうしの角度配分に使う最小隙間 (Phase 1.8)。
/// 兄弟 AuxRing の必要角度幅 `2*asin((半径+この値)/軌道距離)` に効く。
pub const RING_MARGIN: f64 = 8.0;

/// SummonGlyph どうしの最小隙間 (Phase 1.8 の衝突回避)。
pub const GLYPH_MARGIN: f64 = 6.0;

/// シグネチャ円弧ラベルの帯の半幅 (Phase 1.8 の衝突判定用)。
/// この帯に他要素が半径方向で食い込む場合のみ円弧を外側へ広げる。
pub const SIGNATURE_BAND_HALF: f64 = 10.0;

/// キャンバス bounding box に足すマージン。
pub const CANVAS_MARGIN: f64 = 24.0;

/// 交差最小化 hill-climbing の最大パス数 (spec §6.1.4 の決定論要件のため固定)。
pub const CROSSING_OPT_MAX_PASSES: usize = 50;

/// 交差最小化でファン全体を回転させる角度ステップ (ラジアン)。
pub const CROSSING_OPT_ROTATION_STEP_RAD: f64 = 0.2;

// ===== 描画系 (M6 レンダラと共有する視覚定数。重複定義を避けるためここに置く) =====

/// MainRing の線幅。
pub const MAIN_RING_STROKE: f64 = 2.0;

/// AuxRing の線幅。
pub const AUX_RING_STROKE: f64 = 1.5;

/// 接続線 (Edge) の線幅。
pub const EDGE_STROKE: f64 = 1.0;

/// async fn の二重線: 内側リングを外側からどれだけ内に寄せるか。
pub const ASYNC_INNER_RING_OFFSET: f64 = 5.0;

/// リング内の Operation ドットの半径。
pub const OPERATION_DOT_RADIUS: f64 = 3.5;

/// リング外周から Operation ドット列までの内側オフセット。
pub const OPERATION_DOT_INSET: f64 = 16.0;

/// シグネチャ円弧ラベルのリング外周からのオフセット。
pub const SIGNATURE_ARC_OFFSET: f64 = 12.0;

/// Result/Option 戻り値の分岐線の長さ。
pub const RETURN_BRANCH_LENGTH: f64 = 26.0;
