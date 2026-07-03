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

/** 注入された wasm 境界から DataSource を組み立てる (純粋・テスト可能)。
 *
 *  同一ソースへの繰り返し呼び出し (関数ドロップダウンの往復 — デモの主動線) は
 *  結果をメモ化して wasm の再パースなしで返す。ソースが変われば丸ごと捨てる —
 *  デモは「今のソース」1つしか見ないので、キャッシュは直近1ソース分で足りる。
 *  wasm の例外 (構文エラー等) はキャッシュせず素通しする (再試行で再評価される)。 */
export function makeWasmDataSource(wasm: MagiaWasm): WasmDataSource {
  let cachedSource: string | null = null;
  let listCache: FunctionMeta[] | null = null;
  const spellCache = new Map<string, HobbySpellResponse>();
  function ensureSource(source: string): void {
    if (source === cachedSource) return;
    cachedSource = source;
    listCache = null;
    spellCache.clear();
  }
  return {
    list(source) {
      ensureSource(source);
      if (listCache === null) {
        listCache = (JSON.parse(wasm.list(source)) as { functions: FunctionMeta[] }).functions;
      }
      return listCache;
    },
    spell(source, fnName) {
      ensureSource(source);
      const hit = spellCache.get(fnName);
      if (hit !== undefined) return hit;
      const parsed = JSON.parse(wasm.spell(source, fnName)) as HobbySpellResponse;
      spellCache.set(fnName, parsed);
      return parsed;
    },
  };
}

// 生成 wasm を初期化する入口 (loadWasmDataSource) は wasmLoader.ts に分離してある —
// 生成物 (src/hobby/wasm/、.gitignore 済み) への import を持つモジュールをテストが
// 触らないようにするため (未生成のチェックアウトでも単体テストが走る)。
