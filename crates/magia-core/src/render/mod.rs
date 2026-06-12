//! レンダラ群 (Phase 1.6〜, spec §6.1.3 / §6.1.5 / v0.3 §16)。
//!
//! - `ir_export` / `belka::belka_ir`: 配置済み IR の JSON エクスポート
//!   (Vue クライアント + Vue SSR (`magia-render`) の描画契約)。
//!   **SVG 文字列の生成は Phase 4.3 M5 で Vue へ一本化済み** — Rust は
//!   「どこに何があるか」(座標・半径・配置) だけを確定する。

pub mod belka;
pub mod ir_export;
mod midchilda;
// 色相規約はレンダラの内部実装。公開 API に色定数を露出させない (POSD 情報隠蔽)。
// 外部から色を参照したくなったら (Phase 2 dev-server 等) そのとき公開を判断する。
pub(crate) mod palette;

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
