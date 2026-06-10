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
