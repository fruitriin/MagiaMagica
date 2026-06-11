<script setup lang="ts">
// 魔法陣ペイン (Phase 4.0.9 で IR 直結に移行)。
// ミッドチルダ式: 配置済み IR (spec v0.3 §16) を irToSchema で MagicCircleSchema に
// 展開し、<MagicCircle> ツリーが描画する。SSE 更新 → spell 差し替え → computed
// 再評価 → リアクティブ再描画。
// ベルカ式: Phase 4.3 の Vue SSR 移行まで SVG 文字列 (svg_belka) を素通し表示する
// (サーバ生成の信頼済み入力)。この v-html 分岐は 4.3 のリメイクで消える過渡コード —
// 保守価値は低め (オーナー判定 2026-06-11)、改善はリメイクに織り込む。
import { computed } from "vue";

import { irToSchema } from "../converters/irToSchema.ts";
import { useFocusStore } from "../stores/focus.ts";
import { usePaletteStore } from "../stores/palette.ts";
import MagicCircle from "./circle/MagicCircle.vue";

const focus = useFocusStore();
const palette = usePaletteStore();

const schema = computed(() => {
  if (focus.spell === null || palette.style === "belka") return null;
  return irToSchema(focus.spell.ir);
});

const belkaSvg = computed(() =>
  palette.style === "belka" ? (focus.spell?.svg_belka ?? null) : null,
);
</script>

<template>
  <div v-if="schema" w-full>
    <MagicCircle :schema="schema" />
  </div>
  <div v-else-if="belkaSvg" w-full v-html="belkaSvg" />
  <div v-else p-8 text-gray-400>魔法陣を読み込み中…</div>
</template>
