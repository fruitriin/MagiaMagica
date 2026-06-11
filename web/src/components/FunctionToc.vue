<script setup lang="ts">
// 関数目次。選択は URL (?fn=) への push だけ行い、実際のロードは
// HomeView の query watch が担う (URL を唯一の状態源にする一方向ループ)。
// push なのでブラウザの戻る/進むで選択履歴を辿れる。
import { useRoute, useRouter } from "vue-router";

import { useFocusStore } from "../stores/focus.ts";

const focus = useFocusStore();
const route = useRoute();
const router = useRouter();

function select(qualified: string) {
  // 同一関数の再クリックは何もしない (UX 確定)。「再クリックでリセット」のような
  // 仕様を入れる場合はこの早期 return を外すことになる。
  if (qualified === focus.currentFn) return;
  void router.push({ query: { ...route.query, fn: qualified } });
}
</script>

<template>
  <nav overflow-y-auto>
    <h2 px-3 pt-3 pb-1 text-xs font-bold text-gray-500>関数</h2>
    <ul>
      <li v-for="f in focus.functions" :key="f.qualified">
        <button
          w-full
          truncate
          px-3
          py-1
          text-left
          text-sm
          font-mono
          :class="
            f.qualified === focus.currentFn
              ? 'bg-blue-50 text-blue-800 font-bold'
              : 'text-gray-700 hover:bg-gray-100'
          "
          :title="f.signature"
          @click="select(f.qualified)"
        >
          {{ f.qualified }}
        </button>
      </li>
    </ul>
  </nav>
</template>
