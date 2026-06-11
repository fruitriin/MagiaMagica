import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vite-plus/test";

import type { SpellResponse, StateResponse } from "../types/magia.ts";
import { useFocusStore } from "./focus.ts";
import { useSourceStore } from "./source.ts";

const STATE: StateResponse = {
  error: null,
  file: "demo.rs",
  version: 1,
  functions: [
    {
      name: "greet",
      qualified: "greet",
      signature: "fn greet()",
      start_line: 1,
      end_line: 3,
      impl_context: null,
    },
    {
      name: "cast",
      qualified: "Caster::cast",
      signature: "fn cast(&self)",
      start_line: 5,
      end_line: 7,
      impl_context: "Caster",
    },
  ],
};

function spellFor(qualified: string): SpellResponse {
  return {
    qualified,
    signature: `fn ${qualified}`,
    source_html: `<pre>${qualified}</pre>`,
    start_line: 1,
    ir: {
      view_box: [-100, -100, 200, 200],
      rings: [
        {
          id: 0,
          role: "main",
          x: 0,
          y: 0,
          radius: 120,
          is_async: false,
          symbol: null,
          early_return: null,
          operations: [{ x: 104, y: 0, radius: 3.5, effect: "pure" }],
        },
      ],
      glyphs: [],
      edges: [],
      signature: { text: `fn ${qualified}`, arc_path: "M-100,0 A100,100 0 0 1 100,0" },
      return_branch: null,
    },
    svg_belka: `<svg data-belka="${qualified}"></svg>`,
    transcript: `関数 ${qualified}`,
  };
}

function mockFetch() {
  vi.stubGlobal(
    "fetch",
    vi.fn(async (path: string) => {
      const qualified = decodeURIComponent((path.split("/spell/")[1] ?? "").split("?")[0] ?? "");
      const body = path === "/state" ? STATE : spellFor(qualified);
      return new Response(JSON.stringify(body), { status: 200 });
    }),
  );
}

beforeEach(() => {
  setActivePinia(createPinia());
  mockFetch();
});

describe("useFocusStore", () => {
  it("selectFunction は qualified 名で照合する (impl メソッド)", async () => {
    const focus = useFocusStore();
    await focus.loadState();
    await focus.selectFunction("Caster::cast");
    expect(focus.currentFn).toBe("Caster::cast");
    expect(focus.spell?.qualified).toBe("Caster::cast");
  });

  it("一覧にない関数・未指定は先頭関数へ fallback する", async () => {
    const focus = useFocusStore();
    await focus.loadState();
    await focus.selectFunction("no_such_fn");
    expect(focus.currentFn).toBe("greet");
    await focus.selectFunction(null);
    expect(focus.currentFn).toBe("greet");
  });

  it("取得成功で source store にハイライト HTML を分配する", async () => {
    const focus = useFocusStore();
    await focus.loadState();
    await focus.selectFunction("greet");
    const source = useSourceStore();
    expect(source.sourceHtml).toBe("<pre>greet</pre>");
    expect(source.startLine).toBe(1);
  });

  it("取得失敗時は直前の spell を保持して loadError に記録する", async () => {
    const focus = useFocusStore();
    await focus.loadState();
    await focus.selectFunction("greet");
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response("gone", { status: 500 })),
    );
    await focus.selectFunction("Caster::cast");
    expect(focus.spell?.qualified).toBe("greet"); // 直前の表示を消さない
    expect(focus.loadError).not.toBeNull();
  });

  it("spell は配置済み IR とベルカ SVG の両方を保持する (表示切替は MagicCircleView)", async () => {
    const focus = useFocusStore();
    await focus.loadState();
    await focus.selectFunction("greet");
    expect(focus.spell?.ir.rings[0]?.role).toBe("main");
    expect(focus.spell?.svg_belka).toContain("data-belka");
  });
});
