//! Sigil に紐づく多次元レイヤー情報。
//!
//! spec §5.1 に列挙される全関心レイヤーをここに格納する。Phase 1 で値を埋めるのは
//! `control_flow` / `type_info` / `EffectSet` 経由の `effects` のみ。それ以外は
//! `Option` または空コレクションでスキーマ上の場所を確保する (spec §4.1 原則1)。

use serde::{Deserialize, Serialize};

/// 多次元レイヤー情報。Sigil ごとに1つ持つ。
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayerData {
    pub control_flow: Option<ControlFlowInfo>,
    pub data_flow: Option<DataFlowInfo>,
    pub type_info: Option<TypeInfo>,
    pub lifetime: Option<LifetimeInfo>,
    pub concurrency: Option<ConcurrencyInfo>,
    pub test_coverage: Option<CoverageInfo>,
    pub profile: Option<ProfileInfo>,
    pub git_churn: Option<ChurnInfo>,
    pub security: Option<SecurityInfo>,
    /// AI 注釈はレイヤーではなく「上に積む別チャネル」のため、`Option` ではなく
    /// 空 `Vec` で表現する (spec §5.1 の `ai_annotations`)。
    pub ai_annotations: Vec<AiAnnotation>,
}

/// 制御フロー (分岐、ループ、例外経路) (Phase 1 で値を埋める)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ControlFlowInfo {
    /// 分岐数。集計単位:
    /// - `if` 1 個につき 1 (else if / else を1個の if チェーンとしてまとめてカウント)
    /// - `match` 1 個につき (アーム数 - 1) を加算 (アームの分岐個数)
    /// - `?` 演算子は `early_return_count` 側に計上し、ここには含めない
    pub branch_count: u32,
    /// ループ構造の数 (for/while/loop それぞれ 1 として加算)。
    pub loop_count: u32,
    /// 早期リターンの経路数 (`return` 文 + `?` 演算子の総数)。
    pub early_return_count: u32,
}

/// データフロー情報 (Phase 3 以降で値を埋める)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DataFlowInfo {
    /// Use-Def chain の長さ等の指標 (Phase 3+ で具体化)。
    pub use_def_chains: u32,
}

/// 型情報 (Phase 1 では関数シグネチャ程度を埋める)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TypeInfo {
    /// 関数シグネチャの文字列 (例: `fn foo(a: i32) -> Result<(), Error>`)。
    pub signature: Option<String>,
    /// 戻り値が `Result` か。
    pub returns_result: bool,
    /// 戻り値が `Option` か。
    pub returns_option: bool,
}

/// Rust の借用・寿命情報 (Phase 5+)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LifetimeInfo {
    /// 寿命パラメータの名前列。
    pub parameters: Vec<String>,
}

/// 並行性情報 (Phase 1 では `is_async` フラグ程度)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConcurrencyInfo {
    pub is_async: bool,
    /// `.await` 点の数。
    pub await_points: u32,
}

/// テスト被覆 (Phase 5+)。
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CoverageInfo {
    /// 0.0〜1.0 の被覆率。
    pub ratio: f64,
}

/// プロファイル情報 (Phase 5+)。
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProfileInfo {
    /// 平均実行時間 (秒)。
    pub mean_seconds: f64,
    /// 呼び出し回数。
    pub call_count: u64,
}

/// git churn (Phase 5+)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChurnInfo {
    /// 過去 N 日間の変更行数。
    pub changed_lines: u32,
    /// 過去 N 日間のコミット数。
    pub commit_count: u32,
}

/// セキュリティ警告 (Phase 5+)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SecurityInfo {
    pub warnings: Vec<String>,
}

/// AI 注釈 (Phase 5+ の AI 注釈チャネル)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiAnnotation {
    /// 注釈を発した AI の識別子 (例: モデル名)。
    pub source: String,
    /// 注釈本文。
    pub message: String,
    /// 注釈の重要度 (`info` / `warn` / `critical` 等を自由に格納)。
    pub severity: Option<String>,
}
