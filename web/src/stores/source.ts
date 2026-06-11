// フォーカス中の関数のソース表示 (syntect ハイライト済み HTML)。
// M3 の <SourcePane> がここを参照する。M2 では置き場だけ用意する。

import { defineStore } from "pinia";
import { ref } from "vue";

export const useSourceStore = defineStore("source", () => {
  /** サーバ生成のハイライト済み HTML。null = 未ロード。 */
  const sourceHtml = ref<string | null>(null);
  /** 元ファイル内の開始行 (1-origin)。行番号表示と ?fn= ジャンプに使う。 */
  const startLine = ref<number | null>(null);

  function setSource(html: string, line: number) {
    sourceHtml.value = html;
    startLine.value = line;
  }

  function clear() {
    sourceHtml.value = null;
    startLine.value = null;
  }

  return { sourceHtml, startLine, setSource, clear };
});
