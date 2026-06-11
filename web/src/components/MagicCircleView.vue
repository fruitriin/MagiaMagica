<script setup lang="ts">
// 魔法陣ペイン。M2 では store の SVG 文字列を v-html で素通しする過渡対応
// (Phase 4.0.7 で <MagicCircle :schema="MagicCircleSchema"> に置き換える)。
// SVG はサーバ (magia serve) 生成の信頼済み入力。
//
// レイヤー切替 (Phase 2.2) は SVG の <g class="layer-*"> に display/opacity を
// 当てるだけで位置を変えない (spec §5.4 位置共有制約)。v-html の DOM には
// リアクティビティが届かないため、描画後に watch で適用する (4.0.7 で宣言化)。
import { nextTick, ref, watch } from "vue";

import { useFocusStore } from "../stores/focus.ts";
import { LAYERS, usePaletteStore } from "../stores/palette.ts";

const focus = useFocusStore();
const palette = usePaletteStore();
const container = ref<HTMLElement | null>(null);

function applyLayers() {
  if (container.value === null) return;
  for (const layer of LAYERS) {
    // クラス名は Rust レンダラの <g class="layer-*"> と同語彙 (spec §5.4 位置共有制約)。
    const cssClass = `layer-${layer.replace(/_/g, "-")}`;
    const state = palette.layers[layer];
    for (const group of container.value.querySelectorAll<SVGGElement>(`g.${cssClass}`)) {
      group.style.display = state.visible ? "" : "none";
      group.style.opacity = String(state.opacity);
    }
  }
}

watch([() => focus.currentSvg, () => palette.layers], () => void nextTick(applyLayers), {
  deep: true,
  immediate: true,
});
</script>

<template>
  <div v-if="focus.currentSvg" ref="container" w-full v-html="focus.currentSvg" />
  <div v-else p-8 text-gray-400>魔法陣を読み込み中…</div>
</template>
