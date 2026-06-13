// 配置済み IR (spec v0.3 §16) → MagicCircleSchema 変換 (Phase 4.0.9 / 案2)。
//
// Rust が確定したレイアウト (座標・半径・操作配置・シグネチャ円弧) を
// 描画スキーマへ展開する。Vue 側でレイアウトを再計算しない (POSD 分担)。
// z (描画順) は旧 SVG レンダラのレイヤー順 (control_flow → effects → type_info →
// signature) を踏襲し、重なりの z-order を 4.0.7 以前と一致させる。

import type { ControlSymbol, EffectCategory, IrSpell, MagicCircleSchema } from "../types/magia.ts";

/** 効果カテゴリ → 色 (**色の正はここ** — Phase 4.3 M5 で Rust 側の色定数は削除済み)。
 *  !!! uno.config.ts の theme / BelkaCircle.vue の DOT_COLOR と手動同期。 */
const COLOR_BY_EFFECT: Record<EffectCategory, string> = {
  pure: "#000000",
  io: "#1f4dff",
  network: "#7b3ff5",
  db: "#1fa341",
  filesystem: "#7a4a1c",
  unsafe: "#d92626",
};

/** 戻り値分岐線 (return_branch) の長さ。Rust 側 layout/constants.rs:65 の
 *  RETURN_BRANCH_LENGTH (26.0) と同値必須 (描画定数、spec §16)。 */
export const RETURN_BRANCH_LENGTH = 26;

/** 補助陣ラベルの弧テキスト用パスを組み立てる (リング上端の半円弧)。
 *  ラベルが長い場合は signatureFit と同じ近似で fit する (チップ・シグネチャと
 *  同じ規約で省略を統一)。 */
function buildRingLabel(
  ring: { id: number; x: number; y: number; radius: number; label?: string },
  z: number,
) {
  // 弧の半径はリングの少し外 (記号と被らない外周ぎりぎり)。
  const arcRadius = ring.radius + 6;
  // 上半円 (9時 → 12時 → 3時) — シグネチャ円弧と同じ向き。
  const arcPath = `M ${ring.x - arcRadius} ${ring.y} A ${arcRadius} ${arcRadius} 0 0 1 ${ring.x + arcRadius} ${ring.y}`;
  // fit (charWidthRatio 0.65、fontSize 既定 8 — 補助陣はメインより小さい)。
  const arcLength = Math.PI * arcRadius;
  const CHAR_WIDTH_RATIO = 0.65;
  const MIN_FONT_SIZE = 6;
  const defaultFontSize = 9;
  const text = ring.label ?? "";
  const widthAt = (size: number) => text.length * CHAR_WIDTH_RATIO * size;
  let fontSize = defaultFontSize;
  let display = text;
  if (widthAt(fontSize) > arcLength) {
    fontSize = Math.max(
      MIN_FONT_SIZE,
      Math.floor((arcLength / (text.length * CHAR_WIDTH_RATIO)) * 10) / 10,
    );
  }
  if (widthAt(fontSize) > arcLength) {
    const maxChars = Math.floor(arcLength / (CHAR_WIDTH_RATIO * fontSize));
    display = `${text.slice(0, Math.max(1, maxChars - 1))}…`;
  }
  return {
    id: `label-${ring.id}`,
    text: display,
    arcPath,
    fontSize,
    layer: "type_info" as const,
    z,
  };
}

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
    entrySigns: [],
    ringLabels: [],
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
      chain: edge.kind === "chain",
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
      irId: ring.id,
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
    // 入口サイン (Phase 4.0.6 後半): 実行順の出発点をメインリング 3 時に置く。
    if (ring.role === "main") {
      schema.entrySigns.push({
        id: `entry-${ring.id}`,
        x: ring.x,
        y: ring.y,
        radius: ring.radius,
        layer: "control_flow",
        z: nextZ(),
      });
    }
    // 補助陣ラベル (Phase 4.0.6 後半): 上端の弧テキスト。
    if (ring.role === "aux" && ring.label !== undefined && ring.label.length > 0) {
      schema.ringLabels.push(buildRingLabel(ring, nextZ()));
    }
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
        irKey: `${ring.id}-${index}`,
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
      callTarget: glyph.call_target,
      irId: glyph.id,
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
      arcRadius: ir.signature.arc_radius,
      x: 0,
      y: 0,
      fontSize: 11,
      // 組み立て表示用の部品 (細部修正 2026-06-12)。SignatureArc が
      // 変数名/型名チェックボックスに応じて text と組み立てを切り替える。
      name: ir.signature.name,
      args: ir.signature.args,
      ret: ir.signature.ret,
    };
  }

  return schema;
}
