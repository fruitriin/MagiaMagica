// 生成 wasm (scripts/build-hobby-wasm.sh の出力) の読み込み口。
// dataSource.ts から分離してあるのは、ここの動的 import が .gitignore 済みの生成物
// (src/hobby/wasm/) を参照するため — wasm 未生成のチェックアウト (fresh clone / CI) でも
// dataSource の単体テストが Vite の import 解決エラーなしに走るよう、生成物に触る
// コードをテストが import しないモジュールへ隔離する。

import { makeWasmDataSource, type WasmDataSource } from "./dataSource.ts";

/** 生成 wasm を初期化して DataSource を返す (ブラウザ実行時の入口)。
 *  動的 import なので、本関数を呼ばない限り wasm はロードされない。 */
export async function loadWasmDataSource(): Promise<WasmDataSource> {
  const glue = await import("./wasm/magia_hobby.js");
  const wasmUrl = (await import("./wasm/magia_hobby_bg.wasm?url")).default;
  // wasm-bindgen 0.2.9x+ の現行 init 形式。裸の URL を渡す位置引数形式は非推奨で、
  // 毎ロード console.warn が出る (将来の CLI 更新で除去されうる)。
  await glue.default({ module_or_path: wasmUrl });
  return makeWasmDataSource({ list: glue.list, spell: glue.spell });
}
