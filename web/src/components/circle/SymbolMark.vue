<script setup lang="ts">
// 制御記号の描画 (Phase 4.0.9 — 旧 midchilda.rs の branch_symbol / loop_symbol /
// early_return_arrow / write_type_info の戻り値分岐 / async 二重線の幾何移植)。
// 頂点座標は spec §6.1.3 の意匠と画素等価になるよう旧実装と同じ式で計算する。
import { computed } from "vue";

import { RETURN_BRANCH_LENGTH } from "../../converters/irToSchema.ts";
import type { ControlSymbol } from "../../types/magia.ts";

const props = defineProps<{ symbol: ControlSymbol }>();

/** 分岐 (if/match): 二股に分かれる線の合流点。リング中央の Y 字。 */
const branchPath = computed(() => {
  const { x, y } = props.symbol;
  return `M${x} ${y + 9} L${x} ${y} M${x} ${y} L${x - 7} ${y - 8} M${x} ${y} L${x + 7} ${y - 8}`;
});

/** ループ: リング内周上部の反時計回り (画面上は左向き) 三角形。 */
const loopPoints = computed(() => {
  const { x, y, radius } = props.symbol;
  const track = radius - 10;
  const ty = y - track;
  return `${x + 5},${ty - 4} ${x + 5},${ty + 4} ${x - 5},${ty}`;
});

/** 早期リターン: リング内側から外へ抜ける矢印 (線 + 矢頭)。 */
const earlyReturn = computed(() => {
  const { x, y, radius, direction } = props.symbol;
  const [ux, uy] = direction;
  const inner = { x: x + ux * (radius - 10), y: y + uy * (radius - 10) };
  const tip = { x: x + ux * (radius + 12), y: y + uy * (radius + 12) };
  const perp = { x: -uy, y: ux };
  const base = { x: tip.x - ux * 6, y: tip.y - uy * 6 };
  const wingA = { x: base.x + perp.x * 4, y: base.y + perp.y * 4 };
  const wingB = { x: base.x - perp.x * 4, y: base.y - perp.y * 4 };
  return {
    line: { x1: inner.x, y1: inner.y, x2: tip.x, y2: tip.y },
    head: `${tip.x},${tip.y} ${wingA.x},${wingA.y} ${wingB.x},${wingB.y}`,
  };
});

/** Result/Option 戻り値: 9時から出る正常 (実線)/異常 (破線) の分岐線。 */
const returnBranch = computed(() => {
  const { x, y } = props.symbol;
  return {
    ok: { x2: x - RETURN_BRANCH_LENGTH, y2: y - RETURN_BRANCH_LENGTH * 0.45 },
    err: { x2: x - RETURN_BRANCH_LENGTH, y2: y + RETURN_BRANCH_LENGTH * 0.45 },
  };
});

/** async fn の内側二重線のオフセット (layout/constants.rs ASYNC_INNER_RING_OFFSET)。 */
const ASYNC_INNER_RING_OFFSET = 5;
</script>

<template>
  <path
    v-if="symbol.kind === 'branch'"
    class="sym-branch"
    :d="branchPath"
    stroke="#000000"
    stroke-width="1"
    fill="none"
  />
  <polygon
    v-else-if="symbol.kind === 'loop'"
    class="sym-loop"
    :points="loopPoints"
    fill="#000000"
  />
  <template v-else-if="symbol.kind === 'early_return'">
    <line
      class="sym-early-return"
      :x1="earlyReturn.line.x1"
      :y1="earlyReturn.line.y1"
      :x2="earlyReturn.line.x2"
      :y2="earlyReturn.line.y2"
      stroke="#000000"
      stroke-width="1.5"
    />
    <polygon class="sym-early-return" :points="earlyReturn.head" fill="#000000" />
  </template>
  <template v-else-if="symbol.kind === 'return_branch'">
    <line
      class="return-path-ok"
      :x1="symbol.x"
      :y1="symbol.y"
      :x2="returnBranch.ok.x2"
      :y2="returnBranch.ok.y2"
      stroke="#000000"
      stroke-width="1"
    />
    <line
      class="return-path-err"
      :x1="symbol.x"
      :y1="symbol.y"
      :x2="returnBranch.err.x2"
      :y2="returnBranch.err.y2"
      stroke="#000000"
      stroke-width="1"
      stroke-dasharray="4 3"
    />
  </template>
  <circle
    v-else-if="symbol.kind === 'async_inner'"
    class="main-ring-async"
    :cx="symbol.x"
    :cy="symbol.y"
    :r="symbol.radius - ASYNC_INNER_RING_OFFSET"
    fill="none"
    stroke="#000000"
    stroke-width="1"
  />
</template>

<style scoped>
/* 制御記号は装飾 — リング・ドットのホバー/クリック判定を奪わない
   (シグネチャ円弧と同じ原則、オーナー指摘 2026-06-12)。 */
* {
  pointer-events: none;
}
</style>
