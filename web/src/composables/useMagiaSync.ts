// SSE 購読と store 群の調停。コンポーネントのライフサイクルに乗せるため
// composable にする (store は購読の開始/停止を持たない — connection.ts 参照)。

import { onMounted, onUnmounted } from "vue";

import { connectEvents } from "./api.ts";
import { useConnectionStore } from "../stores/connection.ts";
import { useFocusStore } from "../stores/focus.ts";

/** 接続直後の 1 イベント (serve.rs 仕様) も refresh に使うので初回ロードは不要。 */
export function useMagiaSync() {
  const connection = useConnectionStore();
  const focus = useFocusStore();
  let disconnect: (() => void) | null = null;

  onMounted(() => {
    disconnect = connectEvents({
      onUpdate: (version) => {
        connection.markUpdate(version);
        void focus.refresh();
      },
      onStatus: (status) => connection.markStatus(status),
    });
  });

  onUnmounted(() => {
    disconnect?.();
  });
}
