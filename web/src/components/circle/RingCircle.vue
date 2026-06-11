<script setup lang="ts">
// 中央魔法陣 / 補助リングの円 (Rust レンダラの main-ring / aux-ring と画素等価)。
// 補助リングはホバーでガード・ヘッダの断片 (`if cond` 等) をプレビューする
// (Phase 4.1 追加要望4)。fill は transparent — 線上 1.5px では狙えないため
// リング面全体をホバー判定にする。内部のドット・召喚印は後に描画される
// (= 上のレイヤー) ので、個別のホバーがリングより優先される。
import { useFocusStore } from "../../stores/focus.ts";
import type { Circle } from "../../types/magia.ts";

const props = defineProps<{ circle: Circle }>();
const focus = useFocusStore();

function onEnter(event: MouseEvent) {
  if (props.circle.irId === null) return;
  const html = focus.spell?.ring_excerpts[String(props.circle.irId)] ?? null;
  if (html !== null) {
    focus.showHoverExcerpt(html, event.clientX, event.clientY, false);
  }
}

function onLeave() {
  if (props.circle.irId === null) return;
  focus.hideHoverExcerpt();
}
</script>

<template>
  <circle
    :class="circle.role === 'main' ? 'main-ring' : 'aux-ring'"
    :cx="circle.x"
    :cy="circle.y"
    :r="circle.radius"
    fill="transparent"
    stroke="#000000"
    :stroke-width="circle.strokeWidth"
    @mouseenter="onEnter"
    @mouseleave="onLeave"
  />
</template>
