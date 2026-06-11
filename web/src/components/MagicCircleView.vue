<script setup lang="ts">
// 魔法陣ペイン (Phase 4.0.7 で v-html → 境界スキーマ + コンポーネントツリーに置換)。
// SVG 文字列 → MagicCircleSchema の変換はここだけが svgToSchema を参照する
// (converters/ の隔離 — Phase 4.0.9 で IR ビルダに差し替えるとき、この computed と
// svgToSchema.ts だけが変わり、<MagicCircle> 以下は無修正で流用される)。
// SSE 更新 → currentSvg 変化 → computed 再評価 → Vue がリアクティブ再描画。
import { computed } from "vue";

import { svgToSchema } from "../converters/svgToSchema.ts";
import { useFocusStore } from "../stores/focus.ts";
import { usePaletteStore } from "../stores/palette.ts";
import MagicCircle from "./circle/MagicCircle.vue";

const focus = useFocusStore();
const palette = usePaletteStore();

const schema = computed(() => {
  if (focus.currentSvg === null) return null;
  return svgToSchema(focus.currentSvg, palette.style);
});
</script>

<template>
  <div v-if="schema" w-full>
    <MagicCircle :schema="schema" />
  </div>
  <div v-else p-8 text-gray-400>魔法陣を読み込み中…</div>
</template>
