// WASM デモ版 (Phase 4.12 M2) のデータ供給層。
// serve の HTTP api.ts と対の関係: あちらは fetch、こちらは wasm の list/spell を呼ぶ。
// JSON 文字列のパースだけを担い、wasm 本体は注入する (テスト可能性 + 情報隠蔽)。
// magia-hobby crate の JSON 契約 (serve /state /spell のサブセット) をそのまま受ける。

import type { FunctionMeta, HobbySpellResponse } from "../types/magia.ts";

/** wasm の薄い関数境界 (テスト時にフェイクへ差し替える最小インターフェース)。
 *  生成 wasm (`magia_hobby.js`) の `list` / `spell` がこの形。 */
export type MagiaWasm = {
  list(source: string): string;
  spell(source: string, fnName: string): string;
};

/** WASM を入力源とするデータ供給。`make` で wasm を注入して作る。 */
export type WasmDataSource = {
  /** ファイル内の関数一覧 (構文エラー時は wasm が例外を投げる)。 */
  list(source: string): FunctionMeta[];
  /** 関数1つの魔法陣レスポンス (未知関数・構文エラー時は wasm が例外を投げる)。 */
  spell(source: string, fnName: string): HobbySpellResponse;
};

/** 注入された wasm 境界から DataSource を組み立てる (純粋・テスト可能)。 */
export function makeWasmDataSource(wasm: MagiaWasm): WasmDataSource {
  return {
    list(source) {
      const parsed = JSON.parse(wasm.list(source)) as { functions: FunctionMeta[] };
      return parsed.functions;
    },
    spell(source, fnName) {
      return JSON.parse(wasm.spell(source, fnName)) as HobbySpellResponse;
    },
  };
}

/** 生成 wasm を初期化して DataSource を返す (ブラウザ実行時の入口)。
 *  動的 import なので、本関数を呼ばない限り wasm はロードされない
 *  (単体テストは `makeWasmDataSource` をフェイクで叩き、ここは通らない)。 */
export async function loadWasmDataSource(): Promise<WasmDataSource> {
  const glue = await import("./wasm/magia_hobby.js");
  const wasmUrl = (await import("./wasm/magia_hobby_bg.wasm?url")).default;
  await glue.default(wasmUrl);
  return makeWasmDataSource({ list: glue.list, spell: glue.spell });
}
