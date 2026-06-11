<script setup lang="ts">
// ホバープレビュー (Phase 4.1 追加要望3)。召喚印・操作ドットのホバーで
// 原文断片 (syntect HTML) を読み専用表示する。クリック固定 (CallInspector) より
// 上の層 (z-60 > z-50 — オーナー指定「ホバーのほうが上」)。
// pointer-events: none — プレビュー自身がマウスを奪って mouseleave が
// 発火しなくなる (チラつき) のを防ぐ。
import { computed } from "vue";

import { useFocusStore } from "../stores/focus.ts";

const focus = useFocusStore();

/** クリック地点の近くに出す (画面端では内側に寄せる — CallInspector と同規則)。 */
const previewStyle = computed(() => {
  const hover = focus.hoverExcerpt;
  if (hover === null) return {};
  const x = Math.min(hover.clientX + 12, window.innerWidth - 440);
  const y = Math.min(hover.clientY + 12, window.innerHeight - 160);
  return { left: `${Math.max(8, x)}px`, top: `${Math.max(8, y)}px` };
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="focus.hoverExcerpt"
      class="hover-preview"
      fixed
      z-60
      max-w-md
      rounded-lg
      border
      border-gray-300
      bg-white
      p-2
      shadow-lg
      pointer-events-none
      :style="previewStyle"
    >
      <div max-h-48 overflow-hidden text-xs leading-relaxed v-html="focus.hoverExcerpt.html" />
      <div v-if="focus.hoverExcerpt.pinnable" mt-1 text-xs text-gray-400>クリックで固定</div>
    </div>
  </Teleport>
</template>
