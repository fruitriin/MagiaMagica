import { describe, expect, it } from "vite-plus/test";

import golden from "../../golden/phase2x/spell_write_document.json";
import { svgToSchema } from "./svgToSchema.ts";

// golden は Phase 4.0.5 M0 で取得した実レンダラ出力 (write_document、自己ホスティング)。
const SVG = golden.svg;
const SVG_BELKA = golden.svg_belka;

describe("svgToSchema (ミッドチルダ式)", () => {
  const schema = svgToSchema(SVG, "midchilda");

  it("viewBox を数値で取り出す", () => {
    expect(schema.viewBox).toEqual([-464, -339.23, 664, 678.45]);
  });

  it("メインリング1 + 補助リング群を復元する", () => {
    const mains = schema.circles.filter((c) => c.role === "main");
    expect(mains).toHaveLength(1);
    expect(mains[0]).toMatchObject({ x: 0, y: 0, radius: 120, layer: "control_flow" });
    expect(schema.circles.filter((c) => c.role === "aux").length).toBeGreaterThan(0);
  });

  it("操作ドットを effects レイヤー付きで復元し、色から効果カテゴリを引く", () => {
    expect(schema.operations.length).toBeGreaterThan(0);
    expect(schema.operations.every((op) => op.layer === "effects")).toBe(true);
    expect(schema.operations.every((op) => op.selectable)).toBe(true);
    // write_document は pure (黒) の操作のみ
    expect(new Set(schema.operations.map((op) => op.effect))).toEqual(new Set(["pure"]));
  });

  it("召喚印の色 → 効果カテゴリ (io = 青が含まれる)", () => {
    const effects = new Set(schema.glyphs.map((g) => g.effect));
    expect(effects.has("io")).toBe(true);
    expect(effects.has("pure")).toBe(true);
  });

  it("接続線を座標つきで復元する (from/to は 4.0.9 まで未定)", () => {
    expect(schema.edges.length).toBeGreaterThan(0);
    const edge = schema.edges[0];
    expect(edge?.from).toBeNull();
    expect(Number.isFinite(edge?.x1)).toBe(true);
  });

  it("シグネチャは textPath の本文と円弧 path を持つ", () => {
    expect(schema.signature?.text).toContain("fn write_document");
    expect(schema.signature?.arcPath).toMatch(/^M-396/);
  });

  it("複合記号 (sym-* 等) は raw 素通しに落ち、レイヤー所属を保つ", () => {
    expect(schema.raws.some((r) => r.markup.includes("sym-branch"))).toBe(true);
    const branch = schema.raws.find((r) => r.markup.includes("sym-branch"));
    expect(branch?.layer).toBe("control_flow");
  });

  it("取り溢しゼロ: SVG の描画要素数とスキーマ要素数が一致する", () => {
    // <defs> と <g> は構造要素 (スキーマでは viewBox/layer に正規化) なので除外。
    // `text` の後の [ >] により <textPath> にはマッチしない (text の子要素として数えない)。
    const drawable = (SVG.match(/<(circle|line|path|polygon|text)[ >]/g) ?? []).length;
    const sigArcInDefs = 1; // defs 内の sig-arc path は signature.arcPath に畳まれる
    const inSchema =
      schema.circles.length +
      schema.operations.length +
      schema.glyphs.length +
      schema.edges.length +
      schema.raws.length +
      (schema.signature ? 1 + sigArcInDefs : 0);
    expect(inSchema).toBe(drawable);
  });

  it("id は出現順の一時識別子 (重複なし)", () => {
    const ids = [
      ...schema.circles,
      ...schema.operations,
      ...schema.glyphs,
      ...schema.edges,
      ...schema.raws,
    ].map((e) => e.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});

describe("svgToSchema (ベルカ式)", () => {
  const schema = svgToSchema(SVG_BELKA, "belka");

  it("操作ドットとシグネチャは共有語彙として復元され、力場は raw 素通しになる", () => {
    expect(schema.style).toBe("belka");
    expect(schema.operations.length).toBeGreaterThan(0);
    expect(schema.signature?.text).toContain("fn write_document");
    expect(schema.raws.some((r) => r.markup.includes("belka-pole"))).toBe(true);
    // ミッドチルダ専用の意味論は出ない
    expect(schema.circles).toHaveLength(0);
    expect(schema.edges).toHaveLength(0);
  });
});
