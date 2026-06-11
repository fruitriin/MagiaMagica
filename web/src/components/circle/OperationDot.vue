<script setup lang="ts">
// 操作ドット (リング上の術式1ステップ)。ホバー/選択を focus store に反映する —
// v-html 時代にはできなかったリアクティブ操作の最初の入口 (Phase 4.0.7)。
import { computed } from "vue";

import { useFocusStore } from "../../stores/focus.ts";
import type { Operation } from "../../types/magia.ts";

const props = defineProps<{ op: Operation }>();
const focus = useFocusStore();

const isHovered = computed(() => focus.hoveredOperationId === props.op.id);
const isSelected = computed(() => focus.selectedOperationId === props.op.id);

function onClick() {
  focus.selectOperation(isSelected.value ? null : props.op.id);
}
</script>

<template>
  <circle
    class="op-dot"
    :class="{ 'op-hovered': isHovered, 'op-selected': isSelected }"
    :cx="op.x"
    :cy="op.y"
    :r="op.radius"
    :fill="op.color"
    :style="op.selectable ? { cursor: 'pointer' } : {}"
    @mouseenter="focus.hoverOperation(op.id)"
    @mouseleave="focus.hoverOperation(null)"
    @click="op.selectable && onClick()"
  />
</template>

<style scoped>
/* ホバー/選択の強調は記号色 (効果カテゴリの色相規約) に触れず輪郭で表現する
   (Phase 3.2 のハロー意匠と同じ原則)。 */
.op-hovered {
  stroke: #00a0c0;
  stroke-width: 2;
}
.op-selected {
  stroke: #d4a017;
  stroke-width: 2.5;
}
</style>
