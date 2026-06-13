<script setup lang="ts">
// 実行順の出発点を示す入口サイン (Phase 4.0.6 後半)。メインリングの 3 時に
// 内向きの ▷ (右向き三角) を置き、上向きにごく短い反時計回りの円弧矢を添える —
// 「ここから上 (12時方向) へ反時計回りに読む」が一目で分かる。
// 視覚アフォーダンス専用 — クリック・ホバーは奪わない。
import { computed } from "vue";

import type { EntrySign } from "../../types/magia.ts";

const props = defineProps<{ sign: EntrySign }>();

/** 三角の頂点 (リング 3 時、内側を向く) と短い円弧の終点。
 *  矢じりは判定で「もっと大きく」要望 — 円弧本体は控えめのまま、矢じりだけ
 *  しっかり見えるサイズ (オーナー判定 2026-06-13)。 */
const geometry = computed(() => {
  const { x, y, radius } = props.sign;
  // 三角は内向きの ▷ — チップが内側、底辺がリングの内周直前。控えめサイズ
  // (矢じりが十分目立てば三角は補助的でよい — 判定での示唆)。
  const tipInset = 4;
  const tipX = x + radius - 11;
  const baseX = x + radius - 4;
  const halfHeight = 4;
  // 反時計回り (上向き) の円弧 — 上端は 12 時 + 内側少し。
  const arcRadius = radius - tipInset;
  const start = { x: x + arcRadius, y };
  const end = {
    x: x + arcRadius * Math.cos(-Math.PI / 4),
    y: y + arcRadius * Math.sin(-Math.PI / 4),
  };
  // 矢じり: 円弧の接線方向に対して左右に開いた三角 (判定で強化要望)。
  // 接線方向は終点での円弧の進行方向 — 角度 -π/4 における接線 = 角度 (-π/4 + π/2)。
  const tangent = -Math.PI / 4 + Math.PI / 2; // 反時計回りの進行方向
  const arrowLen = 9;
  const arrowHalfWidth = 5;
  const back = {
    x: end.x - arrowLen * Math.cos(tangent),
    y: end.y - arrowLen * Math.sin(tangent),
  };
  const perpX = -Math.sin(tangent);
  const perpY = Math.cos(tangent);
  const wingA = {
    x: back.x + arrowHalfWidth * perpX,
    y: back.y + arrowHalfWidth * perpY,
  };
  const wingB = {
    x: back.x - arrowHalfWidth * perpX,
    y: back.y - arrowHalfWidth * perpY,
  };
  return {
    triangle: `${tipX},${y} ${baseX},${y - halfHeight} ${baseX},${y + halfHeight}`,
    arc: `M ${start.x} ${start.y} A ${arcRadius} ${arcRadius} 0 0 0 ${back.x} ${back.y}`,
    arrowhead: `${end.x},${end.y} ${wingA.x},${wingA.y} ${wingB.x},${wingB.y}`,
  };
});
</script>

<template>
  <g class="entry-sign">
    <!-- 内向き三角 = 「ここから始まる」 -->
    <polygon :points="geometry.triangle" fill="#000000" />
    <!-- 反時計回りの矢 = 「この向きに読む」(矢じりは判定で強化) -->
    <path :d="geometry.arc" fill="none" stroke="#000000" stroke-width="1.2" />
    <polygon :points="geometry.arrowhead" fill="#000000" />
  </g>
</template>

<style scoped>
/* 装飾 — クリック/ホバー判定を奪わない (SVG 装飾規約)。 */
.entry-sign {
  pointer-events: none;
}
</style>
