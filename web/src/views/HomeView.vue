<script setup lang="ts">
// ペアビュー (魔法陣 | ソース | パレット + 関数目次)。URL を唯一の状態源とし、
// UI 操作 → store → URL (replace) → watch → store の一方向で流す (useQuerySync)。
// 関数切替 (?fn=) だけは FunctionToc が push して履歴に積む。
// 初回ロードは SSE 接続直後イベント (serve.rs 仕様) の refresh に一本化する。
import FunctionToc from "../components/FunctionToc.vue";
import LayerPalette from "../components/LayerPalette.vue";
import MagicCircleView from "../components/MagicCircleView.vue";
import SourcePane from "../components/SourcePane.vue";
import TranscriptRegion from "../components/TranscriptRegion.vue";
import { useMagiaSync } from "../composables/useMagiaSync.ts";
import { useQuerySync } from "../composables/useQuerySync.ts";
import { useFocusStore } from "../stores/focus.ts";

const focus = useFocusStore();
useQuerySync();
useMagiaSync();
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

    <TranscriptRegion />

    <main flex min-h-0 flex-1>
      <!-- 一番見せたいのは魔法陣 (オーナー判定 M3): 左端 + 最大幅でゆったり置く -->
      <MagicCircleView min-w-0 class="flex-[1.6]" overflow-auto />
      <SourcePane min-w-0 class="flex-[1]" border-l border-gray-200 />
      <aside flex w-56 shrink-0 flex-col border-l border-gray-200>
        <LayerPalette shrink-0 border-b border-gray-200 />
        <FunctionToc min-h-0 flex-1 />
      </aside>
    </main>
  </div>
</template>
