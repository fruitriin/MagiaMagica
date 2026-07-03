// 共有リンク (Phase 4.12 M3)。ソース (+ 選択関数) を URL fragment に載せ、リンク一本で
// 「この関数の魔法陣」を再現できるようにする (Playground 方式)。サーバ不要で SNS 拡散に効く。
//
// エンコードは base64url(utf8)。圧縮 (deflate 等) は将来 CompressionStream で**この関数に
// 閉じて**差し替えられる — 短い関数なら base64 でも URL 長に十分収まるため、まずは依存なし・
// 同期・全ブラウザで動く素の base64url を採る (計画 (a) 案の精神)。

/** UTF-8 文字列 → base64url (パディングなし)。 */
export function encodeShare(source: string): string {
  const bytes = new TextEncoder().encode(source);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

/** base64url → UTF-8 文字列。不正な入力は例外を投げる (呼び出し側で握る)。 */
export function decodeShare(code: string): string {
  // atob はパディング (`=`) を要求する (古い Safari は欠落で例外)。encodeShare は外すので補う。
  const remainder = code.length % 4;
  const padded = remainder === 0 ? code : code + "=".repeat(4 - remainder);
  const base64 = padded.replace(/-/g, "+").replace(/_/g, "/");
  const binary = atob(base64);
  const bytes = Uint8Array.from(binary, (ch) => ch.charCodeAt(0));
  // fatal 必須: 既定の TextDecoder は壊れた UTF-8 (SNS で途中切断されたリンク等) を
  // U+FFFD に置換して「成功」してしまい、prefill フォールバックが効かなくなる。
  return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
}

/** 現在の状態を `#code=...(&fn=...&style=...)` の fragment にした共有 URL を組み立てる。
 *  `base` は通常 `location.origin + location.pathname`。`base` 内の既存ハッシュは落とす
 *  (`##...` になるのを防ぐ — 呼び出し側が pathname を渡せば通常は無害だが防御する)。
 *  `style` は既定 (ミッドチルダ式) なら null で省略 — 「この見た目を見て」という共有の
 *  意図を受け手でも再現するため、非既定のときだけ運ぶ。 */
export function buildShareUrl(
  base: string,
  source: string,
  fn: string | null,
  style: string | null = null,
): string {
  const safeBase = base.replace(/#.*$/, "");
  const params = new URLSearchParams();
  params.set("code", encodeShare(source));
  if (fn !== null) params.set("fn", fn);
  if (style !== null) params.set("style", style);
  return `${safeBase}#${params.toString()}`;
}

/** `location.hash` から共有状態を取り出す。`code` が無ければ空オブジェクト。
 *  `code` のデコードに失敗したら **fn / style も含めて全体を捨てる** (壊れた source に
 *  対して残りだけ復元する価値がないため)。呼び出し側は source 欠落を prefill に倒す。 */
export function parseShareHash(hash: string): { source?: string; fn?: string; style?: string } {
  const raw = hash.startsWith("#") ? hash.slice(1) : hash;
  if (raw === "") return {};
  const params = new URLSearchParams(raw);
  const code = params.get("code");
  if (code === null) return {};
  const result: { source?: string; fn?: string; style?: string } = {};
  try {
    result.source = decodeShare(code);
  } catch {
    // 壊れた共有リンク — source は載せずに返す (prefill フォールバック)。
    return {};
  }
  const fn = params.get("fn");
  if (fn !== null) result.fn = fn;
  const style = params.get("style");
  if (style !== null) result.style = style;
  return result;
}
