<script setup lang="ts">
// 魔法陣ペイン (Phase 4.1 でピン中心ビューが標準表示に)。
// フォーカス魔法陣を中央に、周辺関数チップを距離リングへ配置する太陽系モデル。
// 配置 (x/y/scale/opacity) は Rust の focus_layout が確定済みで、Vue は
// 「中心原点のコンテンツ + g の CSS transform」で描くだけ。ピン打ち直しでは
// 同一 key (qualified) の <g> の transform が CSS transition で補間され、
// 旧フォーカスが周辺へ縮みつつ移動・新フォーカスが中心へズームインする。
// ベルカ式は Phase 4.3 の Vue SSR 移行まで SVG 文字列 (svg_belka) を素通し表示
// (保守価値低め — 改善はリメイクに織り込む。オーナー判定 2026-06-11)。
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";

import { irToSchema } from "../converters/irToSchema.ts";
import { useFocusStore } from "../stores/focus.ts";
import { usePaletteStore } from "../stores/palette.ts";
import MagicCircle from "./circle/MagicCircle.vue";
import NeighborChip from "./circle/NeighborChip.vue";

const focus = useFocusStore();
const palette = usePaletteStore();
const route = useRoute();
const router = useRouter();

const schema = computed(() => {
  if (focus.spell === null || palette.style === "belka") return null;
  return irToSchema(focus.spell.ir);
});

const belkaSvg = computed(() =>
  palette.style === "belka" ? (focus.spell?.svg_belka ?? null) : null,
);

const layout = computed(() => focus.spell?.focus_layout ?? null);

/** 外側 svg の viewBox。周辺があれば全体配置、なければフォーカス単体。 */
const outerViewBox = computed(() => {
  if (schema.value === null) return "0 0 0 0";
  return (layout.value?.view_box ?? schema.value.viewBox).join(" ");
});

type PinNode = {
  qualified: string;
  kind: "focus" | "chip";
  transform: string;
  opacity: number;
};

/** フォーカス + チップを単一リストで描く (同一 key の g が遷移で再利用される)。 */
const nodes = computed<PinNode[]>(() => {
  if (schema.value === null || focus.spell === null) return [];
  const [minX, minY, width, height] = schema.value.viewBox;
  const centerX = minX + width / 2;
  const centerY = minY + height / 2;
  const result: PinNode[] = [
    {
      qualified: focus.spell.qualified,
      kind: "focus",
      transform: `translate(${centerX}px, ${centerY}px) scale(1)`,
      opacity: 1,
    },
  ];
  for (const chip of layout.value?.neighbors ?? []) {
    result.push({
      qualified: chip.qualified,
      kind: "chip",
      transform: `translate(${chip.x}px, ${chip.y}px) scale(${chip.scale})`,
      opacity: chip.opacity,
    });
  }
  return result;
});

const chipByQualified = computed(() => {
  const map = new Map(layout.value?.neighbors.map((c) => [c.qualified, c]) ?? []);
  return map;
});

/** フォーカス魔法陣を中心原点で置くためのネスト svg 配置。 */
const focusPlacement = computed(() => {
  if (schema.value === null) return null;
  const [, , width, height] = schema.value.viewBox;
  return { x: -width / 2, y: -height / 2, width, height };
});

function pin(qualified: string) {
  if (qualified === focus.currentFn) return;
  void router.push({ query: { ...route.query, pin: qualified } });
}
</script>

<template>
  <div v-if="schema" w-full>
    <svg xmlns="http://www.w3.org/2000/svg" class="pin-view" :viewBox="outerViewBox" w-full>
      <g
        v-for="node in nodes"
        :key="node.qualified"
        class="pin-node"
        :style="{ transform: node.transform, opacity: node.opacity }"
      >
        <MagicCircle
          v-if="node.kind === 'focus'"
          :schema="schema"
          :x="focusPlacement?.x"
          :y="focusPlacement?.y"
          :width="focusPlacement?.width"
          :height="focusPlacement?.height"
        />
        <NeighborChip
          v-else-if="chipByQualified.get(node.qualified)"
          :chip="chipByQualified.get(node.qualified)!"
          @pin="pin"
        />
      </g>
    </svg>
  </div>
  <div v-else-if="belkaSvg" w-full v-html="belkaSvg" />
  <div v-else p-8 text-gray-400>魔法陣を読み込み中…</div>
</template>

<style scoped>
/* ピン遷移: 位置・スケール・透明度を補間 (250〜400ms の指定中庸)。
   reduced-motion 環境では即時切替 (アクセシビリティ要件)。 */
.pin-node {
  transition:
    transform 320ms ease,
    opacity 320ms ease;
}
@media (prefers-reduced-motion: reduce) {
  .pin-node {
    transition: none;
  }
}
</style>
