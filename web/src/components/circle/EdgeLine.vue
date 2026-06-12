<script setup lang="ts">
// 接続線。制御フロー (リングの係留線) と、メソッドチェーンの連結線 (Phase 4.8 —
// 召喚陣から召喚陣へ値が手渡される鎖) を線の太さで描き分ける。
import type { SchemaEdge } from "../../types/magia.ts";

defineProps<{ edge: SchemaEdge }>();
</script>

<template>
  <line
    :class="edge.chain ? 'edge-chain' : 'edge-control-flow'"
    :x1="edge.x1"
    :y1="edge.y1"
    :x2="edge.x2"
    :y2="edge.y2"
    stroke="#000000"
    :stroke-width="edge.chain ? 0.6 : 1.0"
  />
</template>
