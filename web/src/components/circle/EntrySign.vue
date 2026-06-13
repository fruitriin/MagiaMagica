<script setup lang="ts">
// 実行順の出発点を示す入口サイン (Phase 4.0.6 後半)。メインリングの 3 時に
// 内向きの ▷ (右向き三角) を置き、上向きにごく短い反時計回りの円弧矢を添える —
// 「ここから上 (12時方向) へ反時計回りに読む」が一目で分かる。
// 視覚アフォーダンス専用 — クリック・ホバーは奪わない。
import { computed } from "vue";

import type { EntrySign } from "../../types/magia.ts";

const props = defineProps<{ sign: EntrySign }>();

/** 三角の頂点 (リング 3 時、内側を向く) と円弧の終点・矢じり。
 *  矢じりのサイズは召喚陣 (SUMMON_GLYPH_RADIUS=14) の半分くらいが目安
 *  (オーナー判定 2026-06-13)。 */
const geometry = computed(() => {
  const { x, y, radius } = props.sign;
  // 三角 ▷ は外向き (頂点がリング外周方向、底辺が内側) — 「ここから外へ
  // 流れていく」進入の向き感を出す (オーナー判定 2026-06-13 で頂点方向反転)。
  const tipInset = 4;
  const tipX = x + radius - 4; // 頂点 = リング外周直前
  const baseX = x + radius - 11; // 底辺 = 内側
  const halfHeight = 4;
  // 反時計回り (画面上 3時 → 12時方向) の円弧。終端角度を -π/3 まで伸ばし、
  // 矢じり付きの円弧として十分な進行を見せる。
  const arcRadius = radius - tipInset;
  const endAngle = -Math.PI / 3;
  const start = { x: x + arcRadius, y };
  const end = {
    x: x + arcRadius * Math.cos(endAngle),
    y: y + arcRadius * Math.sin(endAngle),
  };
  // 矢じり: end での「進行方向 (接線)」を半径ベクトルから直接計算する。
  // 半径 r = end - center を画面座標系で 90度回す (rx, ry) → (ry, -rx) の向きが、
  // SVG y軸下向きにおける「反時計回りの接線」(= 3時から見て上 = 進行方向)。
  // 進行方向と逆方向 (= 円弧の手前側) が矢じりの後ろ (back) になる。
  const rx = end.x - x;
  const ry = end.y - y;
  const rlen = Math.hypot(rx, ry) || 1;
  const dirX = ry / rlen;
  const dirY = -rx / rlen;
  // 判定 2026-06-13 (4回目): 5x3 は小さすぎたため少し大きく。
  const arrowLen = 9;
  const arrowHalfWidth = 5;
  const back = {
    x: end.x - arrowLen * dirX,
    y: end.y - arrowLen * dirY,
  };
  // 接線に垂直 (= 半径方向) の単位ベクトルで翼を開く。
  const perpX = -dirY;
  const perpY = dirX;
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
    // 円弧は矢じり本体まで描く (back ではなく end へ — path と arrowhead で
    // 二重に重ねて先端の見え方を出す)。
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
