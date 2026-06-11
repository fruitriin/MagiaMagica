//! MagiaIR — 言語非依存の中間表現。
//!
//! 設計方針 (spec §4.1):
//! - 将来想定される全関心軸 (制御フロー、データフロー、副作用、テスト被覆、
//!   プロファイル、git 履歴、AI 注釈等) を格納可能な多次元構造とする
//! - Phase 1 で値が入らないフィールドも `Option` または空コレクションで
//!   スキーマ上の場所を確保する
//!
//! 後方互換性の規約:
//! - 全 struct に `#[serde(default)]` を付与し、欠落フィールドは `Default` で補う
//! - `#[serde(deny_unknown_fields)]` は **意図的に付けない**。新しい Phase で追加された
//!   フィールドを古いバイナリが無視して読めるようにするため
//!
//! モジュール構成:
//! - [`graph`][graph]: プロジェクト全体グラフとモジュール
//! - [`sigil`][sigil]: 図形ノード (リング・記号) と ID/濃度
//! - [`operation`][operation]: 処理単位と副作用
//! - [`edge`][edge]: 線 (リング間・記号間の接続)
//! - [`layers`][layers]: 多次元レイヤー情報
//! - [`source`][source]: ソース位置情報

pub mod edge;
pub mod graph;
pub mod layers;
pub mod operation;
pub mod sigil;
pub mod source;

pub use edge::{Edge, EdgeDataFlowInfo, EdgeKind, EdgeLayerData, EdgeProfileInfo};
pub use graph::{MagiaGraph, Module, ModuleId, ProjectMetadata};
pub use layers::{
    AiAnnotation, AuxRingKind, AuxRingRole, ChurnInfo, ConcurrencyInfo, ControlFlowInfo,
    CoverageInfo, DataFlowInfo, LayerData, LifetimeInfo, LoopKind, ProfileInfo, SecurityInfo,
    TypeInfo,
};
pub use operation::{CustomEffect, EffectSet, Operation, OperationKind, OperationPayload};
pub use sigil::{Cardinality, Sigil, SigilId, SigilKind};
pub use source::SourceSpan;
