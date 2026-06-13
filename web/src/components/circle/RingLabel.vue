<script setup lang="ts">
// 補助陣のラベル (Phase 4.0.6 後半)。リング上端 (9時 → 12時 → 3時) の弧テキスト。
// 並んだ補助陣の意味 (if/else/for/while/loop/closure params/match pat) を一目で
// 読み取れるようにする — シグネチャ円弧と同じ規約。
// 装飾専用 — クリック/ホバーは奪わない。
import { useId } from "vue";

import type { RingLabel } from "../../types/magia.ts";

defineProps<{ label: RingLabel }>();
// arcPath の id はインスタンス単位で一意化 (ピン中心ビューで複数陣が載るため)。
const arcId = `ring-label-${useId()}`;
</script>

<template>
  <g class="ring-label">
    <defs>
      <path :id="arcId" :d="label.arcPath" fill="none" />
    </defs>
    <text :font-size="label.fontSize" fill="#000000" font-family="ui-monospace, monospace">
      <textPath :href="`#${arcId}`" startOffset="50%" text-anchor="middle">{{
        label.text
      }}</textPath>
    </text>
  </g>
</template>

<style scoped>
.ring-label {
  pointer-events: none;
}
</style>
