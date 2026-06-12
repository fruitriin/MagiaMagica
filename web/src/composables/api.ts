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

/** ワークスペース俯瞰 (Phase 4.5 M1): 全 .rs の関数一覧 +
 *  ファイル横断の呼び出しエッジ (M2 前段)。 */
export function fetchWorkspace(): Promise<{
  files: {
    path: string;
    dir: string;
    functions: { qualified: string; signature: string }[];
    error?: boolean;
  }[];
  cross_edges: { from_file: string; from: string; to_file: string; to: string }[];
}> {
  return fetchJson("/workspace");
}

/** ワークスペース配下の .rs 一覧 (監視対象の切替候補、Phase 4.4.5)。 */
export function fetchFiles(): Promise<{ files: string[] }> {
  return fetchJson<{ files: string[] }>("/files");
}

/** 監視対象ファイルの切替。成功時はサーバが採用した正規化パスを返す。
 *  失敗 (境界外・不在など) は fetch ヘルパが throw する。 */
export async function postFile(path: string): Promise<string> {
  const res = await fetch("/file", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ path }),
  });
  const body = (await res.json()) as { file?: string; error?: string };
  if (!res.ok || body.file === undefined) {
    throw new Error(body.error ?? `POST /file ${res.status}`);
  }
  return body.file;
}

export function fetchSpell(fn: string, diffRev?: string | null): Promise<SpellResponse> {
  // ピン中心ビュー (Phase 4.1) が標準表示のため、周辺配置を常に併載で取得する。
  // diffRev (Phase 4.3.7) があれば差分強調 (diff_overlay / diff_report) も併載される。
  const diff = diffRev ? `&diff=${encodeURIComponent(diffRev)}` : "";
  return fetchJson<SpellResponse>(`/spell/${encodeURIComponent(fn)}?with=neighbors${diff}`);
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
