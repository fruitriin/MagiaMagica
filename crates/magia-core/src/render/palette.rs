//! 効果カテゴリの分類 (spec §6.1.3)。
//!
//! Phase 4.3 M5: **色の定義 (HEX) は Vue 側へ一本化済み** — 正は
//! `web/src/converters/irToSchema.ts` の `COLOR_BY_EFFECT`、ベルカ三極は
//! `BelkaCircle.vue` の `POLE_STYLE`、差分強調は `MagicCircle.vue` の
//! `DIFF_STYLE` (uno.config.ts の theme とも同語彙)。
//! Rust 側はカテゴリ語彙 (`EffectCategory`) だけを IR に出す。

use crate::filter::EffectCategory;
use crate::ir::EffectSet;

/// `EffectSet` → 表示カテゴリ。分類の実体は語彙の持ち主である
/// `filter::EffectCategory::of` (レンダラ外からも使う共有ロジック)。
pub(crate) fn category_of(effects: &EffectSet) -> EffectCategory {
    EffectCategory::of(effects)
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
    }

    #[test]
    fn default_is_pure() {
        assert_eq!(category_of(&EffectSet::default()), EffectCategory::Pure);
    }

    #[test]
    fn each_single_effect_maps_to_its_category() {
        let cases = [
            (
                EffectSet {
                    io: true,
                    ..EffectSet::default()
                },
                EffectCategory::Io,
            ),
            (
                EffectSet {
                    network: true,
                    ..EffectSet::default()
                },
                EffectCategory::Network,
            ),
            (
                EffectSet {
                    db: true,
                    ..EffectSet::default()
                },
                EffectCategory::Db,
            ),
            (
                EffectSet {
                    filesystem: true,
                    ..EffectSet::default()
                },
                EffectCategory::Filesystem,
            ),
        ];
        for (effects, expected_category) in cases {
            assert_eq!(category_of(&effects), expected_category);
        }
    }
}
