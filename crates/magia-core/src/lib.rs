//! MagiaMagica core: IR, analysis, layout, and SVG rendering.
//!
//! - [`ir`][ir]: 言語非依存の中間表現 (Phase 1.1)
//! - [`layout`][layout]: 決定論的レイアウトエンジン (Phase 1.5)
//! - [`render`][render]: SVG レンダラ — ミッドチルダ式 ConcentricRings (Phase 1.6)
//! - [`filter`][filter]: フィルター言語 `.magia` (Phase 2.3)

pub mod filter;
pub mod ir;
pub mod layout;
pub mod render;
