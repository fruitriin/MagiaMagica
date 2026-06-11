import presetAttributify from "@unocss/preset-attributify";
import presetUno from "@unocss/preset-uno";
import { defineConfig } from "unocss";

// 色の語彙は Rust 側 crates/magia-core/src/render/palette.rs と同一に保つ (spec §6.1.3)。
// 値を変えるときは palette.rs と同時に変更する。
export default defineConfig({
  presets: [presetUno(), presetAttributify()],
  theme: {
    colors: {
      // 効果カテゴリ 6 色 (palette.rs: PURE..UNSAFE)
      effect: {
        pure: "#000000",
        io: "#1f4dff",
        network: "#7b3ff5",
        db: "#1fa341",
        filesystem: "#7a4a1c",
        unsafe: "#d92626",
      },
      // 差分強調チャネル (palette.rs: DIFF_*, spec v0.3 §8)
      diff: {
        added: "#d4a017",
        changed: "#00a0c0",
        removed: "#909090",
      },
      // ベルカ式の三極 (palette.rs: BELKA_*, spec v0.3 §14.2)
      belka: {
        genesis: "#2f86c9",
        transmute: "#c98a2f",
        consume: "#b04a5a",
      },
    },
  },
});
