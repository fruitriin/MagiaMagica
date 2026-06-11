<script setup lang="ts">
// シンボル凡例 (Phase 4.0.6 前半)。魔法陣の色と記号の意味を UI 上から参照できる
// 開閉式パネル。記号サンプルは実際の描画コンポーネント (circle/*) を小さな
// viewBox で再利用する — 意匠を変えたとき凡例が自動で追従し、乖離しない。
// ラベルは書き起こし (transcript.rs) と同語源。色は uno.config.ts theme =
// palette.rs と同語彙。
import type { Circle, ControlSymbol, EffectCategory, Operation } from "../types/magia.ts";
import EdgeLine from "./circle/EdgeLine.vue";
import GlyphDot from "./circle/GlyphDot.vue";
import OperationDot from "./circle/OperationDot.vue";
import RingCircle from "./circle/RingCircle.vue";
import SymbolMark from "./circle/SymbolMark.vue";

const EFFECTS: { effect: EffectCategory; color: string; label: string }[] = [
  { effect: "pure", color: "#000000", label: "純粋" },
  { effect: "io", color: "#1f4dff", label: "IO 副作用" },
  { effect: "network", color: "#7b3ff5", label: "ネットワーク副作用" },
  { effect: "db", color: "#1fa341", label: "DB 副作用" },
  { effect: "filesystem", color: "#7a4a1c", label: "ファイルシステム副作用" },
  { effect: "unsafe", color: "#d92626", label: "unsafe" },
];

const BELKA = [
  { color: "#2f86c9", label: "生成 (値の誕生)" },
  { color: "#c98a2f", label: "変換 (鍛錬・加工)" },
  { color: "#b04a5a", label: "消費 (放出・副作用・帰還)" },
];

// 凡例サンプル用のスキーマ片 (selectable: false — クリックで選択状態にしない)。
const sampleRing = (role: "main" | "aux"): Circle => ({
  id: `legend-ring-${role}`,
  z: 0,
  role,
  x: 0,
  y: 0,
  radius: 11,
  strokeWidth: role === "main" ? 2 : 1.5,
  layer: null,
});

const sampleOp = (color: string): Operation => ({
  id: `legend-op-${color}`,
  z: 0,
  x: 0,
  y: 0,
  radius: 3.5,
  color,
  effect: null,
  selectable: false,
  layer: null,
});

const sampleSymbol = (kind: ControlSymbol["kind"]): ControlSymbol => ({
  id: `legend-sym-${kind}`,
  kind,
  x: 0,
  y: kind === "loop" ? 0 : kind === "early_return" ? 0 : 2,
  radius: kind === "loop" ? 22 : kind === "early_return" ? 4 : 11,
  direction: [1, 0],
  layer: "control_flow",
  z: 0,
});
</script>

<template>
  <details px-3 py-2 text-sm>
    <summary cursor-pointer select-none text-xs font-bold text-gray-600>📖 凡例</summary>

    <strong mt-2 block text-xs text-gray-500>効果カテゴリ (操作・召喚印の色)</strong>
    <div v-for="e in EFFECTS" :key="e.effect" mt-1 flex items-center gap-2>
      <svg viewBox="-6 -6 12 12" w-4 h-4 shrink-0>
        <OperationDot :op="sampleOp(e.color)" />
      </svg>
      <span text-xs>{{ e.label }}</span>
    </div>

    <strong mt-3 block text-xs text-gray-500>図形</strong>
    <div mt-1 flex items-center gap-2>
      <svg viewBox="-14 -14 28 28" w-5 h-5 shrink-0>
        <RingCircle :circle="sampleRing('main')" />
      </svg>
      <span text-xs>メインリング = 関数本体 (操作は3時起点・反時計回り)</span>
    </div>
    <div mt-1 flex items-center gap-2>
      <svg viewBox="-14 -14 28 28" w-5 h-5 shrink-0>
        <RingCircle :circle="sampleRing('aux')" />
      </svg>
      <span text-xs>補助リング = 分岐・ループの本体</span>
    </div>
    <div mt-1 flex items-center gap-2>
      <svg viewBox="-7 -7 14 14" w-5 h-5 shrink-0>
        <GlyphDot
          :glyph="{
            id: 'legend-glyph',
            z: 0,
            x: 0,
            y: 0,
            radius: 5,
            color: '#1f4dff',
            effect: 'io',
            selectable: false,
            layer: null,
          }"
        />
      </svg>
      <span text-xs>召喚印 = 外部呼び出し (色 = 効果カテゴリ)</span>
    </div>
    <div mt-1 flex items-center gap-2>
      <svg viewBox="-10 -10 20 20" w-5 h-5 shrink-0>
        <SymbolMark :symbol="sampleSymbol('branch')" />
      </svg>
      <span text-xs>Y 字 = 分岐 (if / match)</span>
    </div>
    <div mt-1 flex items-center gap-2>
      <svg viewBox="-10 -18 20 24" w-5 h-5 shrink-0>
        <SymbolMark :symbol="sampleSymbol('loop')" />
      </svg>
      <span text-xs>左向き三角 = ループ (反時計回りに周回)</span>
    </div>
    <div mt-1 flex items-center gap-2>
      <svg viewBox="-8 -8 28 16" w-5 h-5 shrink-0>
        <SymbolMark :symbol="sampleSymbol('early_return')" />
      </svg>
      <span text-xs>外向き矢印 = 早期リターン</span>
    </div>
    <div mt-1 flex items-center gap-2>
      <svg viewBox="-30 -16 34 32" w-5 h-5 shrink-0>
        <SymbolMark :symbol="sampleSymbol('return_branch')" />
      </svg>
      <span text-xs>9時の実線/破線 = Result・Option の正常/異常経路</span>
    </div>
    <div mt-1 flex items-center gap-2>
      <svg viewBox="0 -6 24 12" w-5 h-5 shrink-0>
        <EdgeLine
          :edge="{
            id: 'legend-edge',
            z: 0,
            x1: 0,
            y1: 0,
            x2: 24,
            y2: 0,
            layer: null,
            from: null,
            to: null,
          }"
        />
      </svg>
      <span text-xs>線 = 制御フローの接続</span>
    </div>

    <strong mt-3 block text-xs text-gray-500>操作の強調</strong>
    <div mt-1 flex items-center gap-2>
      <svg viewBox="-6 -6 12 12" w-4 h-4 shrink-0>
        <circle cx="0" cy="0" r="3.5" fill="#000000" stroke="#00a0c0" stroke-width="2" />
      </svg>
      <span text-xs>シアン輪郭 = ホバー中</span>
    </div>
    <div mt-1 flex items-center gap-2>
      <svg viewBox="-6 -6 12 12" w-4 h-4 shrink-0>
        <circle cx="0" cy="0" r="3.5" fill="#000000" stroke="#d4a017" stroke-width="2.5" />
      </svg>
      <span text-xs>金輪郭 = 選択中 (クリックで切替)</span>
    </div>

    <strong mt-3 block text-xs text-gray-500>ベルカ式の三極</strong>
    <div v-for="b in BELKA" :key="b.color" mt-1 flex items-center gap-2>
      <svg viewBox="-6 -6 12 12" w-4 h-4 shrink-0>
        <circle cx="0" cy="0" r="5" fill="none" :stroke="b.color" stroke-width="1.5" />
      </svg>
      <span text-xs>{{ b.label }}</span>
    </div>
  </details>
</template>
