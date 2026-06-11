// 配置済み IR (spec v0.3 §16) → MagicCircleSchema 変換 (Phase 4.0.9 / 案2)。
//
// Rust が確定したレイアウト (座標・半径・操作配置・シグネチャ円弧) を
// 描画スキーマへ展開する。Vue 側でレイアウトを再計算しない (POSD 分担)。
// z (描画順) は旧 SVG レンダラのレイヤー順 (control_flow → effects → type_info →
// signature) を踏襲し、重なりの z-order を 4.0.7 以前と一致させる。

import type { ControlSymbol, EffectCategory, IrSpell, MagicCircleSchema } from "../types/magia.ts";

/** 効果カテゴリ → 色 (palette.rs / uno.config.ts theme と同語彙の正引き)。
 *  !!! palette.rs の定数・uno.config.ts の theme と手動同期 — 色変更時は全て直す。 */
const COLOR_BY_EFFECT: Record<EffectCategory, string> = {
  pure: "#000000",
  io: "#1f4dff",
  network: "#7b3ff5",
  db: "#1fa341",
  filesystem: "#7a4a1c",
  unsafe: "#d92626",
};

/** 戻り値分岐線 (return_branch) の長さ。Rust 側 layout/constants.rs の
 *  RETURN_BRANCH_LENGTH と同値 (描画定数、spec §16)。 */
export const RETURN_BRANCH_LENGTH = 30;

export function irToSchema(ir: IrSpell): MagicCircleSchema {
  let seq = 0;
  const nextZ = () => seq++;

  const schema: MagicCircleSchema = {
    signature: null,
    style: "midchilda",
    viewBox: ir.view_box,
    circles: [],
    operations: [],
    edges: [],
    glyphs: [],
    symbols: [],
    raws: [],
  };

  // リング/グリフの中心・半径の索引 (エッジ端点の計算に使う)。
  const nodes = new Map<number, { x: number; y: number; radius: number }>();
  for (const ring of ir.rings) nodes.set(ring.id, ring);
  for (const glyph of ir.glyphs) nodes.set(glyph.id, glyph);

  // --- layer-control-flow: 接続線 → リング → 記号 (旧レンダラと同順) ---
  for (const [index, edge] of ir.edges.entries()) {
    const from = nodes.get(edge.from);
    const to = nodes.get(edge.to);
    if (!from || !to) continue; // 壊れた参照への防御 (描かないだけで失敗しない)
    const dx = to.x - from.x;
    const dy = to.y - from.y;
    const length = Math.hypot(dx, dy);
    if (length < 1e-6) continue;
    const ux = dx / length;
    const uy = dy / length;
    schema.edges.push({
      id: `edge-${index}`,
      z: nextZ(),
      x1: from.x + ux * from.radius,
      y1: from.y + uy * from.radius,
      x2: to.x - ux * to.radius,
      y2: to.y - uy * to.radius,
      layer: "control_flow",
      from: String(edge.from),
      to: String(edge.to),
    });
  }

  for (const ring of ir.rings) {
    schema.circles.push({
      id: `ring-${ring.id}`,
      z: nextZ(),
      role: ring.role,
      x: ring.x,
      y: ring.y,
      radius: ring.radius,
      strokeWidth: ring.role === "main" ? 2 : 1.5,
      layer: "control_flow",
    });
    const symbol = (kind: ControlSymbol["kind"], direction: [number, number] = [0, 0]) => {
      schema.symbols.push({
        id: `sym-${kind}-${ring.id}`,
        kind,
        x: ring.x,
        y: ring.y,
        radius: ring.radius,
        direction,
        layer: "control_flow",
        z: nextZ(),
      });
    };
    if (ring.is_async) symbol("async_inner");
    if (ring.symbol !== null) symbol(ring.symbol);
    if (ring.early_return !== null) symbol("early_return", ring.early_return);
  }

  // --- layer-effects: 操作ドット → 召喚印 ---
  for (const ring of ir.rings) {
    for (const [index, op] of ring.operations.entries()) {
      schema.operations.push({
        id: `op-${ring.id}-${index}`,
        z: nextZ(),
        x: op.x,
        y: op.y,
        radius: op.radius,
        color: COLOR_BY_EFFECT[op.effect],
        effect: op.effect,
        selectable: true,
        layer: "effects",
      });
    }
  }
  for (const glyph of ir.glyphs) {
    schema.glyphs.push({
      id: `glyph-${glyph.id}`,
      z: nextZ(),
      x: glyph.x,
      y: glyph.y,
      radius: glyph.radius,
      color: COLOR_BY_EFFECT[glyph.effect],
      effect: glyph.effect,
      selectable: true,
      layer: "effects",
    });
  }

  // --- layer-type-info: 戻り値分岐 ---
  if (ir.return_branch !== null) {
    const [x, y] = ir.return_branch;
    schema.symbols.push({
      id: "sym-return-branch",
      kind: "return_branch",
      x,
      y,
      radius: 0,
      direction: [0, 0],
      layer: "type_info",
      z: nextZ(),
    });
  }

  // --- シグネチャ (レイヤー外、最後に描く) ---
  if (ir.signature !== null) {
    schema.signature = {
      text: ir.signature.text,
      arcPath: ir.signature.arc_path,
      x: 0,
      y: 0,
      fontSize: 11,
    };
  }

  return schema;
}
