//! プロジェクト全体グラフとモジュール。

use serde::{Deserialize, Serialize};

use super::edge::Edge;
use super::sigil::Sigil;

/// モジュールを一意に識別する ID。決定論的に採番する (乱数禁止)。
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct ModuleId(pub u32);

/// プロジェクト全体のグラフ。MagiaIR のトップレベル。
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MagiaGraph {
    /// プロジェクト内のモジュール一覧。
    pub modules: Vec<Module>,
    /// モジュール間をまたぐ Edge。
    pub cross_module_edges: Vec<Edge>,
    /// プロジェクトメタデータ。
    pub metadata: ProjectMetadata,
}

/// モジュール (ファイル / クレート / コンポーネント等)。
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Module {
    pub id: ModuleId,
    pub name: String,
    /// このモジュールに属する図形ノード。
    pub sigils: Vec<Sigil>,
    /// モジュール内に閉じた Edge。
    pub edges: Vec<Edge>,
}

/// プロジェクト全体に関するメタ情報。
///
/// Phase 1 ではプロジェクト名のみを必須とし、将来 git 情報や解析時刻等を追加する。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: Option<String>,
    /// 解析対象のルートパス (Phase 1: 任意)。
    pub root_path: Option<String>,
}
