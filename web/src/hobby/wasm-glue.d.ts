// 生成 wasm グルー (scripts/build-hobby-wasm.sh で生成、src/hobby/wasm/ は .gitignore 済み)
// の型境界。実体の .d.ts は生成物が無い環境 (fresh checkout / CI の main ビルド) では
// 存在しないため、ここで安定したワイルドカード宣言を与える。
//
// TS の挙動: 実体が存在すればそちらの .d.ts が優先され、無ければこのワイルドカード宣言が
// フォールバックとして使われる (両立 — 競合しない)。これにより main の `vue-tsc` は wasm
// 未生成でも通り、hobby ビルド時 (wasm 生成後) は実体の正確な型で検証される。
declare module "*/wasm/magia_hobby.js" {
  // パラメータ名は実体 (生成 .d.ts) と一致させる (`fn_name`)。戻り値は使わないので void。
  export function list(source: string): string;
  export function spell(source: string, fn_name: string): string;
  export default function init(wasmUrl?: string | URL): Promise<void>;
}
