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
///
/// カウント系フィールドは「そのリングの `content` に直接現れる構造」のみを数える。
/// 入れ子の制御構造は対応する AuxRing 側の `ControlFlowInfo` に計上されるため、
/// 関数全体の合計はリングを辿って総和を取る (二重計上を防ぐ規約)。
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
    /// この Sigil が AuxRing のとき、親リングとの接続情報。MainRing では `None`。
    pub role: Option<AuxRingRole>,
}

/// AuxRing が親リングに対して持つ役割 (spec §6.1.2 の補助リング)。
///
/// Phase 1.5 のレイアウトエンジンが「親リング上のどこに補助リングを置くか」を決める
/// ための情報。制御構造は親リングの `content` 上で常に1個の Operation を占めるため、
/// `anchor_operation` が入口かつ出口の位置 (spec §6.1.4 の極座標配置の基準点) になる。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AuxRingRole {
    /// 分岐種別・ループ種別。
    pub kind: AuxRingKind,
    /// 親 Sigil の `content` 内で、この制御構造に対応する Operation の添字。
    pub anchor_operation: u32,
    /// 同一制御構造内での序数 (if 連鎖の何番目の分岐か / match の何アーム目か)。
    /// ループ本体は常に 0。
    pub ordinal: u32,
    /// 表示用ラベル (match アームのパターン等)。
    pub label: Option<String>,
    /// この腕のガード・ヘッダの原文位置 (`if cond` / `pat if guard` /
    /// `for pat in expr` 等)。`else` のような無条件の腕は `None`。
    /// 補助リングのホバープレビューに使う (Phase 4.1 追加要望4)。
    pub guard_location: Option<super::SourceSpan>,
}

/// AuxRing の分岐種別 (spec §6.1.3 の制御構造記号に対応)。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuxRingKind {
    /// `if` / `else if` の then 節。
    #[default]
    IfBranch,
    /// if 連鎖末尾の `else` 節。
    ElseBranch,
    /// `match` のアーム1つ。
    MatchArm,
    /// ループ本体。
    LoopBody(LoopKind),
    /// 召喚の引数に渡されたクロージャ (コールバック) の本体 (Phase 4.8 M2)。
    /// 親は SummonGlyph (glyph が子リングを持つ唯一の形)。
    Closure,
}

/// ループ種別。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoopKind {
    #[default]
    For,
    While,
    Loop,
}

/// データフロー情報 (Phase 3.4 で値を埋める)。
///
/// 集計単位: このリングで def された変数のみ (use が子リングで起きても def 側に計上 —
/// ControlFlowInfo と同じ「二重計上を防ぐ規約」)。再代入 (`x = e` / `x += e`) は
/// 「再代入が起きたリングでの新 def」として扱うため、同名変数が別リングで再代入されると
/// チェーンはそのリング側に帰属する (元リングとは別チェーン、二重計上はない)。
/// syn ベースの近似であり、マクロ内・クロージャ内の def/use は追わない。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DataFlowInfo {
    /// Use-Def chain の数 = このリングで def され、1回以上 use された変数の数。
    pub use_def_chains: u32,
    /// 最長チェーン長 = チェーンを構成する変数に触れた Operation 数
    /// (def 1 + use 回数) の最大値。チェーンが1本もなければ 0。
    pub longest_chain: u32,
}

/// 型情報 (Phase 1 では関数シグネチャ程度を埋める)。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TypeInfo {
    /// 関数シグネチャの文字列 (例: `fn foo(a: i32) -> Result<(), Error>`)。
    pub signature: Option<String>,
    /// 関数名 (`foo`)。シグネチャ表示の組み立て用 (細部修正 2026-06-12)。
    #[serde(default)]
    pub fn_name: Option<String>,
    /// 引数 (self を除く) の (パターン, 型) — 空白を詰めた表示用文字列。
    #[serde(default)]
    pub args: Vec<(String, String)>,
    /// 戻り値型 (表示用、`-> ` なし)。unit (`()`) は None。
    #[serde(default)]
    pub ret: Option<String>,
    /// 戻り値が `Result` か。
    pub returns_result: bool,
    /// 戻り値が `Option` か。
    pub returns_option: bool,
    /// Reducer 形 (`(A, B) -> A`: 第1引数型 = 戻り値型) か (Phase 3.5, spec §14.3)。
    /// 構文上のトークン一致による判定で、型別名は見抜けない (意味解決なしの近似)。
    pub reducer_shape: bool,
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
