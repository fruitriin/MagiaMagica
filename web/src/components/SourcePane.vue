<script setup lang="ts">
// ソースペイン。syntect でハイライト済みの HTML (サーバ生成、信頼済み入力) を
// 素通しする。スタイルは syntect のインラインスタイルに任せ、ここでは器だけ持つ。
import { useSourceStore } from "../stores/source.ts";

const source = useSourceStore();
</script>

<template>
  <section overflow-auto>
    <div v-if="source.sourceHtml" p-3 text-sm leading-relaxed v-html="source.sourceHtml" />
    <div v-else p-4 text-gray-400>ソースを読み込み中…</div>
  </section>
</template>
