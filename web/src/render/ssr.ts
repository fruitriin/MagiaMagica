// 静止画レンダのエントリ (Phase 4.3)。stdin で IR JSON を受け、Vue SSR で
// 魔法陣を SVG 文字列にして stdout へ書く。動的 UI (serve) と同じ
// コンポーネントツリー (MagicCircle) を使う — 意匠の定義は Vue 1箇所だけ。
//
// 入力: `{ "ir": IrSpell }` (spec v0.3 §16 の配置済み IR)
// 出力: SVG 文字列 (stdout)。失敗時は stderr にエラー + 非0 exit。
// ビルド: vite の SSR ビルド → `bun build --compile` で単一実行ファイル
// (`magia-render`)。.vue を Bun が直接読めないため二段にする。
import { createPinia } from "pinia";
import { createSSRApp, h } from "vue";
import { renderToString } from "vue/server-renderer";

import BelkaCircle from "../components/circle/BelkaCircle.vue";
import MagicCircle from "../components/circle/MagicCircle.vue";
import { irToSchema } from "../converters/irToSchema.ts";
import type {
  BelkaIr,
  DiffMark,
  EffectCategory,
  IrSpell,
  MagicCircleSchema,
  SchemaLayer,
} from "../types/magia.ts";

export type RenderRequest = {
  /** ミッドチルダ式の配置済み IR (`belka` と排他 — どちらか一方)。 */
  ir?: IrSpell;
  /** ベルカ式の配置済み IR (Phase 4.3 M5 — `magia render --style belka`)。 */
  belka?: BelkaIr;
  /** 差分強調 (magia diff --svg、Phase 4.3 M4)。 */
  diff_overlay?: DiffMark[];
  /** 表示するレイヤー (.magia の show/hide 適用結果)。省略 = 全レイヤー。
   *  Rust レンダラの「<g> ごと出さない」と同じく要素自体を出力しない (M5)。 */
  show_layers?: SchemaLayer[];
  /** effects レイヤーのカテゴリ絞り込み (`effects[io, db]`)。省略 = 全カテゴリ。 */
  effects?: EffectCategory[];
};

/** フィルタ (show_layers / effects) をスキーマへ適用する。
 *  レイヤーに属さない要素 (layer: null) はレイヤーフィルタの対象外。
 *  シグネチャと type_info レイヤーは一体 (Rust レンダラの write_defs と同じ括り)。 */
function applyFilter(schema: MagicCircleSchema, request: RenderRequest): MagicCircleSchema {
  const layers = request.show_layers;
  const layerOk = (layer: SchemaLayer | null) =>
    layer === null || layers === undefined || layers.includes(layer);
  const effectOk = (effect: EffectCategory | null) =>
    request.effects === undefined || (effect !== null && request.effects.includes(effect));
  return {
    ...schema,
    circles: schema.circles.filter((c) => layerOk(c.layer)),
    operations: schema.operations.filter((op) => layerOk(op.layer) && effectOk(op.effect)),
    glyphs: schema.glyphs.filter((g) => layerOk(g.layer) && effectOk(g.effect)),
    edges: schema.edges.filter((e) => layerOk(e.layer)),
    symbols: schema.symbols.filter((s) => layerOk(s.layer)),
    raws: schema.raws.filter((r) => layerOk(r.layer)),
    signature: layers === undefined || layers.includes("type_info") ? schema.signature : null,
  };
}

/** SSR 由来の camelCase 属性の小文字化を復元する (スタンドアロン SVG は XML —
 *  `viewbox` は無効)。テンプレート内の静的な camelCase (textPath / startOffset)
 *  は SFC コンパイラが保持するが、動的バインド (`:viewBox`) は小文字化される。
 *  新しい camelCase 属性を使うときはここに足す (XML validity テストが検出する)。 */
const CAMEL_ATTRS = ["viewBox", "gradientUnits", "preserveAspectRatio"];

/** renderToString の出力をスタンドアロン SVG (XML) に整える。
 *  - fragment コメント (`<!--[-->` 等) は Vue の hydration マーカー — 静止画に不要
 *  - 値なしの `data-v-*` (scoped style の印) は **XML として無効** なので必ず落とす
 *  - 空の `style=""` は SSR だけが出すノイズ (クライアントは属性自体を出さない)
 *  - 数値は小数2桁へ丸める (Rust レンダラの num() と同精度) — Vue 側で計算する
 *    エッジ端点・記号頂点の浮動小数ノイズ (59.99979…) がファイルに漏れない */
export function toStandaloneSvg(html: string): string {
  let svg = html
    .replace(/<!--[^>]*-->/g, "")
    .replace(/ data-v-[0-9a-f]+(="")?/g, "")
    .replace(/ style=""/g, "");
  for (const attr of CAMEL_ATTRS) {
    svg = svg.replaceAll(` ${attr.toLowerCase()}="`, ` ${attr}="`);
  }
  // テキストノードも対象になるが、表示テキスト (極ラベル・関数シグネチャ) に
  // 小数3桁以上は現れないため実質属性値のみに効く。
  svg = svg.replace(/-?\d+\.\d{3,}(?:e-?\d+)?/g, (token) => {
    const value = Number(token);
    return Number.isFinite(value) ? String(Math.round(value * 100) / 100) : token;
  });
  return svg;
}

/** 配置済み IR をスタンドアロン SVG 文字列にレンダする (SSR の本体)。 */
export async function renderSpellSvg(request: RenderRequest): Promise<string> {
  const app = (() => {
    if (request.belka !== undefined) {
      const belka = request.belka;
      return createSSRApp({ render: () => h(BelkaCircle, { belka }) });
    }
    if (request.ir === undefined) {
      throw new Error("リクエストに ir / belka のどちらもありません");
    }
    const schema = applyFilter(irToSchema(request.ir), request);
    const overlay = request.diff_overlay;
    return createSSRApp({ render: () => h(MagicCircle, { schema, overlay }) });
  })();
  // MagicCircle ツリーは palette / focus store を参照する — SSR では
  // 既定状態 (全レイヤー表示・選択なし) の Pinia を与える。
  app.use(createPinia());
  return toStandaloneSvg(await renderToString(app));
}

async function readStdin(): Promise<string> {
  const chunks: Buffer[] = [];
  for await (const chunk of process.stdin) {
    chunks.push(chunk as Buffer);
  }
  return Buffer.concat(chunks).toString("utf8");
}

// CLI エントリ (bun 実行時のみ)。vitest からの import では走らない。
if (import.meta.main) {
  readStdin()
    .then(async (input) => {
      const svg = await renderSpellSvg(JSON.parse(input) as RenderRequest);
      process.stdout.write(`${svg}\n`);
    })
    .catch((error: unknown) => {
      console.error(
        `magia-render: ${error instanceof Error ? (error.stack ?? error.message) : String(error)}`,
      );
      process.exit(1);
    });
}
