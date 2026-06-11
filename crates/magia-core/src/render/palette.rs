//! 効果カテゴリの色相規約 (spec §6.1.3)。
//!
//! 色の変更はこのモジュールだけで完結させる (計画の設計判断)。
//! カテゴリの語彙は `filter::EffectCategory` と共有する (Phase 2.3 でフィルターが
//! 同じ分類で記号を絞り込むため、分類ロジックを一元化する)。

use crate::filter::EffectCategory;
use crate::ir::EffectSet;

/// 純粋計算: 黒。
pub const PURE: &str = "#000000";
/// IO: 青。
pub const IO: &str = "#1f4dff";
/// ネットワーク: 紫。
pub const NETWORK: &str = "#7b3ff5";
/// DB: 緑。
pub const DB: &str = "#1fa341";
/// ファイルシステム: 茶。
pub const FILESYSTEM: &str = "#7a4a1c";
/// Unsafe: 赤 (Rust のみ)。
pub const UNSAFE: &str = "#d92626";

// ===== 差分強調チャネル (Phase 3.2, spec v0.3 §8) =====
// 効果カテゴリの色相規約 (上記6色) と衝突しない色を選ぶ。強調は記号の塗り替えでなく
// 輪郭ハロー/ゴーストで表現するため、面積の大きい縁取りでも視認できる彩度にする。

/// 追加された術式: 金 (新たに刻まれた印)。
pub const DIFF_ADDED: &str = "#d4a017";
/// 変更された術式: シアン (書き換えられた印)。
pub const DIFF_CHANGED: &str = "#00a0c0";
/// 削除された術式のゴースト: 灰 (消えた術式の残滓)。
pub const DIFF_REMOVED: &str = "#909090";

// ===== ベルカ式の三極 (Phase 3.5, spec v0.3 §14.2) =====
// 効果カテゴリの6色・diff の3色と衝突しない色相を選ぶ。
// 操作ドット自体は効果カテゴリ色 (上記) を式をまたいで共有する。

/// 生成 (値の誕生): 空色 (湧き出す泉)。
pub const BELKA_GENESIS: &str = "#2f86c9";
/// 変換 (鍛錬・加工): 琥珀 (鍛冶の火)。
pub const BELKA_TRANSMUTE: &str = "#c98a2f";
/// 消費 (放出・副作用・帰還): 深紅がかった臙脂 (撃ち出す砲)。
pub const BELKA_CONSUME: &str = "#b04a5a";

/// `EffectSet` → 表示カテゴリ。分類の実体は語彙の持ち主である
/// `filter::EffectCategory::of` (レンダラ外からも使う共有ロジック)。
pub(crate) fn category_of(effects: &EffectSet) -> EffectCategory {
    EffectCategory::of(effects)
}

/// カテゴリ → 表示色。
pub(crate) fn color_of(category: EffectCategory) -> &'static str {
    match category {
        EffectCategory::Pure => PURE,
        EffectCategory::Io => IO,
        EffectCategory::Network => NETWORK,
        EffectCategory::Db => DB,
        EffectCategory::Filesystem => FILESYSTEM,
        EffectCategory::Unsafe => UNSAFE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_wins_over_everything() {
        let effects = EffectSet {
            unsafe_block: true,
            io: true,
            network: true,
            ..EffectSet::default()
        };
        assert_eq!(category_of(&effects), EffectCategory::Unsafe);
        assert_eq!(color_of(category_of(&effects)), UNSAFE);
    }

    #[test]
    fn default_is_pure_black() {
        assert_eq!(category_of(&EffectSet::default()), EffectCategory::Pure);
        assert_eq!(color_of(category_of(&EffectSet::default())), PURE);
    }

    #[test]
    fn each_single_category_maps_to_its_color() {
        let cases = [
            (
                EffectSet {
                    io: true,
                    ..EffectSet::default()
                },
                EffectCategory::Io,
                IO,
            ),
            (
                EffectSet {
                    network: true,
                    ..EffectSet::default()
                },
                EffectCategory::Network,
                NETWORK,
            ),
            (
                EffectSet {
                    db: true,
                    ..EffectSet::default()
                },
                EffectCategory::Db,
                DB,
            ),
            (
                EffectSet {
                    filesystem: true,
                    ..EffectSet::default()
                },
                EffectCategory::Filesystem,
                FILESYSTEM,
            ),
        ];
        for (effects, expected_category, expected_color) in cases {
            assert_eq!(category_of(&effects), expected_category);
            assert_eq!(color_of(category_of(&effects)), expected_color);
        }
    }
}
