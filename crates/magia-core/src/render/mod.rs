//! レンダラ群 (Phase 1.6〜, spec §6.1.3 / §6.1.5 / v0.3 §16)。
//!
//! - SVG レンダラ (`render` / `render_with` / `render_diff`): `MagiaGraph` +
//!   `LayoutResult` から SVG 文字列を生成する。CLI (`magia render` / `diff` / `ci`)
//!   専用 — serve の動的 UI は Phase 4.0.9 で `ir_export` へ移行済み。
//!   **Phase 4.3 (Vue SSR 一本化) で削除予定**
//! - `ir_export`: 配置済み IR の JSON エクスポート (Vue クライアントの描画契約)。
//!
//! SVG 生成は `std::fmt::Write` ベースの自前ビルダー (tech-selection §2.4:
//! 属性順序を固定でき、スナップショットテストが安定する)。

mod belka;
pub mod ir_export;
mod midchilda;
// 色相規約はレンダラの内部実装。公開 API に色定数を露出させない (POSD 情報隠蔽)。
// 外部から色を参照したくなったら (Phase 2 dev-server 等) そのとき公開を判断する。
pub(crate) mod palette;

use crate::diff::SpellDiff;
use crate::filter::FilterSpec;
use crate::ir::MagiaGraph;
use crate::layout::LayoutResult;

/// 描画式 (魔法陣の流派)。Phase 1 でミッドチルダ式、Phase 3.5 でベルカ式を実装。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderStyle {
    /// ミッドチルダ式 ConcentricRings (Phase 1 のフラッグシップ)。
    #[default]
    MidchildaConcentric,
    /// ベルカ式 (Phase 3.5: データフロー三角力場)。
    Belka,
    /// 夜天の書式 (Phase 6+、未実装)。
    Yagami,
}

impl RenderStyle {
    /// 選択可能な式 (CLI / serve / DSL で共有する語彙)。Yagami は未実装のため含めない。
    pub const SELECTABLE: [RenderStyle; 2] = [RenderStyle::MidchildaConcentric, RenderStyle::Belka];

    /// CLI / URL クエリで使う名前。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            RenderStyle::MidchildaConcentric => "midchilda",
            RenderStyle::Belka => "belka",
            RenderStyle::Yagami => "yagami",
        }
    }
}

impl std::str::FromStr for RenderStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RenderStyle::SELECTABLE
            .into_iter()
            .find(|style| style.as_str() == s)
            .ok_or_else(|| {
                format!(
                    "未知の式 `{s}` (使用可能: {})",
                    RenderStyle::SELECTABLE.map(RenderStyle::as_str).join(", ")
                )
            })
    }
}

/// レイアウト済みのグラフを SVG 文字列へ描画する。
///
/// 同じ入力からは常に同一の文字列が出る (決定論、spec §6.1.4)。
/// 出力はレイヤーごとに `<g class="layer-*">` で分離され、Phase 2 の
/// CSS レイヤー切替にそのまま使える (spec §6.1.5)。
///
/// # Panics
///
/// 未実装の式 (`Yagami` は Phase 6+) を指定すると panic する。
#[must_use = "SVG 文字列は出力先に書き込まれるべき"]
pub fn render(graph: &MagiaGraph, layout: &LayoutResult, style: RenderStyle) -> String {
    render_with(graph, layout, style, &FilterSpec::default())
}

/// フィルター (spec v0.2 §8) を適用して描画する。
///
/// `show` に含まれないレイヤーは `<g>` ごと出力されず、`effects[...]` の
/// カテゴリ絞り込みは Operation ドット・召喚記号の単位で適用される
/// (CSS では色相による絞り込みが表現できないため render 時に行う)。
///
/// # Panics
///
/// 未実装の式 (`Yagami` は Phase 6+) を指定すると panic する。
#[must_use = "SVG 文字列は出力先に書き込まれるべき"]
pub fn render_with(
    graph: &MagiaGraph,
    layout: &LayoutResult,
    style: RenderStyle,
    filter: &FilterSpec,
) -> String {
    match style {
        RenderStyle::MidchildaConcentric => midchilda::render(graph, layout, filter),
        // ベルカ式は三角配置を閉形式で決めるため LayoutResult と FilterSpec を使わない
        // (レイヤー語彙はミッドチルダ式の3層前提。CLI が併用を明示エラーにする)。
        RenderStyle::Belka => belka::render(graph),
        RenderStyle::Yagami => unimplemented!("夜天の書式は Phase 6+"),
    }
}

/// 差分強調つきで描画する (Phase 3.2, spec v0.3 §8)。
///
/// after を基準に描画し、`SpellDiff` の追加/変更ノードにハロー輪郭、削除ノードに
/// before 位置のゴースト (半透明破線) を `<g class="overlay-diff">` として重ねる。
/// 強調チャネルはレイヤーの show/hide の影響を受けない。
/// レイアウトは内部で before / after 両方を計算する (呼び出し側は IR と diff を
/// 渡すだけでよい — 2つのレイアウトの対応付けという複雑さを下に畳む)。
///
/// # Panics
///
/// diff 強調はミッドチルダ式のみ対応 (`Belka` の差分強調は Phase 3 振り返りで判断、
/// `Yagami` は未実装)。それ以外の式を指定すると panic する。
#[must_use = "SVG 文字列は出力先に書き込まれるべき"]
pub fn render_diff(
    before: &MagiaGraph,
    after: &MagiaGraph,
    diff: &SpellDiff,
    style: RenderStyle,
    filter: &FilterSpec,
) -> String {
    match style {
        RenderStyle::MidchildaConcentric => midchilda::render_diff(before, after, diff, filter),
        RenderStyle::Belka | RenderStyle::Yagami => {
            unimplemented!("diff 強調はミッドチルダ式のみ (Phase 3.5 時点)")
        }
    }
}
