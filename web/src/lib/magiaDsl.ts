// .magia DSL (spec §8) のパレット側サブセット。パレットが扱うのは可視性のみで、
// `effects[カテゴリ]` の絞り込みは render 時適用のため CLI (`magia render --filter`)
// を案内する (inline HTML 版 Phase 2.3 と同じ責務分担)。
// 純関数にして store / コンポーネントから分離する (Vitest の対象、M6)。

import { LAYERS, type LayerName } from "../stores/palette.ts";

export type DslParseResult =
  | { ok: true; visible: Set<LayerName>; note: string }
  | { ok: false; error: string };

/** 現在の可視集合を DSL テキストに書き出す。 */
export function exportDsl(visible: ReadonlySet<LayerName>): string {
  const shown = LAYERS.filter((l) => visible.has(l));
  return shown.length > 0
    ? `show: ${shown.join(" + ")}`
    : `# 全レイヤー非表示\nhide: ${LAYERS.join(" + ")}`;
}

/** DSL テキストをパースして可視集合を返す。エラーは行番号付き (1-origin)。 */
export function parseDsl(text: string): DslParseResult {
  let show: Set<LayerName> | null = null;
  const hide = new Set<LayerName>();
  let hasCategories = false;

  for (const [i, raw] of text.split("\n").entries()) {
    const line = raw.trim();
    if (line === "" || line.startsWith("#")) continue;
    const directive = line.startsWith("show:") ? "show" : line.startsWith("hide:") ? "hide" : null;
    if (directive === null) {
      return { ok: false, error: `${i + 1}行目: show: / hide: のみ使用できます` };
    }
    for (const part of line
      .slice(5)
      .split("+")
      .map((s) => s.trim())
      .filter(Boolean)) {
      if (part.includes("[")) {
        if (directive === "hide") {
          return { ok: false, error: `${i + 1}行目: hide にカテゴリ指定 [...] はできません` };
        }
        hasCategories = true;
      }
      // split は常に1要素以上返すので [0] は実在する (?? "" は noUncheckedIndexedAccess 対策)。
      const name = (part.split("[")[0] ?? "").trim();
      if (!(LAYERS as readonly string[]).includes(name)) {
        return { ok: false, error: `${i + 1}行目: 未知のレイヤー名 \`${name}\`` };
      }
      const layer = name as LayerName;
      if (directive === "show") {
        (show ??= new Set()).add(layer);
      } else {
        hide.add(layer);
      }
    }
  }

  const visible = new Set((show ? [...show] : [...LAYERS]).filter((l) => !hide.has(l)));
  const note = hasCategories
    ? "effects[カテゴリ] の絞り込みは magia render --filter で適用されます"
    : "";
  return { ok: true, visible, note };
}
