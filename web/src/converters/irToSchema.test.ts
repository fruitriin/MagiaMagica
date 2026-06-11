import { describe, expect, it } from "vite-plus/test";

import golden from "../../golden/phase2x/spell_write_document.json";
import type { IrSpell } from "../types/magia.ts";
import { irToSchema } from "./irToSchema.ts";

// golden は実レンダラ出力 (write_document、自己ホスティング、spec v0.3 §16 契約)。
const IR = golden.ir as IrSpell;

describe("irToSchema (write_document golden)", () => {
  const schema = irToSchema(IR);

  it("viewBox を素通しする (レイアウトは Rust 確定値)", () => {
    expect(schema.viewBox).toEqual(IR.view_box);
  });

  it("メインリング1 + 補助リング群、stroke 幅は役割で決まる", () => {
    const mains = schema.circles.filter((c) => c.role === "main");
    expect(mains).toHaveLength(1);
    expect(mains[0]).toMatchObject({ strokeWidth: 2, layer: "control_flow" });
    expect(schema.circles.filter((c) => c.role === "aux").every((c) => c.strokeWidth === 1.5)).toBe(
      true,
    );
  });

  it("操作ドットは配置済み座標と効果色 (palette 同語彙) を持つ", () => {
    expect(schema.operations.length).toBeGreaterThan(0);
    expect(schema.operations.every((op) => op.layer === "effects")).toBe(true);
    // write_document の操作は pure (黒) のみ
    expect(new Set(schema.operations.map((op) => op.color))).toEqual(new Set(["#000000"]));
  });

  it("召喚印は効果カテゴリの色 (io = 青を含む)", () => {
    expect(schema.glyphs.some((g) => g.color === "#1f4dff" && g.effect === "io")).toBe(true);
  });

  it("エッジ端点はリング表面 (中心間距離から両半径を引いた線分)", () => {
    expect(schema.edges).toHaveLength(IR.edges.length);
    const edge = schema.edges[0];
    expect(edge?.from).not.toBeNull(); // IR 直結で from/to が埋まる (4.0.7 では null だった)
    const fromNode = [...IR.rings, ...IR.glyphs].find((n) => String(n.id) === edge?.from);
    expect(fromNode).toBeDefined();
    // 端点は from の中心から半径ぶん離れている
    const d = Math.hypot(
      (edge?.x1 ?? 0) - (fromNode?.x ?? 0),
      (edge?.y1 ?? 0) - (fromNode?.y ?? 0),
    );
    expect(d).toBeCloseTo(fromNode?.radius ?? 0, 6);
  });

  it("記号は意味論 (branch / early_return) として展開される", () => {
    const kinds = new Set(schema.symbols.map((s) => s.kind));
    expect(kinds.has("branch")).toBe(true); // write_document は分岐補助リングを持つ
    expect(schema.symbols.every((s) => s.layer !== null)).toBe(true);
  });

  it("シグネチャは円弧 path を持つ", () => {
    expect(schema.signature?.text).toContain("fn write_document");
    expect(schema.signature?.arcPath).toMatch(/^M/);
  });

  it("z は旧レンダラのレイヤー順 (edges/rings/syms → ops/glyphs → type_info)", () => {
    const maxControlZ = Math.max(
      ...schema.edges.map((e) => e.z),
      ...schema.circles.map((c) => c.z),
    );
    const minEffectsZ = Math.min(
      ...schema.operations.map((o) => o.z),
      ...schema.glyphs.map((g) => g.z),
    );
    expect(maxControlZ).toBeLessThan(minEffectsZ);
  });

  it("id は重複しない", () => {
    const ids = [
      ...schema.circles,
      ...schema.operations,
      ...schema.glyphs,
      ...schema.edges,
      ...schema.symbols,
    ].map((e) => e.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("raws は IR 直結では常に空 (素通しが要る要素は存在しない)", () => {
    expect(schema.raws).toHaveLength(0);
  });
});

describe("irToSchema (合成ケース)", () => {
  it("async リングは async_inner 記号になり、return_branch は type_info 層", () => {
    const ir: IrSpell = {
      view_box: [-10, -10, 20, 20],
      rings: [
        {
          id: 0,
          role: "main",
          x: 0,
          y: 0,
          radius: 120,
          is_async: true,
          symbol: null,
          early_return: [-1, 0],
          operations: [],
          guard_span: null,
        },
      ],
      glyphs: [],
      edges: [],
      signature: null,
      return_branch: [-120, 0],
    };
    const schema = irToSchema(ir);
    const kinds = schema.symbols.map((s) => s.kind);
    expect(kinds).toContain("async_inner");
    expect(kinds).toContain("early_return");
    const rb = schema.symbols.find((s) => s.kind === "return_branch");
    expect(rb).toMatchObject({ x: -120, y: 0, layer: "type_info" });
  });

  it("壊れたエッジ参照は描かないだけで失敗しない", () => {
    const ir: IrSpell = {
      view_box: [0, 0, 10, 10],
      rings: [],
      glyphs: [],
      edges: [{ from: 1, to: 2 }],
      signature: null,
      return_branch: null,
    };
    expect(irToSchema(ir).edges).toHaveLength(0);
  });
});
