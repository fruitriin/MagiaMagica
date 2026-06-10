//! SVG レンダラ (Phase 1.6, spec §6.1.3 / §6.1.5)。
//!
//! `MagiaGraph` + `LayoutResult` から SVG 文字列を生成する。
//! Phase 1 はミッドチルダ式 ConcentricRings バリアントのみ実装する。
//! SVG 生成は `std::fmt::Write` ベースの自前ビルダー (tech-selection §2.4:
//! 属性順序を固定でき、スナップショットテストが安定する)。

mod midchilda;
// 色相規約はレンダラの内部実装。公開 API に色定数を露出させない (POSD 情報隠蔽)。
// 外部から色を参照したくなったら (Phase 2 dev-server 等) そのとき公開を判断する。
pub(crate) mod palette;

use crate::filter::FilterSpec;
use crate::ir::MagiaGraph;
use crate::layout::LayoutResult;

/// 描画式 (魔法陣の流派)。Phase 1 はミッドチルダ式のみ実装。
///
/// バリアント名は将来の式追加に開いた形で先に定義しておく
/// (実装は `unimplemented!` stub)。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderStyle {
    /// ミッドチルダ式 ConcentricRings (Phase 1 のフラッグシップ)。
    #[default]
    MidchildaConcentric,
    /// ベルカ式 (Phase 3+: データフロー三角陣)。
    Belka,
    /// 夜天の書式 (Phase 6+)。
    Yagami,
}

/// レイアウト済みのグラフを SVG 文字列へ描画する。
///
/// 同じ入力からは常に同一の文字列が出る (決定論、spec §6.1.4)。
/// 出力はレイヤーごとに `<g class="layer-*">` で分離され、Phase 2 の
/// CSS レイヤー切替にそのまま使える (spec §6.1.5)。
///
/// # Panics
///
/// Phase 1 で未実装の式 (`Belka` / `Yagami`) を指定すると panic する。
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
/// Phase 1 で未実装の式 (`Belka` / `Yagami`) を指定すると panic する。
#[must_use = "SVG 文字列は出力先に書き込まれるべき"]
pub fn render_with(
    graph: &MagiaGraph,
    layout: &LayoutResult,
    style: RenderStyle,
    filter: &FilterSpec,
) -> String {
    match style {
        RenderStyle::MidchildaConcentric => midchilda::render(graph, layout, filter),
        RenderStyle::Belka | RenderStyle::Yagami => {
            unimplemented!("not implemented in Phase 1")
        }
    }
}
