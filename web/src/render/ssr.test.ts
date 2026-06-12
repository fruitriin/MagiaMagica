import { describe, expect, it } from "vite-plus/test";

import golden from "../../golden/phase2x/spell_write_document.json";
import { irToSchema } from "../converters/irToSchema.ts";
import type { BelkaIr, IrSpell } from "../types/magia.ts";
import { renderSpellSvg, toStandaloneSvg } from "./ssr.ts";

const IR = golden.ir as IrSpell;

describe("renderSpellSvg (SSR — 動的 UI と同じコンポーネントツリー)", () => {
  it("スタンドアロン SVG (XML) として正しい属性で出力する", async () => {
    const svg = await renderSpellSvg({ ir: IR });
    expect(svg.startsWith("<svg ")).toBe(true);
    // 数値は小数2桁へ丸めて出る (toStandaloneSvg の正規化 — num() と同精度)。
    const round2 = (v: number) => Math.round(v * 100) / 100;
    expect(svg).toContain(`viewBox="${IR.view_box.map(round2).join(" ")}"`);
    // XML invalid / SSR ノイズが残っていないこと (toStandaloneSvg の契約)。
    expect(svg).not.toContain("<!--");
    expect(svg).not.toMatch(/ data-v-/);
    expect(svg).not.toContain('style=""');
    expect(svg).not.toContain("viewbox=");
  });

  it("要素数がスキーマと一致する (取りこぼしなし)", async () => {
    const svg = await renderSpellSvg({ ir: IR });
    const schema = irToSchema(IR);
    // リング + 操作ドット + 召喚印は全て circle 要素 (async 内円は symbol 側)。
    const asyncInner = schema.symbols.filter((s) => s.kind === "async_inner").length;
    const circles = (svg.match(/<circle /g) ?? []).length;
    expect(circles).toBe(
      schema.circles.length + schema.operations.length + schema.glyphs.length + asyncInner,
    );
    // 制御フロー接続線 + 早期リターン線 + 戻り値分岐線。
    expect((svg.match(/<line /g) ?? []).length).toBeGreaterThanOrEqual(schema.edges.length);
    // シグネチャ円弧。
    if (schema.signature) {
      expect(svg).toContain("<textPath");
      expect(svg).toContain("startOffset");
    }
  });

  it("同一入力からは同一出力 (決定論 — golden 比較の前提)", async () => {
    const a = await renderSpellSvg({ ir: IR });
    const b = await renderSpellSvg({ ir: IR });
    expect(a).toBe(b);
  });
});

describe("renderSpellSvg (ベルカ式 — Phase 4.3 M5)", () => {
  it("belka リクエストは BelkaCircle で描く (三極 + 力場 + ラベル)", async () => {
    const belka: BelkaIr = {
      view_box: [-100, -100, 200, 200],
      poles: [
        {
          pole: "genesis",
          x: 0,
          y: -50,
          radius: 26,
          field_radius: 50,
          label_x: 0,
          label_y: -90,
          dots: [{ x: 0, y: -50, effect: "pure" }],
        },
      ],
      flows: [{ x1: 0, y1: -24, x2: 0, y2: 40, width: 1.9, tip_x: 0, tip_y: 46 }],
      signature: { text: "fn t()", x: 0, y: -84 },
    };
    const svg = await renderSpellSvg({ belka });
    expect(svg).toContain("belka-pole");
    expect(svg).toContain("radialGradient");
    expect(svg).toContain("生成");
    expect(svg).toContain("belka-flow-head");
    expect(svg).not.toContain("data-v-");
  });
});

describe("toStandaloneSvg (SSR 出力の XML 正規化)", () => {
  it("hydration コメント・値なし data-v・空 style を落とす", () => {
    const html =
      '<svg viewbox="0 0 1 1"><!--[--><circle data-v-1a2b3c4d style="" r="1"></circle><!--]--></svg>';
    expect(toStandaloneSvg(html)).toBe('<svg viewBox="0 0 1 1"><circle r="1"></circle></svg>');
  });

  it("値つき data-v も落とす", () => {
    expect(toStandaloneSvg('<g data-v-abc123=""></g>')).toBe("<g></g>");
  });
});
