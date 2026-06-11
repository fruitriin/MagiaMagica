//! MagiaMagica core: IR, analysis, layout, and SVG rendering.
//!
//! - [`ir`][ir]: 言語非依存の中間表現 (Phase 1.1)
//! - [`layout`][layout]: 決定論的レイアウトエンジン (Phase 1.5)
//! - [`render`][render]: レンダラ群 — SVG (CLI 用、Phase 4.3 で削除予定) と配置済み IR エクスポート (Phase 4.0.9、serve の描画契約)
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
