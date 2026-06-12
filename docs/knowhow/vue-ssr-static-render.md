# Vue SSR + Bun compile による静止画レンダ (Vue 1本化)

> Phase 4.3 で確立。動的 UI と CLI 静止画出力 (`magia render` / `diff --svg`) が
> 同じ Vue コンポーネントツリーで描く構成。Bun 1.3.9 / Vue 3.5 / Vite+ 0.1.24 で確認。

## 発見した知見

### ビルドは「vite SSR ビルド → bun build --compile」の二段

- Bun は `.vue` を直接読めない。`vp build --ssr src/render/ssr.ts --outDir dist-ssr` で
  node 互換 JS に落としてから `bun build --compile dist-ssr/ssr.js --outfile <BIN>`
- 単一実行ファイルは ~59MB、**ウォーム起動 ~30ms** (初回は Gatekeeper 検証で ~800ms)。
  CLI のサブプロセスとして十分速い
- エントリは `if (import.meta.main)` で CLI 部を囲むと、vitest から `renderSpellSvg` を
  副作用なしで import できる (Bun 実行時のみ main が走る)

### renderToString の出力はスタンドアロン SVG (XML) としてそのままでは無効

SSR 出力には XML として無効/ノイズな断片が混ざる — 後処理 (`toStandaloneSvg`) が必須:

1. **動的バインドの camelCase 属性が小文字化される**: `:viewBox` → `viewbox` (XML 無効)。
   テンプレート内の**静的な** camelCase (`<textPath>`, `startOffset`) は SFC コンパイラが
   保持する — 落ちるのは動的バインドだけ。既知属性テーブル (`CAMEL_ATTRS`) で復元する
2. **値なしの `data-v-*` 属性** (scoped style の印) は **XML としてパースエラー**。必ず除去
3. fragment コメント `<!--[-->` (hydration マーカー) と空 `style=""` はノイズ
4. **Vue 側で計算する座標は浮動小数ノイズを含む** (エッジ端点 59.99979…)。属性値の
   数値を小数2桁へ丸める regex を最後に通す (Rust 側 IR の `nz()` と同精度規約)。
   テキストノードも対象になるが、表示テキストに小数3桁以上は現れない前提を
   コメントで明示する

### Pinia 依存コンポーネントの SSR は「既定状態の Pinia を挿す」だけ

`createSSRApp(...).use(createPinia())` — store を参照するコンポーネント
(レイヤー可視性等) は既定状態 (全表示・選択なし) で描かれる。静止画の意味論と一致

### Rust ⇔ SSR サブプロセスの規約

- パス解決は4段: ①環境変数 (明示) → ②exe と同 dir (配布) → ③exe の親 dir
  (開発: `target/debug/magia` と `target/magia-render`) → ④PATH。
  **build.rs に組み込む** (web/dist と同じ鮮度判定) とテスト・CI が常に最新を持つ
- stdin は**ブロックで明示 drop して EOF を送ってから** `wait_with_output()`。
  Rust の実装は内部でも閉じるが、意図を読み手に明示する
- 失敗時は Vue 側の stderr (スタック込み) をエラーに含める — SSR 内の例外が
  CLI のエラーメッセージとしてそのまま出る

### フィルタは「適用結果」を境界に渡す

`.magia` (FilterSpec) の解釈は Rust 側に残し、SSR へは畳んだ結果
(`show_layers: [...]` + `effects: [...]`) だけ渡す。Vue に DSL の知識を持ち込まない。
旧レンダラの「`<g>` ごと出さない」は SSR では「要素単位で filter」が等価
(テストの検証も g 単位 → 要素単位に書き換えが要る。**SSR 出力は1行**なので
`lines().any(...)` の検証は偽陽性になる — `split('<')` の要素単位で見る)

### 色定数を Vue に一本化したら旧参照コメントを一掃する

Rust 側の色定数 (palette.rs) を消すと「palette.rs と同期」コメントが各所で stale になる。
削除コミットと同時に `grep -rn "palette.rs"` で参照コメントを全部更新する
(レビューで5箇所指摘された — 削除系の変更は「定義の削除 + 参照コメントの掃除」をセットに)

## プロジェクトへの適用

- `web/src/render/ssr.ts` (エントリ + 正規化)、`crates/magia-cli/src/ssr.rs` (spawn)、
  `crates/magia-cli/build.rs` (自動ビルド)
- 色の正: `irToSchema.ts` COLOR_BY_EFFECT / `BelkaCircle.vue` POLE_STYLE・DOT_COLOR /
  `MagicCircle.vue` DIFF_STYLE / `uno.config.ts` theme (相互に手動同期)

## 注意点・制約

- `cargo install` した `magia` からは `magia-render` が見つからない (①で指定するか
  2バイナリ同梱の配布を待つ)。CLAUDE.repo.md に記載
- `<Transition>` 等のアニメは SSR では最終状態のみ (静止画の目的と整合)

## 参照

- docs/plans/phase4.3-composite-still-render.md (実装結果メモ)
- CLAUDE.repo.md「静止画レンダ magia-render」節
