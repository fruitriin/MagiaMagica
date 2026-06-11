<script setup lang="ts">
// 制御フローの接続線 (Rust レンダラの edge-control-flow と画素等価)。
import type { SchemaEdge } from "../../types/magia.ts";

defineProps<{ edge: SchemaEdge }>();
</script>

<template>
  <line
    class="edge-control-flow"
    :x1="edge.x1"
    :y1="edge.y1"
    :x2="edge.x2"
    :y2="edge.y2"
    stroke="#000000"
    stroke-width="1.00"
  />
</template>
