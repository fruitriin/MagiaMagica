// 表示様式とレイヤー可視性 (Phase 2.2〜2.3 の UI 状態の置き場)。
// M2 ではスケルトン — 切替 UI は M4 の <LayerPalette> / <DslEditor> で載る。

import { defineStore } from "pinia";
import { ref } from "vue";

import type { RenderStyle } from "../types/magia.ts";

/** レイヤー名は Rust 側 FilterSpec の語彙と同じ (spec §8)。 */
export type LayerName = "control" | "effects" | "types";

export type LayerState = {
  visible: boolean;
  opacity: number;
};

export const usePaletteStore = defineStore("palette", () => {
  const style = ref<RenderStyle>("midchilda");
  const layers = ref<Record<LayerName, LayerState>>({
    control: { visible: true, opacity: 1 },
    effects: { visible: true, opacity: 1 },
    types: { visible: true, opacity: 1 },
  });

  function setStyle(next: RenderStyle) {
    style.value = next;
  }

  return { style, layers, setStyle };
});
