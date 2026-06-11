<script setup lang="ts">
// 関数シグネチャ。ミッドチルダ式は円弧 textPath、ベルカ式は直線テキスト
// (Rust レンダラの signature と画素等価)。
// arc path の id はコンポーネントインスタンスごとに一意化する: SVG の参照 id は
// **DOM ドキュメント全体**でユニークである必要があり、同一ページに複数の魔法陣が
// 載る Phase 4.1 のピン中心ビューで固定 id だと衝突する。
import { useId } from "vue";

import type { Signature } from "../../types/magia.ts";

defineProps<{ signature: Signature }>();
const arcId = `sig-arc-${useId()}`;
</script>

<template>
  <template v-if="signature.arcPath !== null">
    <defs>
      <path :id="arcId" :d="signature.arcPath" fill="none" />
    </defs>
    <text class="signature" :font-size="signature.fontSize" fill="#000000">
      <textPath :href="`#${arcId}`" startOffset="50%" text-anchor="middle">{{
        signature.text
      }}</textPath>
    </text>
  </template>
  <text
    v-else
    class="signature"
    :x="signature.x"
    :y="signature.y"
    :font-size="signature.fontSize"
    fill="#000000"
    text-anchor="middle"
  >
    {{ signature.text }}
  </text>
</template>

<style scoped>
/* シグネチャは装飾 — 外周の円弧テキストが召喚印に重なってクリック判定を
   奪わないようにする (オーナー指摘 2026-06-12)。 */
.signature {
  pointer-events: none;
}
</style>
