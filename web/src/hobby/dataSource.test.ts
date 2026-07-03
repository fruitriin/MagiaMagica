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

  it("同一ソースの繰り返し呼び出しは wasm を再実行しない (メモ化)", () => {
    let listCalls = 0;
    let spellCalls = 0;
    const ds = makeWasmDataSource({
      list: () => {
        listCalls += 1;
        return JSON.stringify({ functions: [] });
      },
      spell: (_source, fnName) => {
        spellCalls += 1;
        return JSON.stringify({ qualified: fnName });
      },
    });
    ds.list("src");
    ds.list("src");
    ds.spell("src", "g");
    ds.spell("src", "g");
    expect(listCalls).toBe(1);
    expect(spellCalls).toBe(1);
    // 別関数は別エントリとして評価される。
    ds.spell("src", "h");
    expect(spellCalls).toBe(2);
  });

  it("ソースが変わればキャッシュを捨てて再実行する", () => {
    let calls = 0;
    const ds = makeWasmDataSource(
      fakeWasm({
        list: () => {
          calls += 1;
          return JSON.stringify({ functions: [] });
        },
      }),
    );
    ds.list("fn a() {}");
    ds.list("fn b() {}");
    ds.list("fn b() {}");
    expect(calls).toBe(2);
  });

  it("wasm の例外はキャッシュされない (再試行で再評価される)", () => {
    let calls = 0;
    const ds = makeWasmDataSource(
      fakeWasm({
        list: () => {
          calls += 1;
          if (calls === 1) throw new Error("一時的なエラー");
          return JSON.stringify({ functions: [] });
        },
      }),
    );
    expect(() => ds.list("src")).toThrow("一時的なエラー");
    expect(ds.list("src")).toEqual([]);
  });
});
