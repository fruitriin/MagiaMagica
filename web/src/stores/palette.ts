// 表示様式とレイヤー可視性 (Phase 2.2〜2.3 の UI 状態の置き場)。
// レイヤー切替は CSS のみで位置を変えない (spec §5.4 位置共有制約) — 適用は
// MagicCircleView が SVG の <g> に対して行い、この store は状態だけ持つ。

import { defineStore } from "pinia";
import { ref } from "vue";

import type { RenderStyle } from "../types/magia.ts";

/** レイヤー名は Rust 側 FilterSpec / SVG の g.layer-* と同語彙 (spec §8)。 */
export const LAYERS = ["control_flow", "effects", "type_info"] as const;
export type LayerName = (typeof LAYERS)[number];

/** レイヤーの日本語表示名 (inline HTML 版と同じ語彙)。 */
export const LAYER_LABELS: Record<LayerName, string> = {
  control_flow: "制御フロー",
  effects: "効果",
  type_info: "型情報",
};

export type LayerState = {
  visible: boolean;
  opacity: number;
};

function allVisible(): Record<LayerName, LayerState> {
  return {
    control_flow: { visible: true, opacity: 1 },
    effects: { visible: true, opacity: 1 },
    type_info: { visible: true, opacity: 1 },
  };
}

export const usePaletteStore = defineStore("palette", () => {
  const style = ref<RenderStyle>("midchilda");
  const layers = ref<Record<LayerName, LayerState>>(allVisible());

  function setStyle(next: RenderStyle) {
    style.value = next;
  }

  function setVisible(layer: LayerName, visible: boolean) {
    layers.value[layer].visible = visible;
  }

  function setOpacity(layer: LayerName, opacity: number) {
    layers.value[layer].opacity = opacity;
  }

  function showAll() {
    for (const layer of LAYERS) layers.value[layer].visible = true;
  }

  function hideAll() {
    for (const layer of LAYERS) layers.value[layer].visible = false;
  }

  /** 可視レイヤー集合を一括設定する (DSL 適用・URL 復元の入口)。透明度は保持。 */
  function setVisibleSet(visible: ReadonlySet<LayerName>) {
    for (const layer of LAYERS) layers.value[layer].visible = visible.has(layer);
  }

  return {
    style,
    layers,
    setStyle,
    setVisible,
    setOpacity,
    showAll,
    hideAll,
    setVisibleSet,
  };
});
