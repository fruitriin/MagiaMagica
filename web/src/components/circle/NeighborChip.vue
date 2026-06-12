<script setup lang="ts">
// 周辺関数のチップ (縮小盾、Phase 4.1)。円 + 関数名で「魔法陣の影」を表す。
// クリック/Enter でピン打ち直し (button 化により Tab/Enter はブラウザ標準動作)。
// 位置・スケール・透明度は親 (FocusView) が g の CSS transform で与える —
// このコンポーネントは原点中心にチップ1枚を描くだけ。
// Phase 4.4: フォーカスとの呼び出し関係 (relation) を向きマークで示し、
// ホバーで対応する召喚印と相互に光る (hoveredLink — 記号 ⇆ チップの地図対応)。
import { computed } from "vue";

import { useFocusStore } from "../../stores/focus.ts";
import type { NeighborChip } from "../../types/magia.ts";

const props = defineProps<{ chip: NeighborChip }>();
const emit = defineEmits<{ pin: [qualified: string] }>();

const focus = useFocusStore();

/** チップ内に収まるよう関数名を短縮する (詳細は title ツールチップで補完)。 */
const displayName = () => {
  const name = props.chip.name;
  return name.length > 12 ? `${name.slice(0, 11)}…` : name;
};

/** 呼び出し関係の向きマーク (フォーカス基準)。 */
const RELATION_MARK = {
  calls: { mark: "→", note: "フォーカスが呼ぶ" },
  called_by: { mark: "←", note: "フォーカスを呼ぶ" },
  mutual: { mark: "⇄", note: "相互に呼び合う" },
} as const;

const relation = computed(() =>
  props.chip.relation !== undefined ? RELATION_MARK[props.chip.relation] : null,
);

const tooltip = computed(() =>
  relation.value === null
    ? props.chip.signature
    : `${props.chip.signature}\n${relation.value.mark} ${relation.value.note}`,
);

/** 対応する召喚印側がホバーされている (双方向リンク強調)。 */
const isLinked = computed(() => focus.hoveredLink === props.chip.qualified);
</script>

<template>
  <g
    role="button"
    tabindex="0"
    cursor-pointer
    @click="emit('pin', chip.qualified)"
    @keydown.enter="emit('pin', chip.qualified)"
    @keydown.space.prevent="emit('pin', chip.qualified)"
    @mouseenter="focus.setHoveredLink(chip.qualified)"
    @mouseleave="focus.setHoveredLink(null)"
  >
    <title>{{ tooltip }}</title>
    <circle
      class="neighbor-chip"
      :class="{ 'link-highlight': isLinked }"
      :r="chip.radius"
      fill="#ffffff"
      stroke="#000000"
      :stroke-width="1.5"
    />
    <text
      text-anchor="middle"
      dominant-baseline="middle"
      font-size="11"
      font-family="ui-monospace, monospace"
      fill="#000000"
    >
      {{ displayName() }}
    </text>
    <!-- 呼び出し関係の向き (フォーカス基準、Phase 4.4)。チップ上端に小さく -->
    <text
      v-if="relation"
      class="relation-mark"
      text-anchor="middle"
      :y="-chip.radius + 14"
      font-size="12"
      fill="#00a0c0"
    >
      {{ relation.mark }}
    </text>
  </g>
</template>

<style scoped>
.neighbor-chip {
  transition: stroke 0.15s;
}
g:hover .neighbor-chip,
g:focus .neighbor-chip {
  stroke: #00a0c0;
  stroke-width: 2.5;
}
g:focus {
  outline: none;
}
/* 対応する召喚印のホバーに呼応するリンク強調 (Phase 4.4) */
.neighbor-chip.link-highlight {
  stroke: #00a0c0;
  stroke-width: 3;
}
.relation-mark {
  pointer-events: none;
}
</style>
