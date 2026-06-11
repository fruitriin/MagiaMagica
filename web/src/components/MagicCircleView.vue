<script setup lang="ts">
// 魔法陣ペイン。M2 では store の SVG 文字列を v-html で素通しする過渡対応
// (Phase 4.0.7 で <MagicCircle :schema="MagicCircleSchema"> に置き換える)。
// SVG はサーバ (magia serve) 生成の信頼済み入力。
import { useFocusStore } from "../stores/focus.ts";

const focus = useFocusStore();
</script>

<template>
  <div v-if="focus.currentSvg" w-full v-html="focus.currentSvg" />
  <div v-else p-8 text-gray-400>魔法陣を読み込み中…</div>
</template>
