<script setup lang="ts">
// ソースペイン (細部修正 2026-06-12 でファイル全体表示に変更)。
// syntect でハイライト済みの行 HTML (サーバ生成、信頼済み入力) を行番号つきで
// 並べ、フォーカス関数の範囲を強調する。関数を切り替えてもソースは動かず、
// 強調とスクロール位置だけが移る。
// Cmd+クリック (Win/Linux は Ctrl) でクリックした語を関数名として解決し、
// 魔法陣のピンを切り替える (関数リストと同じ ?pin= push 経路)。
import { nextTick, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";

import { useFocusStore } from "../stores/focus.ts";
import { useSourceStore } from "../stores/source.ts";

const source = useSourceStore();
const focus = useFocusStore();
const route = useRoute();
const router = useRouter();

const container = ref<HTMLElement | null>(null);

function isFocused(line: number) {
  return (
    source.focusStart !== null &&
    line >= source.focusStart &&
    line <= (source.focusEnd ?? source.focusStart)
  );
}

// フォーカス範囲の変更で先頭行を視界へ (関数切替・初回ロード)。
watch(
  () => [source.focusStart, source.lines.length] as const,
  async ([line, total]) => {
    // setLines と setFocusRange は別々に届く — 範囲が行数を超えている瞬間
    // (ファイル切替直後) はスクロールしない (レビュー S2)。
    if (line === null || total === 0 || line > total) return;
    await nextTick();
    container.value?.querySelector(`[data-line="${line}"]`)?.scrollIntoView({ block: "center" });
  },
  { immediate: true },
);

/** Cmd+クリックで関数切替。クリックされた span のテキストを識別子として扱い、
 *  resolveCall (チップ・召喚印と同じ3段照合) で qualified へ解決する。 */
function onClick(event: MouseEvent) {
  if (!(event.metaKey || event.ctrlKey)) return;
  const text = (event.target as HTMLElement).textContent?.trim() ?? "";
  // syntect の span は識別子単位になることが多いが、記号が混ざる場合に備えて
  // 識別子部分だけを取り出す (`Foo::bar(` → `Foo::bar`)。
  const match = /[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*/.exec(text);
  if (match === null) return;
  const resolved = focus.resolveCall(match[0]);
  if (resolved === null || resolved === focus.currentFn) return;
  void router.push({ query: { ...route.query, pin: resolved } });
}
</script>

<template>
  <section ref="container" overflow-auto @click="onClick">
    <div v-if="source.lines.length > 0" py-3 text-sm leading-relaxed>
      <!-- :key はインデックス固定 — 行内容 (html) はキーにできない
           (`}` だけの行など同一内容の行が普通に複数あるため)。 -->
      <div
        v-for="(html, i) in source.lines"
        :key="i"
        :data-line="i + 1"
        class="src-line"
        :class="{ 'src-focus': isFocused(i + 1) }"
      >
        <span class="src-no">{{ i + 1 }}</span>
        <!-- eslint-disable-next-line vue/no-v-html -->
        <span class="src-code" v-html="html" />
      </div>
    </div>
    <div v-else p-4 text-gray-400>ソースを読み込み中…</div>
  </section>
</template>

<style scoped>
.src-line {
  display: flex;
  white-space: pre;
  font-family: ui-monospace, monospace;
  border-left: 2px solid transparent;
  padding-right: 0.75em;
}
.src-no {
  width: 3.5em;
  flex-shrink: 0;
  padding-right: 0.75em;
  text-align: right;
  color: #c0c0c0;
  user-select: none;
}
/* フォーカス関数の範囲強調 — 魔法陣でピン中の関数がここ、という対応付け */
.src-focus {
  background: #eef6fb;
  border-left-color: #00a0c0;
}
</style>
