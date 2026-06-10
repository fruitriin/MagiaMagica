//! MagiaMagica core: IR, analysis, layout, and SVG rendering.
//!
//! - [`ir`][ir]: 言語非依存の中間表現 (Phase 1.1)
//! - [`layout`][layout]: 決定論的レイアウトエンジン (Phase 1.5)
//! - [`render`][render]: SVG レンダラ — ミッドチルダ式 ConcentricRings (Phase 1.6)
//! - [`filter`][filter]: フィルター言語 `.magia` (Phase 2.3)
//! - [`transcript`][transcript]: 呪文書き起こし — アクセシビリティ (Phase 2.4)
//! - [`metrics`][metrics]: transcript / diff が共有するメトリクス集計 (Phase 3.1)
//! - [`diff`][diff]: IR 差分エンジン — Spell Diff (Phase 3.1, spec v0.3 §9.2)

pub mod diff;
pub mod filter;
pub mod ir;
pub mod layout;
pub mod metrics;
pub mod render;
pub mod transcript;
