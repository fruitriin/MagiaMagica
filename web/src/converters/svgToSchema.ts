// SVG 文字列 → MagicCircleSchema 変換 (Phase 4.0.7 / 案1)。
//
// **このファイルは Phase 4.0.9 (IR JSON ビルダ) で丸ごと削除される前提**で、
// 他のコードからは Schema 経由でしか触れない (POSD: 捨てる前提の隔離)。
//
// 意味論の復元は SVG の class 属性に依る (data-* 属性は存在しない —
// 4.0.7 着手時に確認した実態。Rust 側レンダラの語彙: main-ring / aux-ring /
// op-dot / summon-glyph / edge-control-flow / signature / layer-*)。
// 既知 class 以外の要素は RawElement として素通しし、表示を維持する
// (複合記号 sym-*、return-path-*、ベルカ式の belka-* がこれに該当)。

import type {
  Circle,
  EffectCategory,
  MagicCircleSchema,
  Operation,
  RawElement,
  RenderStyle,
  SchemaEdge,
  SchemaLayer,
  Signature,
} from "../types/magia.ts";

/** palette.rs の色 → 効果カテゴリの逆引き。
 *  !!! palette.rs の定数・uno.config.ts の theme と3箇所の手動同期 — 色変更時は全て直す
 *  (ずれると effect: null に黙って落ちる。4.0.9 の IR 直結で同期問題ごと消える)。 */
const EFFECT_BY_COLOR: Record<string, EffectCategory> = {
  "#000000": "pure",
  "#1f4dff": "io",
  "#7b3ff5": "network",
  "#1fa341": "db",
  "#7a4a1c": "filesystem",
  "#d92626": "unsafe",
};

/** SVG の g.layer-* クラス → スキーマのレイヤー名。 */
const LAYER_BY_CLASS: Record<string, SchemaLayer> = {
  "layer-control-flow": "control_flow",
  "layer-effects": "effects",
  "layer-type-info": "type_info",
};

export function svgToSchema(svg: string, style: RenderStyle): MagicCircleSchema {
  const doc = new DOMParser().parseFromString(svg, "image/svg+xml");
  const root = doc.documentElement;

  const schema: MagicCircleSchema = {
    signature: null,
    style,
    viewBox: parseViewBox(root.getAttribute("viewBox")),
    circles: [],
    operations: [],
    edges: [],
    glyphs: [],
    raws: [],
  };

  // defs は signature の円弧 path の供給源。それ以外の defs 内容は素通しに含める。
  const arcPaths = new Map<string, string>();
  // seq は id と描画順 z を兼ねる (SVG の出現順 = 元レンダラの z-order)。
  let seq = 0;
  const nextId = (kind: string) => `${kind}-${seq++}`;
  const lastZ = () => seq - 1;

  const visit = (element: Element, layer: SchemaLayer | null) => {
    const cls = element.getAttribute("class") ?? "";
    const tag = element.tagName;

    if (tag === "defs") {
      for (const path of element.querySelectorAll("path[id]")) {
        arcPaths.set(path.getAttribute("id") ?? "", path.getAttribute("d") ?? "");
      }
      return;
    }
    if (tag === "g") {
      // g は**常に**開いて子を個別に振り分ける (レイヤーグループに限らない —
      // g 自体は持たず、レイヤー可視性は各要素の layer フィールドで宣言的に適用する)。
      // このため g が raw に落ちて子要素が二重描画されることはない。
      const childLayer = LAYER_BY_CLASS[cls] ?? layer;
      for (const child of element.children) visit(child, childLayer);
      return;
    }

    if (tag === "circle" && (cls === "main-ring" || cls === "aux-ring")) {
      schema.circles.push(
        parseCircle(element, cls === "main-ring" ? "main" : "aux", layer, nextId, lastZ),
      );
      return;
    }
    if (tag === "circle" && cls === "op-dot") {
      schema.operations.push(parseDot(element, layer, nextId("op"), lastZ()));
      return;
    }
    if (tag === "circle" && cls === "summon-glyph") {
      schema.glyphs.push(parseDot(element, layer, nextId("glyph"), lastZ()));
      return;
    }
    if (tag === "line" && cls === "edge-control-flow") {
      schema.edges.push(parseEdge(element, layer, nextId("edge"), lastZ()));
      return;
    }
    if (tag === "text" && cls === "signature") {
      schema.signature = parseSignature(element, arcPaths);
      return;
    }
    schema.raws.push(rawElement(element, layer, nextId("raw"), lastZ()));
  };

  for (const child of root.children) visit(child, null);
  return schema;
}

function parseViewBox(value: string | null): [number, number, number, number] {
  const nums = (value ?? "").split(/\s+/).map(Number);
  if (nums.length === 4 && nums.every((n) => Number.isFinite(n))) {
    return [nums[0] ?? 0, nums[1] ?? 0, nums[2] ?? 0, nums[3] ?? 0];
  }
  // 0 サイズの viewBox は「何も映らない」— 黙って空白になるとデバッグ困難なので警告を残す。
  console.warn(`svgToSchema: viewBox を解釈できません: ${String(value)}`);
  return [0, 0, 0, 0];
}

const attr = (el: Element, name: string) => Number.parseFloat(el.getAttribute(name) ?? "0");

function parseCircle(
  el: Element,
  role: "main" | "aux",
  layer: SchemaLayer | null,
  nextId: (kind: string) => string,
  lastZ: () => number,
): Circle {
  return {
    id: nextId(role === "main" ? "ring-main" : "ring-aux"),
    z: lastZ(),
    role,
    x: attr(el, "cx"),
    y: attr(el, "cy"),
    radius: attr(el, "r"),
    strokeWidth: attr(el, "stroke-width"),
    layer,
  };
}

/** op-dot と summon-glyph は同形 (circle + fill 色)。 */
function parseDot(el: Element, layer: SchemaLayer | null, id: string, z: number): Operation {
  const color = el.getAttribute("fill") ?? "#000000";
  return {
    id,
    z,
    x: attr(el, "cx"),
    y: attr(el, "cy"),
    radius: attr(el, "r"),
    color,
    effect: EFFECT_BY_COLOR[color] ?? null,
    selectable: true,
    layer,
  };
}

function parseEdge(el: Element, layer: SchemaLayer | null, id: string, z: number): SchemaEdge {
  return {
    id,
    z,
    x1: attr(el, "x1"),
    y1: attr(el, "y1"),
    x2: attr(el, "x2"),
    y2: attr(el, "y2"),
    layer,
    from: null, // SVG からは復元できない — IR 直結 (4.0.9) で埋まる
    to: null,
  };
}

/** ミッドチルダ式は textPath (円弧)、ベルカ式は直線テキスト — 両対応。 */
function parseSignature(el: Element, arcPaths: Map<string, string>): Signature {
  const textPath = el.querySelector("textPath");
  const href = (textPath?.getAttribute("href") ?? "").replace(/^#/, "");
  return {
    text: (textPath ?? el).textContent ?? "",
    arcPath: textPath ? (arcPaths.get(href) ?? "") : null,
    x: attr(el, "x"),
    y: attr(el, "y"),
    fontSize: attr(el, "font-size") || 11,
  };
}

function rawElement(el: Element, layer: SchemaLayer | null, id: string, z: number): RawElement {
  return { id, layer, markup: el.outerHTML, z };
}
