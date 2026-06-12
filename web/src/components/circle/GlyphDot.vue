<script setup lang="ts">
// 召喚印 (外部呼び出し)。ホバーで輪郭強調、クリックで呼び出し先インスペクタを開く
// (Phase 4.1 — コード断片の表示と、同ファイル関数ならそこからのピン遷移)。
import { computed } from "vue";

import { useFocusStore } from "../../stores/focus.ts";
import type { EffectGlyph } from "../../types/magia.ts";

const props = defineProps<{ glyph: EffectGlyph }>();
const focus = useFocusStore();

const isHovered = computed(() => focus.hoveredOperationId === props.glyph.id);

/** この召喚印の解決先 (同ファイル関数の qualified)。未解決は null。 */
const resolved = computed(() =>
  props.glyph.callTarget !== null ? focus.resolveCall(props.glyph.callTarget) : null,
);

/** 対応する周辺チップ側がホバーされている (双方向リンク強調、Phase 4.4)。 */
const isLinked = computed(() => resolved.value !== null && focus.hoveredLink === resolved.value);

function onClick(event: MouseEvent) {
  if (!props.glyph.selectable || props.glyph.callTarget === null) return;
  // 固定 (インスペクタ) を開く — window の「外側クリックで閉じる」に拾わせない
  event.stopPropagation();
  focus.inspectCall(props.glyph.callTarget, props.glyph.irId, event.clientX, event.clientY);
}

function onEnter(event: MouseEvent) {
  if (!props.glyph.selectable) return;
  focus.hoverOperation(props.glyph.id);
  // 対応する周辺チップを強調する (記号 ⇆ チップの地図対応、Phase 4.4)
  focus.setHoveredLink(resolved.value);
  // 呼び出し式のホバープレビュー (固定よりも上の層 — 追加要望3)
  const html = focus.spell?.call_excerpts[String(props.glyph.irId)] ?? null;
  if (html !== null) {
    focus.showHoverExcerpt(html, event.clientX, event.clientY, props.glyph.callTarget !== null);
  }
}

function onLeave() {
  if (!props.glyph.selectable) return;
  focus.hoverOperation(null);
  focus.setHoveredLink(null);
  focus.hideHoverExcerpt();
}
</script>

<template>
  <circle
    class="summon-glyph"
    :class="{ 'op-hovered': isHovered, 'link-highlight': isLinked }"
    :cx="glyph.x"
    :cy="glyph.y"
    :r="glyph.radius"
    :fill="glyph.color"
    :style="glyph.selectable && glyph.callTarget !== null ? { cursor: 'pointer' } : {}"
    @mouseenter="onEnter"
    @mouseleave="onLeave"
    @click="onClick"
  />
</template>

<style scoped>
.op-hovered {
  stroke: #00a0c0;
  stroke-width: 2;
}
/* チップ側のホバーに呼応するリンク強調 (Phase 4.4 — ホバー輪郭と同系で太め) */
.link-highlight {
  stroke: #00a0c0;
  stroke-width: 3;
}
</style>
