<script setup lang="ts">
// WASM デモ版のルート (Phase 4.12 M2)。serve の App.vue とは別物 —
// SSE / router / focus store / 俯瞰を持たず、ソース文字列 → list → spell を
// wasm で完結させて単一の魔法陣を描く。描画は serve と同じ circle/* を流用する。
import { computed, onMounted, ref, shallowRef } from "vue";

import { irToSchema } from "../converters/irToSchema.ts";
import { usePaletteStore } from "../stores/palette.ts";
import type { FunctionMeta, HobbySpellResponse } from "../types/magia.ts";
import BelkaCircle from "../components/circle/BelkaCircle.vue";
import MagicCircle from "../components/circle/MagicCircle.vue";
import SymbolLegend from "../components/SymbolLegend.vue";
import type { WasmDataSource } from "./dataSource.ts";
import { loadWasmDataSource } from "./wasmLoader.ts";
import { PREFILL_FN, PREFILL_SOURCE } from "./examples.ts";
import { buildShareUrl, parseShareHash } from "./share.ts";

const palette = usePaletteStore();

const source = ref(PREFILL_SOURCE);
const functions = ref<FunctionMeta[]>([]);
const currentFn = ref<string | null>(null);
const spell = shallowRef<HobbySpellResponse | null>(null);
const error = ref<string | null>(null);
const ready = ref(false);
const copied = ref(false);

let dataSource: WasmDataSource | null = null;

const schema = computed(() => {
  if (spell.value === null || palette.style === "belka") return null;
  return irToSchema(spell.value.ir);
});
const belkaIr = computed(() =>
  palette.style === "belka" ? (spell.value?.belka_ir ?? null) : null,
);

/** ソースを解析して関数一覧を更新し、可能なら関数を選び直して描画する。 */
function analyze(preferFn?: string): void {
  if (dataSource === null) return;
  error.value = null;
  let listed: FunctionMeta[];
  try {
    listed = dataSource.list(source.value);
  } catch (e) {
    // 構文エラー中も直前の正常な魔法陣・関数一覧を保持する (serve の
    // last-good スナップショット配信と同じ規約、spec §7) — 書きかけの
    // 一時的な壊れで、見比べていた陣が消えないように。
    error.value = e instanceof Error ? e.message : String(e);
    return;
  }
  functions.value = listed;
  if (listed.length === 0) {
    currentFn.value = null;
    spell.value = null;
    error.value = "描画できる関数が見つかりませんでした (Rust の fn が要ります)";
    return;
  }
  // 優先指定 → 現在選択の維持 → 先頭、の順で対象を決める。
  const wanted =
    (preferFn !== undefined && listed.some((f) => f.qualified === preferFn) && preferFn) ||
    (currentFn.value !== null &&
      listed.some((f) => f.qualified === currentFn.value) &&
      currentFn.value) ||
    listed[0]!.qualified;
  render(wanted);
}

/** 指定関数の魔法陣を描く。 */
function render(fn: string): void {
  if (dataSource === null) return;
  error.value = null;
  try {
    spell.value = dataSource.spell(source.value, fn);
    currentFn.value = fn;
  } catch (e) {
    // last-good 保持 (analyze と同じ規約): 直前の陣と選択はそのまま、案内だけ出す。
    error.value = e instanceof Error ? e.message : String(e);
  }
}

function onPickFile(event: Event): void {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  // 値を空に戻す — 同じファイルの再選択でも change が発火するように
  // (ネイティブ file input は同一パスの再選択でイベントを出さない)。
  input.value = "";
  if (!file) return;
  if (dataSource === null) {
    error.value = "WASM を読み込み中です。少し待ってからもう一度お試しください";
    return;
  }
  // 読み込み失敗 (権限・破損) も握り潰さず error に畳む (レビュー W1)。
  void file.text().then(
    (text) => {
      source.value = text;
      analyze();
    },
    (e: unknown) => {
      error.value = `ファイルの読み込みに失敗しました: ${e instanceof Error ? e.message : String(e)}`;
    },
  );
}

