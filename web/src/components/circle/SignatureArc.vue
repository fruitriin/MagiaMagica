<script setup lang="ts">
// 関数シグネチャ。ミッドチルダ式は円弧 textPath、ベルカ式は直線テキスト
// (Rust レンダラの signature と画素等価)。
// arc path の id はコンポーネントインスタンスごとに一意化する: SVG の参照 id は
// **DOM ドキュメント全体**でユニークである必要があり、同一ページに複数の魔法陣が
// 載る Phase 4.1 のピン中心ビューで固定 id だと衝突する。
//
// 引数表示オプション (細部修正 2026-06-12): パレットの変数名/型名チェックボックスが
// ON のとき、text の代わりに構造化部品 (name/args/ret) から組み立てる —
// 周辺チップ (NeighborChip) と同じ規則。両方 OFF は従来どおり text そのまま
// (SSR 静止画もこの既定経路 — 静止画の見た目は変えない)。
import { computed, useId } from "vue";

import { usePaletteStore } from "../../stores/palette.ts";
import type { Signature } from "../../types/magia.ts";

const props = defineProps<{ signature: Signature }>();
const arcId = `sig-arc-${useId()}`;
const palette = usePaletteStore();

const displayText = computed(() => {
  const sig = props.signature;
  if ((!palette.argNames && !palette.argTypes) || sig.name === undefined) {
    return sig.text;
  }
  const parts = (sig.args ?? []).map((a) => {
    if (palette.argNames && palette.argTypes) return `${a.name}: ${a.ty}`;
    return palette.argNames ? a.name : a.ty;
  });
  const ret = sig.ret === undefined ? "" : ` -> ${sig.ret}`;
  return `fn ${sig.name}(${parts.join(", ")})${ret}`;
});
</script>

<template>
  <template v-if="signature.arcPath !== null">
    <defs>
      <path :id="arcId" :d="signature.arcPath" fill="none" />
    </defs>
    <text class="signature" :font-size="signature.fontSize" fill="#000000">
      <textPath :href="`#${arcId}`" startOffset="50%" text-anchor="middle">{{
        displayText
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
    {{ displayText }}
  </text>
</template>

<style scoped>
/* シグネチャは装飾 — 外周の円弧テキストが召喚印に重なってクリック判定を
   奪わないようにする (オーナー指摘 2026-06-12)。 */
.signature {
  pointer-events: none;
}
</style>
