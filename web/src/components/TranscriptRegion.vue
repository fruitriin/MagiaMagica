<script setup lang="ts">
// 呪文書き起こし (Phase 2.4)。スクリーンリーダーにのみ露出する (spec v0.2 §15)。
import { useFocusStore } from "../stores/focus.ts";

const focus = useFocusStore();
</script>

<template>
  <div class="visually-hidden" role="region" aria-label="呪文書き起こし">
    {{ focus.spell?.transcript ?? "" }}
  </div>
</template>

<style scoped>
/* clip (旧) と clip-path (新) を併記し、margin/padding/border を明示する完全形。
   UnoCSS の sr-only と等価だが、spec §15 準拠の定義を明示するため scoped で持つ。 */
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  clip-path: inset(50%);
  white-space: nowrap;
  border: 0;
}
</style>
