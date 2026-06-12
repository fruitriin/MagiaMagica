<script setup lang="ts">
// 魔法陣の Vue コンポーネントツリーのルート (Phase 4.0.7)。
// MagicCircleSchema (境界スキーマ) だけを受け、埋め方 (SVG パーサ / IR ビルダ) を知らない。
// 描画は z (元 SVG の出現順) でソートした単一リストで行い、重なりの z-order を
// 元レンダラと一致させる (画素等価の要件 — 種類別に描くと重なり箇所のピクセルが変わる)。
// レイヤー可視性・透明度は palette store から宣言的に適用する (spec §5.4 位置共有制約 —
// 切替は CSS のみで位置を変えない)。
import { computed } from "vue";

import { usePaletteStore } from "../../stores/palette.ts";
import type { DiffMark, MagicCircleSchema, SchemaLayer } from "../../types/magia.ts";
import EdgeLine from "./EdgeLine.vue";
import GlyphDot from "./GlyphDot.vue";
import OperationDot from "./OperationDot.vue";
import RawFragment from "./RawFragment.vue";
import RingCircle from "./RingCircle.vue";
import SignatureArc from "./SignatureArc.vue";
import SymbolMark from "./SymbolMark.vue";

const props = defineProps<{
  schema: MagicCircleSchema;
  /** 差分強調 (overlay-diff、Phase 4.3 M4)。配列順に最上層へ描く。
   *  強調チャネルはレイヤーの show/hide の影響を受けない (spec v0.3 §8)。 */
  overlay?: DiffMark[];
}>();

const palette = usePaletteStore();

/** 差分強調の意匠 (色の正は Vue 側 — uno.config.ts の theme と同語彙)。
 *  ハロー = 実線、ゴースト (removed) = 破線・半透明。 */
const DIFF_STYLE: Record<DiffMark["status"], { color: string; dash?: string; opacity?: number }> = {
  added: { color: "#d4a017" },
  changed: { color: "#00a0c0" },
  removed: { color: "#909090", dash: "5 4", opacity: 0.6 },
};

type DrawItem =
  | { kind: "circle"; z: number; circle: MagicCircleSchema["circles"][number] }
  | { kind: "op"; z: number; op: MagicCircleSchema["operations"][number] }
  | { kind: "glyph"; z: number; glyph: MagicCircleSchema["glyphs"][number] }
  | { kind: "edge"; z: number; edge: MagicCircleSchema["edges"][number] }
  | { kind: "symbol"; z: number; symbol: MagicCircleSchema["symbols"][number] }
  | { kind: "raw"; z: number; raw: MagicCircleSchema["raws"][number] };

const drawList = computed<DrawItem[]>(() =>
  [
    ...props.schema.circles.map((circle): DrawItem => ({ kind: "circle", z: circle.z, circle })),
    ...props.schema.operations.map((op): DrawItem => ({ kind: "op", z: op.z, op })),
    ...props.schema.glyphs.map((glyph): DrawItem => ({ kind: "glyph", z: glyph.z, glyph })),
    ...props.schema.edges.map((edge): DrawItem => ({ kind: "edge", z: edge.z, edge })),
    ...props.schema.symbols.map((symbol): DrawItem => ({ kind: "symbol", z: symbol.z, symbol })),
    ...props.schema.raws.map((raw): DrawItem => ({ kind: "raw", z: raw.z, raw })),
  ].sort((a, b) => a.z - b.z),
);

function itemId(item: DrawItem): string {
  switch (item.kind) {
    case "circle":
      return item.circle.id;
    case "op":
      return item.op.id;
    case "glyph":
      return item.glyph.id;
    case "edge":
      return item.edge.id;
    case "symbol":
      return item.symbol.id;
    case "raw":
      return item.raw.id;
  }
}

function itemLayer(item: DrawItem): SchemaLayer | null {
  switch (item.kind) {
    case "circle":
      return item.circle.layer;
    case "op":
      return item.op.layer;
    case "glyph":
      return item.glyph.layer;
    case "edge":
      return item.edge.layer;
    case "symbol":
      return item.symbol.layer;
    case "raw":
      return item.raw.layer;
  }
}

// 既定値 (表示・不透明) ではスタイルを付けない: opacity は 1 でも明示すると
// stacking context が生まれてラスタライズが変わり、v-html 版との画素等価が崩れる。
function layerStyle(layer: SchemaLayer | null): Record<string, string> {
  if (layer === null) return {};
  const state = palette.layers[layer];
  const style: Record<string, string> = {};
  if (!state.visible) style["display"] = "none";
  if (Math.abs(state.opacity - 1) > 1e-9) style["opacity"] = String(state.opacity);
  return style;
}
</script>

<template>
  <!-- サイズ指定 (w-full や x/y/width/height) は利用側が fallthrough 属性で与える
       (単独表示 = w-full、ピンビュー = ネスト svg の絶対配置。Phase 4.1)。 -->
  <svg xmlns="http://www.w3.org/2000/svg" :viewBox="schema.viewBox.join(' ')">
    <template v-for="item in drawList" :key="itemId(item)">
      <RingCircle
        v-if="item.kind === 'circle'"
        :circle="item.circle"
        :style="layerStyle(itemLayer(item))"
      />
      <OperationDot
        v-else-if="item.kind === 'op'"
        :op="item.op"
        :style="layerStyle(itemLayer(item))"
      />
      <GlyphDot
        v-else-if="item.kind === 'glyph'"
        :glyph="item.glyph"
        :style="layerStyle(itemLayer(item))"
      />
      <EdgeLine
        v-else-if="item.kind === 'edge'"
        :edge="item.edge"
        :style="layerStyle(itemLayer(item))"
      />
      <SymbolMark
        v-else-if="item.kind === 'symbol'"
        :symbol="item.symbol"
        :style="layerStyle(itemLayer(item))"
      />
      <RawFragment v-else :raw="item.raw" :style="layerStyle(itemLayer(item))" />
    </template>
    <SignatureArc v-if="schema.signature" :signature="schema.signature" />
    <!-- overlay-diff (最上層): 金ハロー = 追加 / シアンハロー = 変更 /
         灰破線ゴースト = 削除 (Phase 3.2 の意匠規約を Vue へ移植) -->
    <g v-if="overlay" class="overlay-diff">
      <circle
        v-for="(mark, index) in overlay"
        :key="index"
        :class="`diff-${mark.status}`"
        :cx="mark.x"
        :cy="mark.y"
        :r="mark.radius"
        fill="none"
        :stroke="DIFF_STYLE[mark.status].color"
        stroke-width="2.5"
        :stroke-dasharray="DIFF_STYLE[mark.status].dash"
        :stroke-opacity="DIFF_STYLE[mark.status].opacity"
      />
    </g>
  </svg>
</template>
