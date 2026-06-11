//! リング内の処理単位と副作用カテゴリ。

use serde::{Deserialize, Serialize};

/// リング内の処理単位。
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Operation {
    pub kind: OperationKind,
    pub effects: EffectSet,
    pub payload: OperationPayload,
}

/// 処理種別 (spec §4.2)。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationKind {
    #[default]
    Compute,
    Branch,
    Loop,
    Match,
    Call,
    Await,
    Yield,
    Return,
    Throw,
}

/// 処理単位に紐づく追加情報。Phase 1 では最小限。
///
/// - `source_excerpt`: 元ソースの抜粋 (デバッグ・ホバー用)
/// - `call_target`: `OperationKind::Call` のときに呼び出し先のフルパス候補を保持
/// - `early_return`: `?` 演算子のような早期リターンであるかを示すフラグ
/// - `defs` / `uses`: この Operation が定義・使用する変数名 (Phase 3.4 のデータフロー
///   近似)。どの構文から取ったかをデバッグ可能にする説明可能性のための保持で、
///   集計値は `DataFlowInfo` 側にある。再代入 (`x = e`) も `defs` に含まれ、
///   複合代入 (`x += e`) では同じ変数が `uses` にも同時に現れる
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OperationPayload {
    pub source_excerpt: Option<String>,
    pub call_target: Option<String>,
    pub early_return: bool,
    pub defs: Vec<String>,
    pub uses: Vec<String>,
}

/// 副作用カテゴリ (spec §4.2)。
///
/// `pure` と他フィールドは排他ではなく、解析が判定した観測事実をそのまま積む。
/// 矛盾の解消はレンダラの色相規約 (spec §6.1.3) に委ねる。
///
/// spec §4.2 が直に定義する直交フラグ集合のため、`struct_excessive_bools` は許容する。
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EffectSet {
    pub pure: bool,
    pub io: bool,
    pub network: bool,
    pub db: bool,
    pub filesystem: bool,
    pub mutation: bool,
    pub unsafe_block: bool,
    /// 将来拡張用のドメイン特化効果 (Phase 1 では空)。
    pub custom: Vec<CustomEffect>,
}

/// ドメイン特化の副作用カテゴリ (Phase 4+ で `magia-glyph-*` クレートが利用)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomEffect {
    pub name: String,
    pub description: Option<String>,
}
