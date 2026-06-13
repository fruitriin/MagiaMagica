<script setup lang="ts">
// 実行順の出発点を示す入口サイン (Phase 4.0.6 後半)。メインリングの 3 時に
// 内向きの ▷ (右向き三角) を置き、上向きにごく短い反時計回りの円弧矢を添える —
// 「ここから上 (12時方向) へ反時計回りに読む」が一目で分かる。
// 視覚アフォーダンス専用 — クリック・ホバーは奪わない。
import { computed } from "vue";

import type { EntrySign } from "../../types/magia.ts";

const props = defineProps<{ sign: EntrySign }>();

/** 三角の頂点 (リング 3 時、内側を向く) と短い円弧の終点。 */
const geometry = computed(() => {
  const { x, y, radius } = props.sign;
  // 三角は内向きの ▷ — チップが内側、底辺がリングの内周直前。
  const tipInset = 4;
  const tipX = x + radius - 12;
  const baseX = x + radius - 4;
  const halfHeight = 5;
  // ごく短い反時計回り (上向き) の円弧 — 上端は 12 時 + 内側少し。
  const arcRadius = radius - tipInset;
  const start = { x: x + arcRadius, y };
  const end = {
    x: x + arcRadius * Math.cos(-Math.PI / 4),
    y: y + arcRadius * Math.sin(-Math.PI / 4),
  };
  return {
    triangle: `${tipX},${y} ${baseX},${y - halfHeight} ${baseX},${y + halfHeight}`,
    arc: `M ${start.x} ${start.y} A ${arcRadius} ${arcRadius} 0 0 0 ${end.x} ${end.y}`,
    arrowTip: end,
  };
});
</script>

<template>
  <g class="entry-sign">
    <!-- 内向き三角 = 「ここから始まる」 -->
    <polygon :points="geometry.triangle" fill="#000000" />
    <!-- 反時計回りの矢 = 「この向きに読む」 -->
    <path :d="geometry.arc" fill="none" stroke="#000000" stroke-width="1" />
    <polygon
      :points="`${geometry.arrowTip.x - 4},${geometry.arrowTip.y - 1} ${geometry.arrowTip.x},${geometry.arrowTip.y + 4} ${geometry.arrowTip.x + 1},${geometry.arrowTip.y - 1}`"
      fill="#000000"
    />
  </g>
</template>

<style scoped>
/* 装飾 — クリック/ホバー判定を奪わない (SVG 装飾規約)。 */
.entry-sign {
  pointer-events: none;
}
</style>
