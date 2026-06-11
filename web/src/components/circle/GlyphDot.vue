<script setup lang="ts">
// 召喚印 (外部呼び出し)。ホバーで輪郭強調、クリックで呼び出し先インスペクタを開く
// (Phase 4.1 — コード断片の表示と、同ファイル関数ならそこからのピン遷移)。
import { computed } from "vue";

import { useFocusStore } from "../../stores/focus.ts";
import type { EffectGlyph } from "../../types/magia.ts";

const props = defineProps<{ glyph: EffectGlyph }>();
const focus = useFocusStore();

const isHovered = computed(() => focus.hoveredOperationId === props.glyph.id);

function onClick(event: MouseEvent) {
  if (!props.glyph.selectable || props.glyph.callTarget === null) return;
  focus.inspectCall(props.glyph.callTarget, props.glyph.irId, event.clientX, event.clientY);
}
</script>

<template>
  <circle
    class="summon-glyph"
    :class="{ 'op-hovered': isHovered }"
    :cx="glyph.x"
    :cy="glyph.y"
    :r="glyph.radius"
    :fill="glyph.color"
    :style="glyph.selectable && glyph.callTarget !== null ? { cursor: 'pointer' } : {}"
    @mouseenter="glyph.selectable && focus.hoverOperation(glyph.id)"
    @mouseleave="glyph.selectable && focus.hoverOperation(null)"
    @click="onClick"
  />
</template>

<style scoped>
.op-hovered {
  stroke: #00a0c0;
  stroke-width: 2;
}
</style>
