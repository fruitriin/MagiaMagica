# Phase 4.0.7 — SVG → Vue 境界スキーマ (案1: Rust SVG を Vue で動かす)

## 出典

- オーナー方針 (2026-06-11): Rust SVG を Vue コンポーネントの Props として受け、以降は Vue がリアクティブに操作する (案1)
- 親計画: Phase 4.0.5 (Vue 基盤) で立てた `v-html` での素朴な表示を、**境界スキーマ + コンポーネントツリー** に置き換える
- 後継: Phase 4.0.9 で SVG パーサを IR ビルダに差し替える (案2)。本計画はその伏線

## 目的

Rust が出力する SVG を「Vue が触れる構造化データ (`MagicCircleSchema`)」に変換し、その上に Vue コンポーネントツリー (`<MagicCircle>` `<Ring>` `<Operation>` 等) を構築する。
4.0.5 までの `v-html` 丸投げから、**Vue のリアクティビティが届く世界**に切替える。

**重要な伏線**: 本計画で作る `MagicCircleSchema` と Vue コンポーネント群は、Phase 4.0.9 (案2) でも **無修正で流用** する。本計画で書く「Schema を埋める入口 (SVG パーサ)」だけが 4.0.9 で捨てられる。

## スコープ

### やること

- **`MagicCircleSchema` 型定義** (Vue 側、TypeScript `type` 宣言):
  - `circles: Circle[]` (中央魔法陣 + 補助リング)
  - `operations: Operation[]` (各操作の配置済 x,y、効果カテゴリ、id)
  - `edges: Edge[]` (データフローエッジ、from/to id)
  - `glyphs: EffectGlyph[]` (効果記号、io / pure / async など)
  - `signature: Signature` (関数名、引数、戻り値、impl_context)
  - **全要素が「配置済 (x,y) 座標」を持つ** — レイアウトは Rust 任せ、Vue は描画専任 (POSD 分担)
- **SVG → Schema パーサ** (`web/src/converters/svgToSchema.ts`):
  - サーバから受け取った SVG 文字列を DOMParser で解析
  - `data-*` 属性 (Phase 1.6 で IR ノード id を SVG 要素に埋めた既存仕組み) を頼りに意味論を復元
  - 復元できない要素は `unknown` カテゴリに落として表示は維持
- **Vue コンポーネントツリー**:
  - `<MagicCircle :schema="schema">` — ルート
  - `<Ring :circle="c">` — 中央魔法陣 / 補助リング
  - `<Operation :op="op" :focus="isFocused(op.id)">` — 各操作 (リアクティブ強調)
  - `<EffectGlyph :glyph="g">` — 効果記号
  - `<EdgeLine :edge="e">` — データフローエッジ
- **Pinia store 接続**:
  - `useFocusStore` の `hoveredOperationId` / `selectedOperationId` を `<Operation>` の `:class` に反映
  - 操作クリックで store 更新 → 他コンポーネントに伝播
- **4.0.5 v-html の置換**:
  - `<MagicCircleView>` 内部を `v-html` から `<MagicCircle :schema="schema">` に切替
  - SSE 更新時は SVG 受信 → `svgToSchema` で Schema 再構築 → Vue がリアクティブ再描画

### やらないこと (別計画)

- IR JSON エクスポート + IR ビルダ (Phase 4.0.9)
- ピン中心ビュー (Phase 4.1)
- アニメーション・トランジション (本計画はリアクティブ操作までで、`<Transition>` は 4.1 で本格採用)

### POSD 観点で重要な「境界」

```
Rust (パース + 解析 + レイアウト)
  ↓ SVG 文字列 (現状の出力形式を維持)
SVG パーサ (本計画で書く、4.0.9 で廃止される)
  ↓ MagicCircleSchema (境界スキーマ ★)
Vue コンポーネントツリー (本計画で書く、4.0.9 でも流用)
  ↓ DOM (SVG 要素群)
```

**★ の境界 (`MagicCircleSchema`) を最初に決め切る** のが本計画の最大価値。スキーマが固まれば、入口 (SVG パース) も Vue 側ロジック (描画 + インタラクション) も独立に変えられる。

## 設計上の判断

### `data-*` 属性を頼りに SVG から意味を復元する

Phase 1.6 の SVG レンダラは IR ノード id を `data-node-id` 等で SVG 要素に埋めている (knowhow `svg-deterministic-rendering.md`)。
SVG パーサはこの属性を頼りに、意味論 (operation / glyph / edge) を復元する。
属性がない要素 (装飾要素、線など) は座標と種類だけ拾って Schema に入れる。

### Schema は「描画に必要な最小情報 + リアクティブにしたい情報」

Schema に何を入れて何を入れないか:
- 入れる: id, 種類 (operation/ring/edge/glyph), 配置済 (x,y), 半径・角度・色のヒント、効果カテゴリ
- 入れない: 描画スタイルの詳細 (フォントサイズなど。CSS/UnoCSS で寄せる)
- 入れる: フォーカス可能な要素 (operation) には `selectable: true` フラグ

