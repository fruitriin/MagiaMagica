<script setup lang="ts">
// 中央魔法陣 / 補助リングの円 (Rust レンダラの main-ring / aux-ring と画素等価)。
import type { Circle } from "../../types/magia.ts";

defineProps<{ circle: Circle }>();
</script>

<template>
  <circle
    :class="circle.role === 'main' ? 'main-ring' : 'aux-ring'"
    :cx="circle.x"
    :cy="circle.y"
    :r="circle.radius"
    fill="none"
    stroke="#000000"
    :stroke-width="circle.strokeWidth"
  />
</template>
