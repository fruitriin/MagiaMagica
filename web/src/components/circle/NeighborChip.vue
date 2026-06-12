<script setup lang="ts">
// 周辺関数のチップ (縮小盾、Phase 4.1)。円 + 関数名で「魔法陣の影」を表す。
// クリック/Enter でピン打ち直し (button 化により Tab/Enter はブラウザ標準動作)。
// 位置・スケール・透明度は親 (FocusView) が g の CSS transform で与える —
// このコンポーネントは原点中心にチップ1枚を描くだけ。
import type { NeighborChip } from "../../types/magia.ts";

const props = defineProps<{ chip: NeighborChip }>();
const emit = defineEmits<{ pin: [qualified: string] }>();

/** チップ内に収まるよう関数名を短縮する (詳細は title ツールチップで補完)。 */
const displayName = () => {
  const name = props.chip.name;
  return name.length > 12 ? `${name.slice(0, 11)}…` : name;
};
</script>

<template>
  <g
    role="button"
    tabindex="0"
    cursor-pointer
    @click="emit('pin', chip.qualified)"
    @keydown.enter="emit('pin', chip.qualified)"
    @keydown.space.prevent="emit('pin', chip.qualified)"
  >
    <title>{{ chip.signature }}</title>
    <circle
      class="neighbor-chip"
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
</style>
