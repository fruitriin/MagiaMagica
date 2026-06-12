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
          operations: [{ x: 104, y: 0, radius: 3.5, effect: "pure", source_span: null }],
          guard_span: null,
        },
      ],
      glyphs: [],
      edges: [],
      signature: { text: `fn ${qualified}`, arc_path: "M-100,0 A100,100 0 0 1 100,0" },
      return_branch: null,
    },
    belka_ir: { view_box: [0, 0, 10, 10], poles: [], flows: [], signature: null },
    call_excerpts: {},
    op_excerpts: {},
    ring_excerpts: {},
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

  it("spell は両式の配置済み IR を保持する (表示切替は MagicCircleView)", async () => {
    const focus = useFocusStore();
    await focus.loadState();
    await focus.selectFunction("greet");
    expect(focus.spell?.ir.rings[0]?.role).toBe("main");
    expect(focus.spell?.belka_ir.poles).toEqual([]);
  });
});

describe("useFocusStore — Spell Diff (Phase 4.3.7)", () => {
  it("setDiffRev は diff クエリ付きで取り直す (空・空白はクリア)", async () => {
    const focus = useFocusStore();
    await focus.loadState();
    await focus.selectFunction("greet");
    await focus.setDiffRev("HEAD~1");
    // mock の fetch は path 文字列で呼ばれる (mockFetch の契約)。
    const calls = vi.mocked(globalThis.fetch).mock.calls.map((c) => c[0] as string);
    expect(calls.some((u) => u.includes("diff=HEAD~1"))).toBe(true);
    expect(focus.diffRev).toBe("HEAD~1");
    // 空白はクリア (null) に正規化。
    await focus.setDiffRev("  ");
    expect(focus.diffRev).toBeNull();
  });

  it("同値の setDiffRev は再取得しない", async () => {
    const focus = useFocusStore();
    await focus.loadState();
    await focus.selectFunction("greet");
    await focus.setDiffRev("main");
    const before = vi.mocked(globalThis.fetch).mock.calls.length;
    await focus.setDiffRev("main");
    expect(vi.mocked(globalThis.fetch).mock.calls.length).toBe(before);
  });
});

describe("useFocusStore — インスペクタの refresh 追従 (glyph id は再採番される)", () => {
  const glyph = (id: number, callTarget: string) => ({
    id,
    x: 150,
    y: 0,
    radius: 9,
    effect: "io" as const,
    call_target: callTarget,
    source_span: null,
  });

  /** /spell 応答の glyphs を差し替えた fetch を立てる (refresh 後の世界)。 */
  function mockFetchWithGlyphs(glyphs: ReturnType<typeof glyph>[]) {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (path: string) => {
        const qualified = decodeURIComponent((path.split("/spell/")[1] ?? "").split("?")[0] ?? "");
        const spell = spellFor(qualified);
        spell.ir.glyphs = glyphs;
        const body = path === "/state" ? STATE : spell;
        return new Response(JSON.stringify(body), { status: 200 });
      }),
    );
  }

  it("同 id が同名のまま残っていれば付け替えない", async () => {
    const focus = useFocusStore();
    await focus.loadState();
    await focus.selectFunction("greet");
    focus.inspectCall(".map", 3, 100, 100);
    mockFetchWithGlyphs([glyph(3, ".map")]);
    await focus.refresh();
    expect(focus.inspectedCall?.glyphIrId).toBe(3);
  });

  it("旧 id が消えても同名召喚印が一意なら新 id へ付け替える", async () => {
    const focus = useFocusStore();
    await focus.loadState();
    await focus.selectFunction("greet");
    focus.inspectCall(".map", 3, 100, 100);
    mockFetchWithGlyphs([glyph(7, ".map")]);
    await focus.refresh();
    expect(focus.inspectedCall?.glyphIrId).toBe(7);
  });

  it("同名召喚印が曖昧 (複数) なら式の取り違えを避けて閉じる", async () => {
    const focus = useFocusStore();
    await focus.loadState();
    await focus.selectFunction("greet");
    focus.inspectCall("writeln!", 3, 100, 100);
    mockFetchWithGlyphs([glyph(7, "writeln!"), glyph(8, "writeln!")]);
    await focus.refresh();
    expect(focus.inspectedCall).toBeNull();
  });
});
