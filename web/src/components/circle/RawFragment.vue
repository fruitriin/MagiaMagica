<script setup lang="ts">
// 意味論を復元しない SVG 断片の素通し描画 (複合記号 sym-*、ベルカ式の力場など)。
// markup はサーバ生成 SVG の断片 (信頼済み入力)。Phase 4.0.9 で意味論が IR から
// 直接来るようになったら、この素通しは段階的に専用コンポーネントへ置き換わる。
import type { RawElement } from "../../types/magia.ts";

defineProps<{ raw: RawElement }>();
</script>

<template>
  <g v-html="raw.markup" />
</template>