/** 現在のソース + 選択関数 + 表示式を共有 URL にしてクリップボードへ。URL バーにも反映する。 */
async function shareLink(): Promise<void> {
  error.value = null; // 前回の失敗案内を持ち越さない (成功表示と矛盾するため)。
  const base = window.location.origin + window.location.pathname;
  // style は非既定 (ベルカ式) のときだけ載せる — 受け手が同じ見た目で開けるように。
  const style = palette.style === "belka" ? "belka" : null;
  const url = buildShareUrl(base, source.value, currentFn.value, style);
  // URL バーへ反映 (リロードでこの状態が復元される)。replaceState は hashchange を
  // 発火しない — 将来 hashchange/popstate を購読しても誤発火しない (レビュー W1)。
  window.history.replaceState(null, "", url.slice(url.indexOf("#")));
  if (navigator.clipboard === undefined) {
    error.value = "この環境ではクリップボードを使えません。URL バーからコピーしてください";
    return;
  }
  try {
    await navigator.clipboard.writeText(url);
    copied.value = true;
    window.setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch {
    error.value = "クリップボードにコピーできませんでした。URL バーからコピーしてください";
  }
}

onMounted(async () => {
  // 共有リンク (`#code=...`) があれば prefill より優先して復元する (Phase 4.12 M3)。
  const shared = parseShareHash(window.location.hash);
  if (shared.source !== undefined) source.value = shared.source;
  // 表示式も復元する (未知の値は黙って無視 — 将来の style 追加リンクを踏んでも壊さない)。
  if (shared.style === "belka" || shared.style === "midchilda") palette.setStyle(shared.style);
  try {
    dataSource = await loadWasmDataSource();
    ready.value = true;
    // 共有 fn 指定 → (共有 source なら先頭) → prefill 既定、の順で対象を決める。
    analyze(shared.fn ?? (shared.source === undefined ? PREFILL_FN : undefined));
  } catch (e) {
    error.value = `WASM の初期化に失敗しました: ${e instanceof Error ? e.message : String(e)}`;
  }
});
</script>

<template>
  <div min-h-screen flex flex-col bg-gray-50 text-gray-900>
    <header flex items-center gap-3 px-4 py-3 border-b bg-white>
      <strong text-lg>MagiaMagica</strong>
      <span text-sm text-gray-500
        >コードを貼ると魔法陣になるデモ — コードはブラウザから出ません</span
      >
    </header>

    <div flex flex-col gap-4 p-4 lg="flex-row">
      <!-- 入力ペイン -->
      <section flex flex-col gap-2 lg="w-2/5">
        <div flex items-center gap-2>
          <button
            type="button"
            px-3
            py-1
            rounded
            bg-indigo-600
            text-white
            text-sm
            :disabled="!ready"
            @click="analyze()"
          >
            描画
          </button>
          <label
            text-sm
            text-gray-600
            cursor-pointer
            :class="ready ? '' : 'opacity-50 pointer-events-none'"
          >
            <input
              type="file"
              accept=".rs"
              class="hidden"
              :disabled="!ready"
              @change="onPickFile"
            />
            <span underline>.rs ファイルを開く</span>
          </label>
          <span v-if="!ready" text-xs text-gray-400>WASM 読込中…</span>
        </div>
        <textarea
          v-model="source"
          spellcheck="false"
          w-full
          h-80
          p-3
          font-mono
          text-xs
          border
          rounded
          resize-y
          bg-white
          lg="h-130"
          placeholder="ここに Rust のコードを貼る"
        />
        <div v-if="functions.length > 0" flex items-center gap-2>
          <label text-sm text-gray-600 for="fn-pick">関数</label>
          <select
            id="fn-pick"
            :value="currentFn ?? ''"
            px-2
            py-1
            text-sm
            border
            rounded
            bg-white
            @change="render(($event.target as HTMLSelectElement).value)"
          >
            <option v-for="fn in functions" :key="fn.qualified" :value="fn.qualified">
              {{ fn.qualified }}
            </option>
          </select>
          <span flex-1 />
          <button
            v-if="spell"
            type="button"
            px-2
            py-1
            text-sm
            border
            rounded
            bg-white
            @click="() => void shareLink()"
          >
            {{ copied ? "コピーしました" : "共有リンクをコピー" }}
          </button>
        </div>
        <p v-if="error" text-sm text-red-600 font-mono>{{ error }}</p>
      </section>

      <!-- 魔法陣ペイン -->
      <section flex flex-col gap-3 lg="w-3/5">
        <div flex items-center gap-3>
          <span v-if="spell" font-mono text-sm text-gray-700>{{ spell.signature }}</span>
          <span flex-1 />
          <div flex gap-1 text-sm>
            <button
              type="button"
              px-2
              py-1
              rounded
              border
              :class="palette.style === 'midchilda' ? 'bg-indigo-600 text-white' : 'bg-white'"
              @click="palette.setStyle('midchilda')"
            >
              ミッドチルダ式
            </button>
            <button
              type="button"
              px-2
              py-1
              rounded
              border
              :class="palette.style === 'belka' ? 'bg-indigo-600 text-white' : 'bg-white'"
              @click="palette.setStyle('belka')"
            >
              ベルカ式
            </button>
          </div>
        </div>

        <div border rounded bg-white p-2 min-h-100 flex items-center justify-center>
          <MagicCircle v-if="schema" :schema="schema" w-full />
          <BelkaCircle v-else-if="belkaIr" :belka="belkaIr" w-full />
          <span v-else text-gray-400>魔法陣を待機中…</span>
        </div>

        <SymbolLegend />

        <!-- スクリーンリーダー向けの呪文書き起こし (spec §15)。 -->
        <div class="visually-hidden" role="region" aria-label="呪文書き起こし">
          {{ spell?.transcript ?? "" }}
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  clip-path: inset(50%);
  white-space: nowrap;
  border: 0;
}
</style>
