<script setup lang="ts">
// レイヤーパレット (Phase 2.2〜2.3 の Vue 移植)。式切替・レイヤー可視性/透明度・
// 全表示/全非表示・.magia DSL の往復。状態は palette store、URL 同期は
// useQuerySync が担う (このコンポーネントは store を触るだけ)。
import { ref, watch } from "vue";

import { exportDsl, parseDsl } from "../lib/magiaDsl.ts";
import { useFocusStore } from "../stores/focus.ts";
import { LAYER_LABELS, LAYERS, usePaletteStore } from "../stores/palette.ts";

const palette = usePaletteStore();
const focus = useFocusStore();
const dslText = ref("");
const dslNote = ref("");

// Spell Diff (Phase 4.3.7): 比較基準 rev の入力。store (URL 同期) が正で、
// 入力欄はその編集ビュー — 戻る/進むで diffRev が変わったら追従する。
const diffInput = ref(focus.diffRev ?? "");
watch(
  () => focus.diffRev,
  (rev) => {
    diffInput.value = rev ?? "";
  },
);

function applyDiff(rev: string) {
  diffInput.value = rev;
  void focus.setDiffRev(rev === "" ? null : rev);
}

function onExport() {
  const visible = new Set(LAYERS.filter((l) => palette.layers[l].visible));
  dslText.value = exportDsl(visible);
  dslNote.value = "";
}

function onApply() {
  const result = parseDsl(dslText.value);
  if (!result.ok) {
    dslNote.value = result.error;
    return;
  }
  palette.setVisibleSet(result.visible);
  dslNote.value = result.note;
}
</script>

<template>
  <!-- 折りたたみ式 (オーナー判定 M4)。既定は閉じて魔法陣・関数一覧にスペースを譲る。
       開閉はローカル UI 状態で URL には載せない (状態というより作業中の道具箱)。 -->
  <details px-3 py-2 text-sm>
    <summary cursor-pointer select-none text-xs font-bold text-gray-600>⚙ パレット</summary>
    <strong mt-2 block text-xs text-gray-500>式</strong>
    <div mt-1 flex gap-3>
      <label flex items-center gap-1>
        <input
          type="radio"
          name="style"
          :checked="palette.style === 'midchilda'"
          @change="palette.setStyle('midchilda')"
        />
        ミッドチルダ
      </label>
      <label flex items-center gap-1>
        <input
          type="radio"
          name="style"
          :checked="palette.style === 'belka'"
          @change="palette.setStyle('belka')"
        />
        ベルカ
      </label>
    </div>

    <strong mt-3 block text-xs text-gray-500>レイヤー</strong>
    <div v-for="layer in LAYERS" :key="layer" mt-1 flex items-center gap-2>
      <label flex flex-1 items-center gap-1>
        <input
          type="checkbox"
          :checked="palette.layers[layer].visible"
          @change="palette.setVisible(layer, ($event.target as HTMLInputElement).checked)"
        />
        {{ LAYER_LABELS[layer] }}
      </label>
      <input
        type="range"
        min="0"
        max="1"
        step="0.05"
        w-20
        :value="palette.layers[layer].opacity"
        :aria-label="`${LAYER_LABELS[layer]}の透明度`"
        @input="palette.setOpacity(layer, parseFloat(($event.target as HTMLInputElement).value))"
      />
    </div>
    <div mt-2 flex gap-2>
      <button
        border
        border-gray-300
        rounded
        px-2
        py-0.5
        hover:bg-gray-100
        @click="palette.showAll()"
      >
        全表示
      </button>
      <button
        border
        border-gray-300
        rounded
        px-2
        py-0.5
        hover:bg-gray-100
        @click="palette.hideAll()"
      >
        全非表示
      </button>
    </div>

    <strong mt-3 block text-xs text-gray-500>Spell Diff (基準 rev)</strong>
    <div mt-1 flex items-center gap-1>
      <input
        v-model="diffInput"
        type="text"
        placeholder="HEAD~1 / main ..."
        aria-label="diff 基準リビジョン"
        w-full
        border
        border-gray-300
        rounded
        px-1
        py-0.5
        font-mono
        text-xs
        @keydown.enter="applyDiff(diffInput)"
      />
      <button
        border
        border-gray-300
        rounded
        px-2
        py-0.5
        text-xs
        hover:bg-gray-100
        @click="applyDiff(diffInput)"
      >
        比較
      </button>
    </div>
    <div mt-1 flex gap-2 text-xs>
      <button text-blue-700 underline @click="applyDiff('HEAD~1')">HEAD~1</button>
      <button text-blue-700 underline @click="applyDiff('main')">main</button>
      <button v-if="focus.diffRev" text-gray-500 underline @click="applyDiff('')">クリア</button>
    </div>
    <!-- 差分の要約 (正常時) / 案内 (rev 不正・新規関数など)。live diff:
         ファイル保存のたびに再計算され、書いている変更が金ハローで現れる -->
    <pre
      v-if="focus.diffRev && focus.spell?.diff_report"
      mt-1
      max-h-40
      overflow-auto
      whitespace-pre-wrap
      text-xs
      text-gray-600
      >{{ focus.spell.diff_report }}</pre
    >
    <div v-if="focus.diffRev && focus.spell?.diff_note" mt-1 text-xs text-effect-filesystem>
      {{ focus.spell.diff_note }}
    </div>

    <details mt-3>
      <summary cursor-pointer text-xs text-gray-600>.magia (spec §8)</summary>
      <textarea
        v-model="dslText"
        rows="3"
        spellcheck="false"
        mt-1
        w-full
        border
        border-gray-300
        rounded
        p-1
        font-mono
        text-xs
      />
      <div mt-1 flex gap-2>
        <button border border-gray-300 rounded px-2 py-0.5 hover:bg-gray-100 @click="onExport">
          エクスポート
        </button>
        <button border border-gray-300 rounded px-2 py-0.5 hover:bg-gray-100 @click="onApply">
          適用
        </button>
      </div>
      <div min-h-4 text-xs text-effect-filesystem>{{ dslNote }}</div>
    </details>
  </details>
</template>
