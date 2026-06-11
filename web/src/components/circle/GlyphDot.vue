<script setup lang="ts">
// 召喚印 (外部呼び出し)。ホバーで輪郭強調 — クリックでの呼び出しジャンプは Phase 4.4。
import { computed } from "vue";

import { useFocusStore } from "../../stores/focus.ts";
import type { EffectGlyph } from "../../types/magia.ts";

const props = defineProps<{ glyph: EffectGlyph }>();
const focus = useFocusStore();

const isHovered = computed(() => focus.hoveredOperationId === props.glyph.id);
</script>

<template>
  <circle
    class="summon-glyph"
    :class="{ 'op-hovered': isHovered }"
    :cx="glyph.x"
    :cy="glyph.y"
    :r="glyph.radius"
    :fill="glyph.color"
    @mouseenter="focus.hoverOperation(glyph.id)"
    @mouseleave="focus.hoverOperation(null)"
  />
</template>

<style scoped>
.op-hovered {
  stroke: #00a0c0;
  stroke-width: 2;
}
</style>
