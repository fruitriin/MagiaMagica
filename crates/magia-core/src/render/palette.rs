//! 効果カテゴリの色相規約 (spec §6.1.3)。
//!
//! 色の変更はこのモジュールだけで完結させる (計画の設計判断)。

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

/// `EffectSet` から表示色を1つ選ぶ。
///
/// `EffectSet` は直交フラグの集合で複数同時に立ちうる (`operation.rs` の規約:
/// 「矛盾の解消はレンダラの色相規約に委ねる」)。ここでの優先順位は
/// **危険度・希少度の高い順**: unsafe > network > db > filesystem > io > pure。
/// unsafe を最優先するのは赤が警告色として最も伝達価値が高いため。
#[must_use]
pub fn effect_color(effects: &EffectSet) -> &'static str {
    if effects.unsafe_block {
        UNSAFE
    } else if effects.network {
        NETWORK
    } else if effects.db {
        DB
    } else if effects.filesystem {
        FILESYSTEM
    } else if effects.io {
        IO
    } else {
        PURE
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
        assert_eq!(effect_color(&effects), UNSAFE);
    }

    #[test]
    fn default_is_pure_black() {
        assert_eq!(effect_color(&EffectSet::default()), PURE);
    }

    #[test]
    fn each_single_category_maps_to_its_color() {
        let cases = [
            (
                EffectSet {
                    io: true,
                    ..EffectSet::default()
                },
                IO,
            ),
            (
                EffectSet {
                    network: true,
                    ..EffectSet::default()
                },
                NETWORK,
            ),
            (
                EffectSet {
                    db: true,
                    ..EffectSet::default()
                },
                DB,
            ),
            (
                EffectSet {
                    filesystem: true,
                    ..EffectSet::default()
                },
                FILESYSTEM,
            ),
        ];
        for (effects, expected) in cases {
            assert_eq!(effect_color(&effects), expected);
        }
    }
}
