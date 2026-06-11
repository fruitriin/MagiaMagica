<script setup lang="ts">
// 召喚印インスペクタ (Phase 4.1)。召喚印クリックで「呼び出し式」と
// 呼び出し先のコードをポップオーバー表示する。呼び出し式はレシーバ・引数込みの
// 式全体 (改行込み原文、サーバ切り出し) — `.map` のようなメソッド召喚印でも
// `sigil.map(|role| role.kind)` の形で文脈が見える (オーナー要望 2026-06-12)。
// 同ファイルの関数に解決できれば定義コード断片 (syntect HTML) も出し、
// クリックでピン遷移 — 外周にピン用シンボルを別建てせず、図の中の「呼び出し」
// からそのまま潜れる (オーナー要望 2026-06-11。厳密な呼び出し解決は Phase 4.4)。
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";

import { fetchSpell } from "../composables/api.ts";
import { useFocusStore } from "../stores/focus.ts";

const focus = useFocusStore();
const route = useRoute();
const router = useRouter();

const resolved = computed(() => {
  if (focus.inspectedCall === null) return null;
  return focus.resolveCall(focus.inspectedCall.callTarget);
});

/** 呼び出し式 (サーバ切り出し済み HTML)。span が取れなかった召喚印では null。 */
const excerptHtml = computed(() => {
  const call = focus.inspectedCall;
  if (call === null) return null;
  return focus.spell?.call_excerpts[String(call.glyphIrId)] ?? null;
});

/** 解決先のコード断片 (syntect HTML)。インスペクタを開くたびに取得する。 */
const sourceHtml = ref<string | null>(null);
watch(
  () => [focus.inspectedCall, resolved.value] as const,
  async ([call, qualified]) => {
    sourceHtml.value = null;
    if (call === null || qualified === null) return;
    try {
      const spell = await fetchSpell(qualified);
      // 開いている間に別の召喚印へ移っていたら捨てる
      if (focus.inspectedCall?.callTarget === call.callTarget) {
        sourceHtml.value = spell.source_html;
      }
    } catch {
      sourceHtml.value = null;
    }
  },
  { immediate: true },
);

/** クリック地点の近くに出す (画面端では内側に寄せる)。 */
const popoverStyle = computed(() => {
  const call = focus.inspectedCall;
  if (call === null) return {};
  const x = Math.min(call.clientX + 12, window.innerWidth - 440);
  const y = Math.min(call.clientY + 12, window.innerHeight - 200);
  return { left: `${Math.max(8, x)}px`, top: `${Math.max(8, y)}px` };
});

function pinResolved() {
  if (resolved.value === null) return;
  const target = resolved.value;
  focus.closeInspector();
  if (target === focus.currentFn) return;
  void router.push({ query: { ...route.query, pin: target } });
}

// 外側クリック / Esc で閉じる。薄幕 (全画面の透明 div) は使わない —
// 固定ポップオーバーを出したまま他のドットへホバー/クリックできるようにする
// (オーナー要望 2026-06-12: ホバープレビューは固定の上に重なる)。
// ポップオーバー内と召喚印クリックは stopPropagation でここに届かない。
function onWindowClick() {
  if (focus.inspectedCall !== null) focus.closeInspector();
}
function onWindowKeydown(event: KeyboardEvent) {
  if (event.key === "Escape" && focus.inspectedCall !== null) focus.closeInspector();
}
onMounted(() => {
  window.addEventListener("click", onWindowClick);
  window.addEventListener("keydown", onWindowKeydown);
});
onUnmounted(() => {
  window.removeEventListener("click", onWindowClick);
  window.removeEventListener("keydown", onWindowKeydown);
});
</script>

<template>
  <Teleport to="body">
    <template v-if="focus.inspectedCall">
      <!-- クリック固定のポップオーバー (z-50)。ホバープレビュー (z-60) が上に重なる。
           内側クリックは window の「外側クリックで閉じる」に拾わせない -->
      <div
        fixed
        z-50
        max-w-md
        rounded-lg
        border
        border-gray-300
        bg-white
        p-3
        shadow-lg
        text-sm
        :style="popoverStyle"
        role="dialog"
        aria-label="呼び出し先"
        @click.stop
      >
        <div flex items-baseline justify-between gap-3>
          <code text-xs font-bold>{{ focus.inspectedCall.callTarget }}</code>
          <button text-xs text-gray-400 hover:text-gray-700 @click="focus.closeInspector()">
            ✕
          </button>
        </div>

        <!-- 呼び出し式 (レシーバ・引数込み、改行込み原文)。解決の成否に関わらず出す -->
        <div v-if="excerptHtml" mt-2>
          <div text-xs text-gray-400>呼び出し式</div>
          <div
            class="call-excerpt"
            mt-1
            max-h-48
            overflow-auto
            rounded
            border
            border-gray-200
            text-xs
            leading-relaxed
            p-2
            v-html="excerptHtml"
          />
        </div>

        <template v-if="resolved">
          <!-- コード断片クリックでピン (縦可変・大きめ。オーナー要望) -->
          <div v-if="excerptHtml" mt-2 text-xs text-gray-400>定義</div>
          <div
            mt-1
            max-h-96
            cursor-pointer
            overflow-auto
            rounded
            border
            border-blue-200
            text-xs
            leading-relaxed
            hover:border-blue-400
            :title="`クリックで ${resolved} をピン`"
            @click="pinResolved"
          >
            <div v-if="sourceHtml" p-2 v-html="sourceHtml" />
            <div v-else p-2 text-gray-400>コードを読み込み中…</div>
          </div>
          <button
            mt-2
            w-full
            rounded
            border
            border-blue-300
            bg-blue-50
            px-2
            py-1
            text-xs
            text-blue-800
            hover:bg-blue-100
            @click="pinResolved"
          >
            📌 {{ resolved }} をピン
          </button>
        </template>
        <div v-else mt-2 text-xs text-gray-500>
          このファイル内に定義がない外部呼び出しです (クレート横断は Phase 4.5+)
        </div>
      </div>
    </template>
  </Teleport>
</template>
