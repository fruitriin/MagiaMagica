<script setup lang="ts">
// 関数目次 (Phase 4.1 でピン中心ビューに対応)。
// 「表示中のみ」フィルタ ON のときはフォーカス + 周辺を距離順に列挙し、
// シグネチャを添える (計画: 関数目次の代わりに周辺シグネチャを距離順で)。
// OFF のときは従来どおり全関数を定義順で列挙する。
// 選択は URL (?pin=) への push だけ行い、実際のロードは useQuerySync の
// query watch が担う (URL を唯一の状態源にする一方向ループ)。
import { computed, ref } from "vue";
import { useRoute, useRouter } from "vue-router";

import { useFocusStore } from "../stores/focus.ts";

const focus = useFocusStore();
const route = useRoute();
const router = useRouter();

/** 「表示中 (ピン + 周辺) のみ」フィルタ。既定 ON (ピンビューが標準のため)。 */
const pinnedOnly = ref(true);

type TocEntry = {
  qualified: string;
  label: string;
  title: string;
  distance: number | null;
};

const entries = computed<TocEntry[]>(() => {
  const layout = focus.spell?.focus_layout;
  if (pinnedOnly.value && layout && focus.currentFn !== null) {
    // 並びは常に定義順 (focus.functions の順) — ピンを切り替えても行が
    // 入れ替わらない (オーナー指示 2026-06-12)。距離は title に退避。
    const distanceOf = new Map<string, number>(
      layout.neighbors.map((chip) => [chip.qualified, chip.distance]),
    );
    return focus.functions
      .filter((f) => f.qualified === focus.currentFn || distanceOf.has(f.qualified))
      .map((f) => ({
        qualified: f.qualified,
        label: f.signature,
        title: f.qualified === focus.currentFn ? "ピン中" : `距離 ${distanceOf.get(f.qualified)}`,
        distance: distanceOf.get(f.qualified) ?? 0,
      }));
  }
  return focus.functions.map((f) => ({
    qualified: f.qualified,
    label: f.qualified,
    title: f.signature,
    distance: null,
  }));
});

function select(qualified: string) {
  // 同一関数の再クリックは何もしない (UX 確定)。
  if (qualified === focus.currentFn) return;
  void router.push({ query: { ...route.query, pin: qualified } });
}
</script>

<template>
  <nav flex min-h-0 flex-col>
    <div flex items-center justify-between px-3 pt-3 pb-1>
      <h2 text-xs font-bold text-gray-500>関数</h2>
      <label flex items-center gap-1 text-xs text-gray-500>
        <input v-model="pinnedOnly" type="checkbox" />
        表示中のみ
      </label>
    </div>
    <!-- ファイルパスツリー表示は複数ファイル対応 (Phase 4.5) と合流して実装する。
         トグルの置き場だけ確保 (計画指定の UI 骨格)。 -->
    <label flex items-center gap-1 px-3 pb-1 text-xs text-gray-300 cursor-not-allowed>
      <input type="checkbox" disabled />
      ツリー表示 (複数ファイル対応後)
    </label>
    <ul min-h-0 flex-1 overflow-y-auto>
      <li v-for="entry in entries" :key="entry.qualified">
        <button
          w-full
          truncate
          px-3
          py-1
          text-left
          text-xs
          font-mono
          :class="
            entry.qualified === focus.currentFn
              ? 'bg-blue-50 text-blue-800 font-bold'
              : 'text-gray-700 hover:bg-gray-100'
          "
          :title="entry.title"
          @click="select(entry.qualified)"
        >
          {{ entry.label }}
        </button>
      </li>
    </ul>
  </nav>
</template>
