<script setup lang="ts">
// ベルカ式 — データフロー三角力場 (Phase 4.3 M3、spec v0.3 §14)。
// 配置済み IR (BelkaIr) を受けて描画するだけ — 射影 (三極分類) と三角配置は
// Rust (belka.rs) が確定する。色・ラベル文言・矢じり形状 (描き方) はここが持つ。
// 意匠は旧 Rust SVG レンダラ (belka.rs write_document) と画素等価が目標。
import { useId } from "vue";

import type { BelkaFlowIr, BelkaIr, BelkaPoleKind } from "../../types/magia.ts";

const props = defineProps<{ belka: BelkaIr }>();

/** 極の意匠 (palette.rs の BELKA_* と同値 — 色変更時は両方直す)。 */
const POLE_STYLE: Record<BelkaPoleKind, { color: string; label: string }> = {
  genesis: { color: "#2f86c9", label: "生成" },
  transmute: { color: "#c98a2f", label: "変換" },
  consume: { color: "#b04a5a", label: "消費" },
};

/** 効果カテゴリ → 色 (irToSchema の COLOR_BY_EFFECT と同テーブル)。 */
const DOT_COLOR: Record<string, string> = {
  pure: "#000000",
  io: "#1f4dff",
  network: "#7b3ff5",
  db: "#1fa341",
  filesystem: "#7a4a1c",
  unsafe: "#d92626",
};

/** 力場グラデーション中心の不透明度 (belka.rs FIELD_OPACITY と同値)。 */
const FIELD_OPACITY = 0.28;
/** 操作ドット半径 (belka.rs DOT_RADIUS と同値)。 */
const DOT_RADIUS = 3.5;

// グラデーション id はインスタンスごとに一意化 (SVG の参照 id は DOM 全体で
// ユニーク — SignatureArc と同じ理由)。
const uid = useId();
const gradientId = (pole: BelkaPoleKind) => `belka-field-${pole}-${uid}`;

/** 矢じり (頂点 + 羽2点)。tip → 線端の方向 (= フローの向き) から計算する。 */
function arrowHead(flow: BelkaFlowIr): string {
  const dx = flow.tip_x - flow.x2;
  const dy = flow.tip_y - flow.y2;
  const length = Math.hypot(dx, dy);
  if (length < 1e-6) return "";
  const ux = dx / length;
  const uy = dy / length;
  const baseX = flow.tip_x - ux * 7;
  const baseY = flow.tip_y - uy * 7;
  // 進行方向の垂直ベクトル × 羽幅 4.5 (belka.rs write_flow_line と同値)。
  const wingAx = baseX + -uy * 4.5;
  const wingAy = baseY + ux * 4.5;
  const wingBx = baseX - -uy * 4.5;
  const wingBy = baseY - ux * 4.5;
  return `${flow.tip_x},${flow.tip_y} ${wingAx},${wingAy} ${wingBx},${wingBy}`;
}
</script>

<template>
  <svg xmlns="http://www.w3.org/2000/svg" :viewBox="belka.view_box.join(' ')">
    <!-- 力場のグラデーション (中心が濃く外周で消える)。gradientUnits は既定
         (objectBoundingBox): 参照元が circle なので中心 = 円の中心になる -->
    <defs>
      <radialGradient v-for="p in belka.poles" :id="gradientId(p.pole)" :key="p.pole">
        <stop offset="0" :stop-color="POLE_STYLE[p.pole].color" :stop-opacity="FIELD_OPACITY" />
        <stop offset="1" :stop-color="POLE_STYLE[p.pole].color" stop-opacity="0" />
      </radialGradient>
    </defs>

    <!-- 1) 力場 (最背面、重なりがアディティブに濃くなる) -->
    <g class="belka-field">
      <circle
        v-for="p in belka.poles"
        :key="p.pole"
        :class="`field-${p.pole}`"
        :cx="p.x"
        :cy="p.y"
        :r="p.field_radius"
        :fill="`url(#${gradientId(p.pole)})`"
      />
    </g>

    <!-- 2) フロー線 (極間のデータの流れ。太さ = フロー量) -->
    <g class="belka-flows">
      <template v-for="(flow, index) in belka.flows" :key="index">
        <line
          class="belka-flow"
          :x1="flow.x1"
          :y1="flow.y1"
          :x2="flow.x2"
          :y2="flow.y2"
          stroke="#555555"
          :stroke-width="flow.width"
          stroke-opacity="0.75"
        />
        <polygon class="belka-flow-head" :points="arrowHead(flow)" fill="#555555" />
      </template>
    </g>

    <!-- 3) 極 (頂点円 + ラベル + 操作ドット) -->
    <g class="belka-poles">
      <template v-for="p in belka.poles" :key="p.pole">
        <circle
          class="belka-pole"
          :cx="p.x"
          :cy="p.y"
          :r="p.radius"
          fill="#ffffff"
          fill-opacity="0.6"
          :stroke="POLE_STYLE[p.pole].color"
          stroke-width="2"
        />
        <text
          class="belka-pole-label"
          :x="p.label_x"
          :y="p.label_y"
          font-size="12"
          :fill="POLE_STYLE[p.pole].color"
          text-anchor="middle"
        >
          {{ POLE_STYLE[p.pole].label }}
        </text>
        <circle
          v-for="(dot, index) in p.dots"
          :key="index"
          class="op-dot"
          :cx="dot.x"
          :cy="dot.y"
          :r="DOT_RADIUS"
          :fill="DOT_COLOR[dot.effect] ?? '#000000'"
        />
      </template>
    </g>

    <!-- 4) シグネチャ (上端の平書き — 円弧ラベルはミッドチルダ式の意匠) -->
    <text
      v-if="belka.signature"
      class="signature"
      :x="belka.signature.x"
      :y="belka.signature.y"
      font-size="11"
      fill="#000000"
      text-anchor="middle"
    >
      {{ belka.signature.text }}
    </text>
  </svg>
</template>
