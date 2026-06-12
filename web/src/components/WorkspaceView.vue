<script setup lang="ts">
// ワークスペース俯瞰 (Phase 4.5 M1)。全 .rs をディレクトリごとのファイルカードで
// 並べる「最大ズームアウト」。カードクリックで監視ファイルを切替 (4.4.5) し、
// ピン中心ビューへズームインする。カードのグリッドは CSS レイアウト —
// 「配置は Rust」の規約は魔法陣の幾何の話で、カード一覧は web の自然な
// レイアウトで足りる (M2+ でミニ魔法陣タイルを置くときに再考)。
import { computed } from "vue";

import { useFocusStore } from "../stores/focus.ts";

const focus = useFocusStore();

/** ディレクトリごとのグルーピング (モジュール階層の最小表現 — M1)。
 *  並びはパス辞書順 (決定論)。 */
const groups = computed(() => {
  const byDir = new Map<string, NonNullable<typeof focus.workspace>>();
  for (const file of focus.workspace ?? []) {
    const list = byDir.get(file.dir) ?? [];
    list.push(file);
    byDir.set(file.dir, list);
  }
  return [...byDir.entries()].sort(([a], [b]) => a.localeCompare(b));
});

/** 現在ファイルとの呼び出し関係 (M2 前段)。チップの向きマーク (Phase 4.4) と
 *  同じ語彙: → = 現在ファイルが呼ぶ / ← = 現在ファイルを呼ぶ / ⇄ = 相互。 */
const RELATION_MARK = {
  calls: { mark: "→", note: "現在のファイルが呼ぶ" },
  called_by: { mark: "←", note: "現在のファイルを呼ぶ" },
  mutual: { mark: "⇄", note: "相互に呼び合う" },
} as const;

const relationByFile = computed(() => {
  const out = new Map<string, keyof typeof RELATION_MARK>();
  if (focus.file === null) return out;
  const calls = new Set<string>();
  const calledBy = new Set<string>();
  for (const edge of focus.workspaceEdges) {
    if (edge.from_file === focus.file) calls.add(edge.to_file);
    if (edge.to_file === focus.file) calledBy.add(edge.from_file);
  }
  for (const path of calls) out.set(path, calledBy.has(path) ? "mutual" : "calls");
  for (const path of calledBy) {
    if (!calls.has(path)) out.set(path, "called_by");
  }
  return out;
});

function relationOf(path: string) {
  const kind = relationByFile.value.get(path);
  return kind === undefined ? null : RELATION_MARK[kind];
}

async function zoomInto(path: string) {
  // ズームイン: ファイルを切り替えてピン中心ビューへ。切替の追従は SSE。
  // switchFile はエラーを loadError に畳む設計 — 失敗時は俯瞰に留まり、
  // ヘッダ下のエラーバナーで気づかせる (別ファイルのピンビューに飛ばさない)。
  await focus.switchFile(path);
  if (focus.loadError === null) {
    await focus.setScope("focus");
  }
}
</script>

<template>
  <div h-full overflow-auto p-4>
    <div v-if="focus.workspace === null" text-gray-400>俯瞰を読み込み中…</div>
    <template v-else>
      <section v-for="[dir, files] in groups" :key="dir" mb-4>
        <h2 mb-1 text-xs font-bold text-gray-500>{{ dir === "" ? "(ルート)" : dir }}</h2>
        <div grid grid-cols-3 gap-2 lg:grid-cols-4>
          <button
            v-for="file in files"
            :key="file.path"
            class="file-card"
            border
            rounded-lg
            p-2
            text-left
            cursor-pointer
            :class="
              file.path === focus.file
                ? 'border-cyan-600 bg-cyan-50'
                : 'border-gray-200 bg-white hover:border-gray-400'
            "
            :title="file.path"
            @click="zoomInto(file.path)"
          >
            <div flex items-baseline gap-1>
              <span truncate text-xs font-bold font-mono>
                {{ file.path.split("/").pop() }}
              </span>
              <!-- 現在ファイルとの呼び出し関係 (M2 前段、4.4 のチップマークと同じ語彙) -->
              <span
                v-if="relationOf(file.path)"
                class="file-relation"
                shrink-0
                text-xs
                text-cyan-600
                :title="relationOf(file.path)!.note"
              >
                {{ relationOf(file.path)!.mark }}
              </span>
            </div>
            <div text-xs text-gray-400>
              {{ file.error ? "解析エラー" : `${file.functions.length} 関数` }}
            </div>
            <!-- シグネチャ抜粋 (盾の中身の気配 — 最大3つ) -->
            <ul mt-1 space-y-0.5>
              <li
                v-for="fn in file.functions.slice(0, 3)"
                :key="fn.qualified"
                truncate
                text-xs
                text-gray-600
                font-mono
              >
                {{ fn.qualified }}
              </li>
              <li v-if="file.functions.length > 3" text-xs text-gray-400>
                … 他 {{ file.functions.length - 3 }}
              </li>
            </ul>
          </button>
        </div>
      </section>
    </template>
  </div>
</template>
