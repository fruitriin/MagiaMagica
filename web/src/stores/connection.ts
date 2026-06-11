// SSE 接続状態。誰が再描画を引き起こしたか (version) と接続の健全性を持つ。
// 購読の開始・更新時のリロード調停は composable 側 (useMagiaSync) が行い、
// この store は状態の置き場に徹する。

import { defineStore } from "pinia";
import { ref } from "vue";

export type ConnectionStatus = "connecting" | "open" | "error";

export const useConnectionStore = defineStore("connection", () => {
  const status = ref<ConnectionStatus>("connecting");
  const lastVersion = ref<string | null>(null);
  const lastUpdatedAt = ref<number | null>(null);

  function markUpdate(version: string) {
    lastVersion.value = version;
    lastUpdatedAt.value = Date.now();
  }

  function markStatus(next: ConnectionStatus) {
    status.value = next;
  }

  return { status, lastVersion, lastUpdatedAt, markUpdate, markStatus };
});
