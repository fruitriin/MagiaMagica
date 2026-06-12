import presetAttributify from "@unocss/preset-attributify";
import presetUno from "@unocss/preset-uno";
import { defineConfig } from "unocss";

// 色の正は Vue 側 (Phase 4.3 M5 で Rust の色定数は削除済み、spec §6.1.3)。
// 値を変えるときは irToSchema.ts の COLOR_BY_EFFECT / BelkaCircle.vue の
// POLE_STYLE・DOT_COLOR / MagicCircle.vue の DIFF_STYLE と同時に変更する。
export default defineConfig({
  presets: [presetUno(), presetAttributify()],
  theme: {
    colors: {
      // 効果カテゴリ 6 色 (irToSchema.ts: COLOR_BY_EFFECT)
      effect: {
        pure: "#000000",
        io: "#1f4dff",
        network: "#7b3ff5",
        db: "#1fa341",
        filesystem: "#7a4a1c",
        unsafe: "#d92626",
      },
      // 差分強調チャネル (MagicCircle.vue: DIFF_STYLE, spec v0.3 §8)
      diff: {
        added: "#d4a017",
        changed: "#00a0c0",
        removed: "#909090",
      },
      // ベルカ式の三極 (BelkaCircle.vue: POLE_STYLE, spec v0.3 §14.2)
      belka: {
        genesis: "#2f86c9",
        transmute: "#c98a2f",
        consume: "#b04a5a",
      },
    },
  },
});
