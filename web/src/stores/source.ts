// ソースペインの状態 (細部修正 2026-06-12 でファイル全体表示に変更)。
// サーバが /state に同梱する「行ごとの SH 済み HTML」を保持し、
// フォーカス関数は行範囲の強調として表す (関数切替でソースは差し替えない)。

import { defineStore } from "pinia";
import { ref } from "vue";

export const useSourceStore = defineStore("source", () => {
  /** ファイル全体の行ごと SH 済み HTML (サーバ生成、信頼済み入力)。1要素 = 1行。 */
  const lines = ref<string[]>([]);
  /** フォーカス関数の行範囲 (1-origin、両端含む)。null = 強調なし。 */
  const focusStart = ref<number | null>(null);
  const focusEnd = ref<number | null>(null);

  function setLines(next: string[]) {
    lines.value = next;
  }

  function setFocusRange(start: number, end: number) {
    focusStart.value = start;
    focusEnd.value = end;
  }

  function clear() {
    lines.value = [];
    focusStart.value = null;
    focusEnd.value = null;
  }

  return { lines, focusStart, focusEnd, setLines, setFocusRange, clear };
});
