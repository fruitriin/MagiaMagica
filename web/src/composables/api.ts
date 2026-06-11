// magia serve (Rust) の API を型付きでラップする層。
// コンポーネントはここを直接呼ばず、Pinia store 経由で使う (POSD「深いモジュール」)。

import type { SpellResponse, StateResponse } from "../types/magia.ts";

async function fetchJson<T>(path: string): Promise<T> {
  const res = await fetch(path);
  if (!res.ok) {
    throw new Error(`${path} ${res.status}`);
  }
  return res.json() as Promise<T>;
}

export function fetchState(): Promise<StateResponse> {
  return fetchJson<StateResponse>("/state");
}

export function fetchSpell(fn: string): Promise<SpellResponse> {
  // ピン中心ビュー (Phase 4.1) が標準表示のため、周辺配置を常に併載で取得する。
  return fetchJson<SpellResponse>(`/spell/${encodeURIComponent(fn)}?with=neighbors`);
}

/**
 * `/events` (SSE) を購読する。サーバはファイル更新ごとに `data: <version>` を
 * 1イベント流す (接続直後にも 1 イベント来る — serve.rs の仕様)。
 * EventSource は切断時に自動再接続するため、呼び出し側は onUpdate だけ気にすればよい。
 */
export function connectEvents(handlers: {
  onUpdate: (version: string) => void;
  /** "connecting" は接続前の初期値で EventSource からは遷移しないため、ここには現れない。 */
  onStatus: (status: "open" | "error") => void;
}): () => void {
  const source = new EventSource("/events");
  source.onopen = () => handlers.onStatus("open");
  source.onerror = () => handlers.onStatus("error");
  source.onmessage = (event: MessageEvent<string>) => handlers.onUpdate(event.data);
  return () => source.close();
}
