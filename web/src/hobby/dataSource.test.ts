import { describe, expect, it } from "vite-plus/test";

import { makeWasmDataSource, type MagiaWasm } from "./dataSource.ts";

// wasm 本体を読まずに JSON 境界だけを検証する。list/spell が返す JSON 文字列を
// フェイクで与え、makeWasmDataSource が型付き値へ正しく畳むことを確認する。
function fakeWasm(over: Partial<MagiaWasm> = {}): MagiaWasm {
  return {
    list: () => JSON.stringify({ functions: [] }),
    spell: () => JSON.stringify({}),
    ...over,
  };
}

describe("makeWasmDataSource", () => {
  it("list は functions 配列を取り出す", () => {
    const ds = makeWasmDataSource(
      fakeWasm({
        list: () =>
          JSON.stringify({
            functions: [
              {
                qualified: "Foo::bar",
                name: "bar",
                impl_context: "Foo",
                signature: "fn bar(&self)",
                start_line: 1,
                end_line: 3,
              },
            ],
          }),
      }),
    );
    const fns = ds.list("…");
    expect(fns).toHaveLength(1);
    expect(fns[0]?.qualified).toBe("Foo::bar");
  });

  it("spell はサブセット応答をそのまま渡す", () => {
    const ds = makeWasmDataSource(
      fakeWasm({
        spell: () =>
          JSON.stringify({
            qualified: "greet",
            signature: "fn greet()",
            ir: { view_box: [0, 0, 10, 10], rings: [] },
            belka_ir: {},
            transcript: "…",
            start_line: 1,
          }),
      }),
    );
    const spell = ds.spell("…", "greet");
    expect(spell.qualified).toBe("greet");
    expect(spell.ir.view_box).toEqual([0, 0, 10, 10]);
  });

  it("wasm の例外 (構文エラー等) は呼び出し側へ伝播する", () => {
    const ds = makeWasmDataSource(
      fakeWasm({
        list: () => {
          throw new Error("構文エラー");
        },
      }),
    );
    expect(() => ds.list("broken")).toThrow("構文エラー");
  });

  it("wasm が不正 JSON を返した場合も例外が伝播する", () => {
    const ds = makeWasmDataSource(fakeWasm({ list: () => "not json" }));
    expect(() => ds.list("src")).toThrow(SyntaxError);
  });
});
