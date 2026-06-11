<script setup lang="ts">
// M2: 魔法陣だけ Vue で出る画面。`?fn=` (Phase 4.0 互換) で初期表示の関数を選ぶ。
// 左右ペアビュー (ソース | 魔法陣)・関数目次・URL 同期は M3 で載せる。
import { onMounted } from "vue";
import { useRoute } from "vue-router";

import MagicCircleView from "../components/MagicCircleView.vue";
import { useMagiaSync } from "../composables/useMagiaSync.ts";
import { useFocusStore } from "../stores/focus.ts";

const route = useRoute();
const focus = useFocusStore();
useMagiaSync();

onMounted(async () => {
  await focus.loadState();
  const fn = typeof route.query["fn"] === "string" ? route.query["fn"] : null;
  await focus.selectFunction(fn);
});
</script>

<template>
  <main mx-auto max-w-4xl p-4 font-sans>
    <header flex items-baseline gap-3>
      <h1 text-xl font-bold tracking-wide>MagiaMagica</h1>
      <span v-if="focus.currentFn" text-sm text-gray-500>{{ focus.currentFn }}</span>
      <span v-if="focus.file" text-xs text-gray-400>{{ focus.file }}</span>
    </header>
    <MagicCircleView mt-4 />
  </main>
</template>