過剰に詰めると 4.0.9 移行時にスキーマ変更が必要になる。**4.1 以降で必要な情報を意識して設計** する (ピン対象の id、距離計算の入力に必要な情報)。

### SVG パーサは捨てる前提で隔離する

`web/src/converters/svgToSchema.ts` 単独ファイルに閉じ込め、他のコードからは Schema 経由でしか触れないようにする。
4.0.9 着手時にこのファイルだけ削除すれば移行完了する状態を作る。

## 受け入れ基準

- [x] `MagicCircleSchema` 型が定義され、Vue コンポーネント群がこれを受けて描画する
- [x] 既存 fixtures (medium_render_doc / write_document / dense_dispatch) で 4.0.5 の `v-html` 表示と画素単位等価 (オーナー判定済み:「同じに見える」。実測: fuzz 5% で差 0.01% = AA ゆらぎのみ)
- [x] 操作クリック・ホバーで Pinia store の状態が変わり、対応する `<Operation>` の見た目に反映される (オーナー判定済み:「ホバーも選択もできてる」)
- [x] SSE ファイル更新で SVG 文字列が新規届くと Schema 再構築 → リアクティブ再描画される (Playwright の SSE テストで固定)
- [x] SVG パーサが `web/src/converters/svgToSchema.ts` 1ファイルに閉じている (参照は MagicCircleView の computed 1箇所)
- [x] `cargo test --workspace` / `vite+ test` (Vitest 28本) 通過 + Playwright 9本

## 後続候補

- Phase 4.0.9 で IR ビルダ実装、SVG パーサ削除
- Phase 4.1 で `MagicCircleSchema` の上にピン中心レイアウトとトランジションを乗せる

## 実装ステップ (粗粒度)

1. `MagicCircleSchema` 型を `web/src/types/magia.ts` に定義 (4.0.5 M3 で着手済みの場合は引き継ぎ)
2. SVG パーサ `web/src/converters/svgToSchema.ts` 実装 + Vitest ユニット
3. Vue コンポーネント `<MagicCircle>` `<Ring>` `<Operation>` `<EffectGlyph>` `<EdgeLine>` 実装
4. `<MagicCircleView>` の `v-html` を新コンポーネント呼び出しに置換
5. Pinia store 接続 (ホバー・選択状態のリアクティブ反映)
6. 既存 fixtures で画素単位等価のオーナー判定
7. Stage 1 品質ゲート + コーディング知見記録 (SVG パース、境界スキーマ設計、Vue 描画分担)
8. Stage 2 レビュー + 指摘対応
9. 完了処理

## 実装結果メモ (2026-06-11 完了)

- **計画書の前提訂正**: 「Phase 1.6 で IR ノード id を data-* 属性で埋めた」は実装と不一致 (SVG に data-* は無い)。意味論は class のみ (main-ring / aux-ring / op-dot / summon-glyph / edge-control-flow / signature / layer-*)。Rust 無変更で **class + 出現順 id** 方式に変更 (パーサは捨てる前提なので Rust にパーサ専用属性を足さない。SigilId 非公開の Phase 3.2 方針と整合)
- **スキーマの追加フィールド (計画外で必要になったもの)**: `z` (描画順 — 種類別に描くと重なりの z-order が変わり画素等価が崩れる)、`RawElement` (複合記号 sym-* とベルカ力場の素通し)、`Signature` の円弧/直線両対応 (ベルカは textPath でない)。`SchemaEdge.from/to` は `string | null` (4.0.9 で埋まる)
- **等価検証の実測**: 同版2回撮りノイズ 0 を確認した上で、3 fixtures の版差 0.3〜0.4% → fuzz 5% で 0.01% (Vue 生成 DOM と innerHTML パースの AA 差)。方法論は milestone-gated-ui-plan.md に knowhow 化
- ホバー = シアン / 選択 = 金の輪郭ハロー (Phase 3.2 の「記号色に触れない」原則)。SSE 更新時は選択をクリア (出現順 id の再採番でハローが別操作に移る誤挙動の防止)
- レビュー対応: viewBox 不正時の warn、EFFECT_BY_COLOR の3箇所同期警告コメント、drawList の key を id に、ほか計10件

## 想定リスク

- **スキーマ設計の過不足**: 4.1 以降で必要な情報を見落とすと Schema 拡張が連鎖。実装着手前に 4.1 計画書を読み返し、ピン対象・距離計算入力を Schema に確保
- **SVG パースの取り溢し**: 装飾要素や決定論レイアウトの細部 (微小回転など) が再現できない場合、まず Schema に座標を持って Vue 側で再構築する設計に統一 (Vue 側で描けない構造があれば Schema 拡張)
- **画素単位等価の困難**: SVG → Schema → SVG (Vue 再描画) で完全等価にならない可能性。受け入れは「視覚的に等価」レベルで判定し、ピクセル diff は緩く扱う
