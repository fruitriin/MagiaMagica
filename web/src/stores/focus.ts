// 現在フォーカス中の関数と、その呪文 (魔法陣 SVG + メタ)。
// 関数の選択 → /spell 取得 → source store への分配までを担う
// (利用側が fetch と分配を組み立てる形にしない — POSD「複雑性を下に押し下げる」)。

import { defineStore } from "pinia";
import { computed, ref } from "vue";

import { fetchSpell, fetchState } from "../composables/api.ts";
import type { FunctionMeta, ServerError, SpellResponse } from "../types/magia.ts";
import { usePaletteStore } from "./palette.ts";
import { useSourceStore } from "./source.ts";

export const useFocusStore = defineStore("focus", () => {
  const file = ref<string | null>(null);
  const functions = ref<FunctionMeta[]>([]);
  /** 構文エラー。エラー中も last-good の魔法陣・ソースを表示し続ける (会話を切らない)。 */
  const serverError = ref<ServerError | null>(null);

  /** 現在の関数 (qualified 名 — impl メソッドは `Caster::cast` 形式)。 */
  const currentFn = ref<string | null>(null);
  const spell = ref<SpellResponse | null>(null);
  const loadError = ref<string | null>(null);

  const palette = usePaletteStore();

  /** 表示すべき SVG。様式の選択は palette store の管轄。 */
  const currentSvg = computed(() => {
    if (spell.value === null) return null;
    return palette.style === "belka" ? spell.value.svg_belka : spell.value.svg;
  });

  // ホバー/選択中の操作 (Phase 4.0.7)。id はスキーマの出現順 id (セッション内の
  // 一時識別子) なので、URL 等への永続化はしない。選択はファイル更新で構造が
  // 変わるとズレうる — IR 由来の安定参照は Phase 4.0.9 で検討する。
  const hoveredOperationId = ref<string | null>(null);
  const selectedOperationId = ref<string | null>(null);

  function hoverOperation(id: string | null) {
    hoveredOperationId.value = id;
  }

  function selectOperation(id: string | null) {
    selectedOperationId.value = id;
  }

  /**
   * URL (`?fn=`) 由来の初期希望値を置く。実ロードは SSE 接続直後イベント
   * (serve.rs が必ず1イベント流す) の refresh に一本化し、二重フェッチを避ける。
   */
  function setInitialFn(fn: string | null) {
    currentFn.value = fn;
  }

  /** `/state` を読み直して関数一覧を更新する。 */
  async function loadState() {
    const state = await fetchState();
    file.value = state.file;
    functions.value = state.functions;
    serverError.value = state.error;
  }

  /** 選択リクエストの世代。後発の選択があったら先発の応答を捨てる (競合防止)。 */
  let selectSeq = 0;

  /**
   * 関数を選択して呪文を取得する。`fn` が一覧にない場合や未指定の場合は先頭の関数。
   * 取得失敗時は直前の表示を保持する (エラー中も魔法陣を消さない — Phase 4.0 方針)。
   * 連打 (戻る/進むの連続) では最後に選んだ関数の応答だけを反映する。
   */
  async function selectFunction(fn: string | null) {
    const fallback = functions.value[0]?.qualified ?? null;
    const target = fn !== null && functions.value.some((f) => f.qualified === fn) ? fn : fallback;
    if (target === null) return;
    currentFn.value = target;
    selectSeq += 1;
    const seq = selectSeq;
    try {
      const next = await fetchSpell(target);
      if (seq !== selectSeq) return; // 古い応答 — 後発の選択が優先
      spell.value = next;
      loadError.value = null;
      useSourceStore().setSource(next.source_html, next.start_line);
    } catch (e) {
      if (seq !== selectSeq) return;
      loadError.value = e instanceof Error ? e.message : String(e);
    }
  }

  /** SSE 更新時の再読込。一覧と現在の呪文を取り直す。 */
  async function refresh() {
    // 出現順 id は再パースで再採番されるため、選択を持ち越すと別の操作に
    // ハローが移る誤挙動になる — 更新前にクリアする (安定参照は 4.0.9 で検討)。
    selectedOperationId.value = null;
    await loadState();
    await selectFunction(currentFn.value);
  }

  return {
    file,
    functions,
    serverError,
    currentFn,
    spell,
    loadError,
    currentSvg,
    hoveredOperationId,
    selectedOperationId,
    hoverOperation,
    selectOperation,
    setInitialFn,
    loadState,
    selectFunction,
    refresh,
  };
});
