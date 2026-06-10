//! MagiaMagica core: IR, analysis, layout, and SVG rendering.
//!
//! - [`ir`][ir]: 言語非依存の中間表現 (Phase 1.1)
//! - [`layout`][layout]: 決定論的レイアウトエンジン (Phase 1.5)
//!
//! SVG レンダラは Phase 1.6 で追加する。

pub mod ir;
pub mod layout;
