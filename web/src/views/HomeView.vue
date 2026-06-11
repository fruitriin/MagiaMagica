<script setup lang="ts">
// M1: 基盤起動の確認画面。uno.config.ts の theme (palette.rs と同語彙) が
// 効いていることをスウォッチで目視できるようにする。魔法陣ペインは M2 で載せる。
const effectSwatches = [
  ["pure", "bg-effect-pure"],
  ["io", "bg-effect-io"],
  ["network", "bg-effect-network"],
  ["db", "bg-effect-db"],
  ["filesystem", "bg-effect-filesystem"],
  ["unsafe", "bg-effect-unsafe"],
] as const;
const diffSwatches = [
  ["added", "bg-diff-added"],
  ["changed", "bg-diff-changed"],
  ["removed", "bg-diff-removed"],
] as const;
const belkaSwatches = [
  ["genesis", "bg-belka-genesis"],
  ["transmute", "bg-belka-transmute"],
  ["consume", "bg-belka-consume"],
] as const;
</script>

<template>
  <main mx-auto max-w-2xl p-8 font-sans>
    <h1 text-3xl font-bold tracking-wide>MagiaMagica</h1>
    <p mt-2 text-gray-500>Vue 3 + Vite+ + UnoCSS 基盤 (Phase 4.0.5 M1)</p>

    <section mt-8>
      <h2 text-sm font-bold text-gray-600>効果カテゴリ (palette.rs)</h2>
      <div mt-2 flex gap-3>
        <div v-for="[name, cls] in effectSwatches" :key="name" flex items-center gap-1>
          <span :class="cls" inline-block h-4 w-4 rounded-full />
          <span text-xs text-gray-500>{{ name }}</span>
        </div>
      </div>
    </section>

    <section mt-4>
      <h2 text-sm font-bold text-gray-600>Spell Diff</h2>
      <div mt-2 flex gap-3>
        <div v-for="[name, cls] in diffSwatches" :key="name" flex items-center gap-1>
          <span :class="cls" inline-block h-4 w-4 rounded-full />
          <span text-xs text-gray-500>{{ name }}</span>
        </div>
      </div>
    </section>

    <section mt-4>
      <h2 text-sm font-bold text-gray-600>ベルカ式三極</h2>
      <div mt-2 flex gap-3>
        <div v-for="[name, cls] in belkaSwatches" :key="name" flex items-center gap-1>
          <span :class="cls" inline-block h-4 w-4 rounded-full />
          <span text-xs text-gray-500>{{ name }}</span>
        </div>
      </div>
    </section>
  </main>
</template>
