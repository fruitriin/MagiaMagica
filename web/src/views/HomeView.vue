<script setup lang="ts">
// M3: ペアビュー (関数目次 | ソース | 魔法陣)。URL (?fn=) を唯一の状態源とし、
// TOC クリックも 戻る/進む も「query 変更 → watch → selectFunction」の一方向で流す。
// 初回ロードは SSE 接続直後イベント (serve.rs 仕様) の refresh に一本化する。
import { watch } from "vue";
import { useRoute, useRouter } from "vue-router";

import FunctionToc from "../components/FunctionToc.vue";
import MagicCircleView from "../components/MagicCircleView.vue";
import SourcePane from "../components/SourcePane.vue";
import { useMagiaSync } from "../composables/useMagiaSync.ts";
import { useFocusStore } from "../stores/focus.ts";

const route = useRoute();
const router = useRouter();
const focus = useFocusStore();

function queryFn(): string | null {
  const fn = route.query["fn"];
  return typeof fn === "string" ? fn : null;
}

focus.setInitialFn(queryFn());
useMagiaSync();

// URL → store (TOC クリック・戻る/進む・手入力の全経路がここを通る)。
watch(
  () => route.query["fn"],
  (fn) => {
    if (typeof fn === "string" && fn !== focus.currentFn) {
      void focus.selectFunction(fn);
    }
  },
);

// store → URL (fn 未指定で先頭関数に fallback したときの書き戻し)。
// 履歴は汚さない (replace)。同一値ガードで watch との往復ループは止まる。
watch(
  () => focus.currentFn,
  (fn) => {
    if (fn !== null && fn !== queryFn()) {
      void router.replace({ query: { ...route.query, fn } });
    }
  },
);
</script>

<template>
  <div flex h-screen flex-col font-sans>
    <header flex items-baseline gap-3 border-b border-gray-200 px-4 py-2>
      <h1 text-lg font-bold tracking-wide>MagiaMagica</h1>
      <span v-if="focus.currentFn" text-sm text-gray-600 font-mono>{{ focus.currentFn }}</span>
      <span v-if="focus.file" text-xs text-gray-400>{{ focus.file }}</span>
    </header>

    <div v-if="focus.serverError" border-b border-red-300 bg-red-50 px-4 py-2 text-sm text-red-800>
      構文エラー: {{ focus.serverError.message }}
      <template v-if="focus.serverError.line">({{ focus.serverError.line }} 行目)</template>
      — 直前の正常な魔法陣を表示しています
    </div>
    <div
      v-else-if="focus.loadError"
      border-b
      border-amber-300
      bg-amber-50
      px-4
      py-2
      text-sm
      text-amber-800
    >
      取得エラー: {{ focus.loadError }} — 直前の表示を保持しています
    </div>

    <main flex min-h-0 flex-1>
      <FunctionToc w-48 shrink-0 border-r border-gray-200 />
      <SourcePane min-w-0 flex-1 border-r border-gray-200 />
      <MagicCircleView min-w-0 flex-1 overflow-auto />
    </main>
  </div>
</template>
