// 現在フォーカス中の関数と、その呪文 (配置済み IR + ベルカ SVG + メタ)。
// 関数の選択 → /spell 取得 → source store への分配までを担う
// (利用側が fetch と分配を組み立てる形にしない — POSD「複雑性を下に押し下げる」)。

import { defineStore } from "pinia";
import { ref } from "vue";

import { fetchFiles, fetchSpell, fetchState, postFile } from "../composables/api.ts";
import type { FunctionMeta, ServerError, SpellResponse } from "../types/magia.ts";
import { useSourceStore } from "./source.ts";

export const useFocusStore = defineStore("focus", () => {
  const file = ref<string | null>(null);
  /** 切替候補 (.rs 一覧、Phase 4.4.5)。loadFiles で遅延取得。 */
  const files = ref<string[]>([]);
  /** URL `?file=` 由来の希望ファイル。サーバの採用値 (POST 応答) で正規化する。 */
  const requestedFile = ref<string | null>(null);
  const functions = ref<FunctionMeta[]>([]);
  /** 構文エラー。エラー中も last-good の魔法陣・ソースを表示し続ける (会話を切らない)。 */
  const serverError = ref<ServerError | null>(null);

  /** 現在の関数 (qualified 名 — impl メソッドは `Caster::cast` 形式)。 */
  const currentFn = ref<string | null>(null);
  /** Spell Diff の比較基準 rev (`?diff=` と同期 — Phase 4.3.7)。null = diff なし。 */
  const diffRev = ref<string | null>(null);
  const spell = ref<SpellResponse | null>(null);
  const loadError = ref<string | null>(null);

  // ホバー/選択中の操作。id はスキーマ内の一時識別子 (ring id + 出現順) で、
  // SigilId 同様パースごとに変わりうるため URL 等への永続化はしない
  // (Phase 4.0.9 で検討した結論: 安定参照は IR にも存在しない — spec §16)。
  const hoveredOperationId = ref<string | null>(null);
  const selectedOperationId = ref<string | null>(null);

  function hoverOperation(id: string | null) {
    hoveredOperationId.value = id;
  }

  function selectOperation(id: string | null) {
    selectedOperationId.value = id;
  }

  // 召喚印インスペクタ (Phase 4.1): 召喚印クリックで呼び出し式と呼び出し先の
  // コードをポップオーバー表示し、同ファイル関数ならそこからピンできる。
  // glyphIrId は呼び出し式 (spell.call_excerpts) の引き当てに使う。
  // 画面座標はポップオーバーの表示位置 (クリック地点) に使う。
  const inspectedCall = ref<{
    callTarget: string;
    glyphIrId: number;
    clientX: number;
    clientY: number;
  } | null>(null);

  function inspectCall(callTarget: string, glyphIrId: number, clientX: number, clientY: number) {
    inspectedCall.value = { callTarget, glyphIrId, clientX, clientY };
  }

  function closeInspector() {
    inspectedCall.value = null;
  }

  // ホバープレビュー (Phase 4.1 追加要望3): 召喚印・操作ドットのホバーで
  // 原文断片を読み専用ポップオーバー表示する。クリック固定 (inspectedCall) とは
  // 独立の層で、固定の上に重なる (z: ホバー > 固定 — オーナー指定)。
  // pinnable はヒント文言用 (召喚印 = クリックで固定してピン操作ができる)。
  const hoverExcerpt = ref<{
    html: string;
    clientX: number;
    clientY: number;
    pinnable: boolean;
  } | null>(null);

  function showHoverExcerpt(html: string, clientX: number, clientY: number, pinnable: boolean) {
    hoverExcerpt.value = { html, clientX, clientY, pinnable };
  }

  function hideHoverExcerpt() {
    hoverExcerpt.value = null;
  }

  // 呼び出しリンクのホバー強調 (Phase 4.4)。召喚印とその解決先チップは
  // 同じ qualified を共有する — どちらをホバーしても1つの状態で双方が光る。
  const hoveredLink = ref<string | null>(null);

  function setHoveredLink(qualified: string | null) {
    hoveredLink.value = qualified;
  }

  /** 呼び出し名を同ファイルの関数に解決する (`charge` → `Wand::charge` など)。
   *  名前一致が複数あるときは先頭 (定義順) — 厳密な解決は Phase 4.4 で。 */
  function resolveCall(callTarget: string): string | null {
    const plain = callTarget.replace(/^\./, "").replace(/!$/, "");
    const hit = functions.value.find((f) => f.qualified === plain || f.name === plain);
    return hit?.qualified ?? null;
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

  /** 切替候補 (.rs 一覧) を取得する。 */
  async function loadFiles() {
    try {
      files.value = (await fetchFiles()).files;
    } catch {
      files.value = [];
    }
  }

  /** 監視対象ファイルを切り替える (Phase 4.4.5)。サーバの採用パスで requestedFile を
   *  正規化し、以後の同値比較 (URL 同期・refresh) のループを断つ。実際の一覧・図の
   *  更新は SSE (reload が必ず1イベント流す) の refresh に任せる。 */
  async function switchFile(path: string) {
    if (path === file.value) return;
    try {
      requestedFile.value = await postFile(path);
      loadError.value = null;
    } catch (e) {
      loadError.value = e instanceof Error ? e.message : String(e);
    }
  }

  /** URL (`?file=`) 由来の希望値を置く。切替の発火は useQuerySync の watch が担う
   *  (サーバの file が確定する SSE 初回より前に URL が読まれるため、値とトリガを分離)。 */
  function setRequestedFile(path: string | null) {
    requestedFile.value = path;
  }

  /** 選択リクエストの世代。後発の選択があったら先発の応答を捨てる (競合防止)。
   *  ref にしてストアの再生成 (テストの setActivePinia 等) とライフサイクルを揃える。 */
  const selectSeq = ref(0);

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
    selectSeq.value += 1;
    const seq = selectSeq.value;
    try {
      const next = await fetchSpell(target, diffRev.value);
      if (seq !== selectSeq.value) return; // 古い応答 — 後発の選択が優先
      spell.value = next;
      loadError.value = null;
      // 図が差し替わるとホバー元のドットが消えて mouseleave が来ないことがある
      hoverExcerpt.value = null;
      useSourceStore().setSource(next.source_html, next.start_line);
    } catch (e) {
      if (seq !== selectSeq.value) return;
      loadError.value = e instanceof Error ? e.message : String(e);
    }
  }

  /** diff 基準 rev を設定して現在の関数を取り直す (同値なら何もしない)。 */
  async function setDiffRev(rev: string | null) {
    const normalized = rev !== null && rev.trim() !== "" ? rev.trim() : null;
    if (normalized === diffRev.value) return;
    diffRev.value = normalized;
    await selectFunction(currentFn.value);
  }

  /** SSE 更新時の再読込。一覧と現在の呪文を取り直す。 */
  async function refresh() {
    // id は再パースで再採番されるため、選択を持ち越すと別の操作に
    // ハローが移る誤挙動になる — 更新前にクリアする。
    // インスペクタ (inspectedCall) はクリアしない (SSE 更新のたびに
    // ポップオーバーが閉じる誤挙動になる)。呼び出し名は再採番の影響を
    // 受けないが、glyphIrId は受けるため取得後に付け替える (下記)。
    selectedOperationId.value = null;
    await loadState();
    await selectFunction(currentFn.value);
    // 開いているインスペクタの glyph id を新しい呪文に付け替える。
    // 同 id が同名のまま残っていればそのまま。消えていたら同名の召喚印が
    // 一意のときだけ移し、曖昧 (同名複数) なら式の取り違えを避けて閉じる。
    const call = inspectedCall.value;
    if (call === null) return;
    const matches = spell.value?.ir.glyphs.filter((g) => g.call_target === call.callTarget) ?? [];
    if (matches.some((g) => g.id === call.glyphIrId)) return;
    const only = matches.length === 1 ? matches[0] : undefined;
    inspectedCall.value = only ? { ...call, glyphIrId: only.id } : null;
  }

  return {
    file,
    files,
    requestedFile,
    functions,
    serverError,
    currentFn,
    diffRev,
    spell,
    loadError,
    hoveredOperationId,
    selectedOperationId,
    hoverOperation,
    selectOperation,
    inspectedCall,
    inspectCall,
    closeInspector,
    hoverExcerpt,
    showHoverExcerpt,
    hideHoverExcerpt,
    hoveredLink,
    setHoveredLink,
    resolveCall,
    setDiffRev,
    setInitialFn,
    loadState,
    loadFiles,
    switchFile,
    setRequestedFile,
    selectFunction,
    refresh,
  };
});
