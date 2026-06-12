<script setup lang="ts">
// ペアビュー (魔法陣 | ソース | パレット + 関数目次)。URL を唯一の状態源とし、
// UI 操作 → store → URL (replace) → watch → store の一方向で流す (useQuerySync)。
// 関数切替 (?fn=) だけは FunctionToc が push して履歴に積む。
// 初回ロードは SSE 接続直後イベント (serve.rs 仕様) の refresh に一本化する。
import { computed, onMounted, onUnmounted } from "vue";

import CallInspector from "../components/CallInspector.vue";
import FunctionToc from "../components/FunctionToc.vue";
import HoverPreview from "../components/HoverPreview.vue";
import LayerPalette from "../components/LayerPalette.vue";
import SymbolLegend from "../components/SymbolLegend.vue";
import MagicCircleView from "../components/MagicCircleView.vue";
import SourcePane from "../components/SourcePane.vue";
import TranscriptRegion from "../components/TranscriptRegion.vue";
import WorkspaceView from "../components/WorkspaceView.vue";
import { useMagiaSync } from "../composables/useMagiaSync.ts";
import { useQuerySync } from "../composables/useQuerySync.ts";
import { useFocusStore } from "../stores/focus.ts";

const focus = useFocusStore();
useQuerySync();
useMagiaSync();

// 監視ファイルの切替候補 (Phase 4.4.5)。一覧はヘッダのドロップダウンで使う。
onMounted(() => void focus.loadFiles());

/** ドロップダウンの候補。現在のファイルが一覧外 (絶対パス起動等) でも選択肢に出す。 */
const fileOptions = computed(() => {
  const options = [...focus.files];
  if (focus.file !== null && !options.includes(focus.file)) {
    options.unshift(focus.file);
  }
  return options;
});

function onFileChange(event: Event) {
  void focus.switchFile((event.target as HTMLSelectElement).value);
}

// F = フォーカス中心へ視点を戻す (Phase 4.1 キーボードナビ。
// Tab/Enter のチップ巡回はチップの button 化によりブラウザ標準動作)。
function onKeydown(event: KeyboardEvent) {
  if (event.key !== "f" && event.key !== "F") return;
  const target = event.target as HTMLElement | null;
  if (target && ["INPUT", "TEXTAREA"].includes(target.tagName)) return; // 入力中は無視
  document.querySelector("svg")?.scrollIntoView({ block: "center", inline: "center" });
}
onMounted(() => document.addEventListener("keydown", onKeydown));
onUnmounted(() => document.removeEventListener("keydown", onKeydown));
</script>

<template>
  <div flex h-screen flex-col font-sans>
    <header flex items-baseline gap-3 border-b border-gray-200 px-4 py-2>
      <h1 text-lg font-bold tracking-wide>MagiaMagica</h1>
      <span v-if="focus.currentFn" text-sm text-gray-600 font-mono>{{ focus.currentFn }}</span>
      <!-- 監視ファイルの切替 (Phase 4.4.5)。候補はワークスペース配下の .rs -->
      <select
        v-if="focus.file"
        :value="focus.file"
        aria-label="監視ファイル"
        max-w-80
        border-none
        bg-transparent
        text-xs
        text-gray-400
        font-mono
        cursor-pointer
        hover:text-gray-700
        @change="onFileChange"
      >
        <option v-for="f in fileOptions" :key="f" :value="f">{{ f }}</option>
      </select>
      <!-- 俯瞰トグル (Phase 4.5): ピン中心 ⇆ ワークスペース全体のズーム切替 -->
      <button
        ml-auto
        border
        border-gray-300
        rounded
        px-2
        py-0.5
        text-xs
        cursor-pointer
        :class="
          focus.scope === 'workspace'
            ? 'bg-cyan-600 text-white border-cyan-600'
            : 'bg-white text-gray-600 hover:border-gray-500'
        "
        @click="void focus.setScope(focus.scope === 'workspace' ? 'focus' : 'workspace')"
      >
        {{ focus.scope === "workspace" ? "ピンに戻る" : "俯瞰" }}
      </button>
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
    <CallInspector />
    <HoverPreview />

    <!-- 俯瞰 (Phase 4.5): scope=workspace のときはペアビューの代わりに全ファイルカード -->
    <WorkspaceView v-if="focus.scope === 'workspace'" min-h-0 flex-1 />
    <main v-else flex min-h-0 flex-1>
      <!-- 一番見せたいのは魔法陣 (オーナー判定 M3): 左端 + 最大幅でゆったり置く。
           凡例は魔法陣ペインの下 (オーナー判定 4.0.6) — 図と見比べながら読める位置。 -->
      <div min-w-0 class="flex-[1.6]" flex flex-col>
        <MagicCircleView min-h-0 flex-1 overflow-auto />
        <SymbolLegend shrink-0 border-t border-gray-200 />
      </div>
      <SourcePane min-w-0 class="flex-[1]" border-l border-gray-200 />
      <aside flex w-56 shrink-0 flex-col border-l border-gray-200>
        <LayerPalette shrink-0 border-b border-gray-200 />
        <FunctionToc min-h-0 flex-1 />
      </aside>
    </main>
  </div>
</template>
