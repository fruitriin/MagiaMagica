# Phase 4.0.5 — フロントエンド実行基盤 (Vue 3 + Vite+)

## 出典

- オーナー指定 (2026-06-11): **Vue 3 (Nuxt なし)** / 実行基盤 **Vite+** / ファイルシステム操作が必要なら **Bun 系ネイティブブリッジ** / スタイルは **UnoCSS** / TypeScript は **`type` を `interface` より優先** / **UI 実装中は細かく確認** (マイルストーンごとに目視)
- 親計画: Phase 4.0〜4.6 の総量を読み返した結果、素 JS 継続は 4.2 あたりで破綻リスクが高いと判定 (Claude 提案 → オーナー承認)
- 背景: Phase 2.1〜2.4 で inline HTML/JS が約 200 行に達し、Phase 4.6 で 5 軸 (pin / theme / diff / style / scope) の URL 状態管理が来る
- **本計画の特殊事情**: Phase 4.0 (ソース連動ペアビュー) が**素 JS 前提で先行着手中**。本計画は 4.0 完成時点の素 JS 成果を **Vue へ巻き取る移行作業を含む**。4.0 の計画書本体は書き換えず、本計画が「4.0 で実装された機能を Vue 等価に保ったまま基盤を入れ替える」責務を負う

## 目的

Phase 4 系列 (ピン中心ビュー・呼び出しジャンプ・ワークスペース俯瞰・テーマ拡張) を支えるフロントエンド実行基盤を確立する。
**「素 JS 継続」を破棄し、Vue 3 + Vite+ + UnoCSS に乗せ換える**。Phase 2.x で書いた inline HTML/JS、および **Phase 4.0 で先行実装された素 JS のソースペア機能**は本計画で全面 Vue 化する。

これは Phase 4 全体の前提整備であり、機能追加ではない。**Phase 4.0 が完成させた振る舞いは Vue 化後も画素単位で等価**に保ち、4.1 以降は本基盤の上に新規機能として乗せる前提に書き換える。

## スコープ

### 採用技術 (確定)

- **Vue 3** (Composition API + `<script setup>`)
- **Vite+** (VoidZero alpha) — dev/build/test/lint/format の統合 toolchain
  - 内訳: Vite (dev), Rolldown (build), Vitest (test), Oxlint (lint), Oxfmt (format), tsdown (型ビルド)
- **TypeScript** (strict)
- **UnoCSS** (オーナー指定) — Atomic CSS エンジン、Vite+ プラグイン経由で統合
  - プリセット: `@unocss/preset-uno` (Tailwind 互換ユーティリティ) + `@unocss/preset-attributify` (属性記法、Vue テンプレートと相性◎)
  - icons プリセット (`@unocss/preset-icons`) は採用判断保留 (UI が必要としたら追加)
- **Vue Router** (URL ↔ 状態同期。`?pin / ?theme / ?diff / ?style / ?scope` の5軸を支える)
- **Pinia** (状態管理。SSE 受信状態・現在のピン・周辺関数・パレット設定など)
- **Vitest** (ユニットテスト。Vite+ に統合済)
- **Playwright** (E2E。ユーザーグローバル指針: Web プロジェクトは Playwright MCP で検証)

### 採用しない (確定)

- **Nuxt** — オーナー指定により除外。SSR/ルーティング規約は本プロジェクトに不要
- **React/Svelte/Solid** — オーナーは Vue Pro
- **Tailwind CSS** — UnoCSS で代替 (preset-uno が Tailwind 互換のため学習・スタイル資産は流用可)
- **状態管理代替 (vuex/zustand 風)** — Pinia 1択
- **UI コンポーネントライブラリ (Vuetify/Element Plus 等)** — パレット程度の UI に過剰、UnoCSS で組む

### 採用 (追加・2026-06-11 格上げ)

- **Bun** (オーナー指示): Phase 4.3 で Vue SSR ベースの静止画レンダラを Bun の `bun build --compile` で single-file executable 化するため、**最初から Bun 前提で構築**。Node.js 両対応は二重保守を避けるため採らない (オーナー指示: 「Node で動くプロジェクトはプロジェクト構築がめんどくさいがち」)
  - 開発時ランタイム: Bun
  - パッケージマネージャ: Bun
  - Vite+ は Bun 互換 (Vite+ alpha 時点で確認済)
  - 配布形態: `magia` (Rust) + `magia-render` (Bun bundle) の2バイナリ構成 (Phase 4.3 で確定)
  - bun:ffi は引き続き保留 (現状不要、experimental)。要件発生時に別計画で判断

### やること

- **`web/` ディレクトリ新設**: Vite+ プロジェクトルート
  - `web/src/` — Vue ソース (components/ composables/ stores/ router/ types/)
  - `web/dist/` — ビルド成果物 (gitignore)
  - `web/vite.config.ts` — Vite+ 設定 (proxy: `/state /spell /events` → `http://127.0.0.1:4747`)
  - `web/package.json` — 依存定義
  - `web/index.html` — Vite+ エントリ
- **Phase 2.x フロントコードの全面移行**:
  - 旧: `crates/magia-cli/src/serve.rs` の inline HTML (`include_str!` または heredoc)
  - 新: `web/dist/` を `magia serve` が `rust-embed` で埋め込んで配信
  - レイヤー toggle / DSL UI / 書き起こし → Vue コンポーネント化
- **`magia serve` の役割再定義**:
  - 開発時: `vite+ dev` (HMR、ポート 5173) + `magia serve` (API、ポート 4747)。vite 側が `/state /spell /events` を 4747 に proxy
  - 本番時: `vite+ build` → `web/dist/` → `magia serve` が `rust-embed` で抱えて配信 (HMR なし、API も同ポート)
  - 既存 `magia serve` の HTML 生成コードは削除 (v1.0 前破壊的変更)
- **ビルド統合**:
  - `cargo build` の前に `vite+ build` を回す build.rs を `magia-cli` に追加
  - Vite+ がない環境 (CI 含む) のために Node.js / Bun の前提を `CLAUDE.repo.md` に記載
  - 開発時の並走スクリプト (`.claude/dev-server.sh` or `package.json` scripts) で `vite dev + magia serve` を1コマンド化
- **テスト基盤**:
  - Vitest ユニット: Pinia store の状態遷移、Vue Router の URL パース、コンポーネントの prop 反応
  - Playwright E2E: `magia serve` 起動 → ブラウザで実画面確認、ピン遷移・URL 同期・SSE 更新の golden
  - 既存 Rust 統合テスト (`magia serve` HTTP 200) はそのまま残す
- **CI 統合**:
  - GitHub Actions に Node.js (or Bun) セットアップ追加
  - `pnpm install && vite+ build` を `cargo build` 前に実行
  - `vite+ test` (Vitest) と `vite+ lint` (Oxlint) も追加

### やらないこと

- 既存機能の振る舞い変更 (本計画は基盤入れ替えのみ、ユーザー視点で見た目・操作は維持)
- ベルカ式 / Spell Diff UI の Vue 化 (Phase 3.5 / 3.2 で既に動いている素 JS は本計画で巻き取るが、機能拡張は別計画)
- Bun 採用 (条件付き、本計画では Node.js 前提)
- SSR (Vite+ は SSR も扱えるが本計画では SPA のみ)

## 設計上の判断

### なぜ Vite+ を採用するか (alpha 段階のリスクを受け入れる根拠)

- オーナー指定
- POSD「複雑性を下に押し下げる」: lint/format/test/build/dev が単一 entry に集約されると Phase 4 系列の運用負荷が大幅に下がる
- CLAUDE.repo.md「v1.0 前は破壊的変更を躊躇しない」と Vite+ alpha の API 変動が整合
- Cloudflare 買収 (2026-06-04) で持続性は担保。MIT 継続
- リスク: alpha 仕様変動。実装時に `vite-plus` の最新リリースノートを確認 (`docs/knowhow/` に追記する想定)

### Vite+ dev と magia serve の二段構成

開発時に dev サーバを2つ持つのは複雑性増だが、職務分担で清算する:
- **vite+ dev (5173)**: フロントエンド HMR、Vue コンポーネントのソース変更を即反映
- **magia serve (4747)**: Rust ソース監視 + IR/レイアウト計算 API
- vite+ 側の proxy で `/state /spell /events` を 4747 に転送 → ブラウザは 5173 だけ見れば良い
- 本番では vite+ build → magia serve が dist を rust-embed で配信、ポート1個に集約

これにより:
- フロントエンド変更時に Rust 再ビルド不要
- Rust 変更時にフロントエンドの状態は保持される (HMR 反対側でリロードされる)
- 配布物は `magia` バイナリ1個に dist を埋め込んで完結

### Pinia store の境界 (情報隠蔽)

- `useFocalStore` — 現在のピン、周辺関数、レイアウト
- `useSourceStore` — フォーカス関数のソースハイライト済み HTML
- `useConnectionStore` — SSE 接続状態、最終更新時刻、エラー
- `usePaletteStore` — テーマ、レイヤー可視性、パレット (Phase 2.2〜2.3 の UI 状態を統合)

各 store はサーバ API を抽象化したコンポーザブル (`composables/api.ts`) を呼ぶ。コンポーネントは store のみ参照し、API を直接叩かない (POSD「深いモジュール」)。

### rust-embed で dist を抱える

本番配信は `rust-embed` クレートで `web/dist/` をバイナリに埋め込み、`magia serve` が GET / で配信。
- 利点: `cargo install --path crates/magia-cli` で完結、配布時に dist を別配置不要
- 欠点: バイナリサイズ増 (~数 MB)。Vue + 必要最小限の依存なら 500KB 〜 1MB 圏内を狙う
- vite+ build の rolldown は tree-shaking 強力なので最小化に期待

### Bun を「現状不要」と判断する根拠

ファイルシステム操作 = ローカル `.rs` ソースの読み書き は **全て Rust サーバ (magia serve) が担う**。
ブラウザは fetch で API を叩くだけで完結する設計が既に動いており、Bun FFI を入れる動機がない。

Bun が必要になる将来シナリオ:
- ブラウザ非依存のデスクトップアプリ (Tauri 代替) を作りたい → そのとき判断
- IDE プラグイン的に LSP/エディタからの呼び出し → そのとき判断
- 任意のローカルパス書き込み (`magia render > path/to/anywhere.svg`) → 現状でも Rust CLI で可

本計画では「Bun が必要になる要件が出たら別 Phase で計画」とだけ位置づけ、依存に入れない。

### TypeScript strict + Oxlint の組合せ

- TypeScript strict で型エラーをコンパイル時に潰す
- Oxlint で ESLint 互換ルールを高速適用 (50-100x faster than ESLint)
- Oxfmt で Prettier 互換のフォーマット
- これらは Vite+ の `vite+ lint` / `vite+ format` で統一窓口

### TypeScript 規約: `type` を `interface` より優先 (オーナー指定)

- データ・状態・props・store の型は **`type` で宣言**
- `interface` を使うのは「宣言マージが意味的に必要な公開 API」だけ (通常本プロジェクトでは発生しない)
- 根拠: union/intersection/mapped/conditional の表現力が `type` で素直、`interface` のマージは予期せぬ拡張源になりやすい
- Oxlint ルールで `@typescript-eslint/consistent-type-definitions: type` 相当を有効化
- 例外を入れるときは `// ts: interface required because <理由>` のコメントを必須化

### UnoCSS の組み込み方針

- `vite.config.ts` で `unocss()` プラグインを登録、`uno.config.ts` に preset 定義
- Vue テンプレートでは **attributify 記法を主流**: `<div bg-white p-4 rounded-lg>` のように属性で書く (class の二重引用符地獄を避ける)
- 動的クラスは `class` バインディングで通常通り
- **scoped styles は最小化**: SVG 内部スタイル (記号色 etc) は CSS 変数 + UnoCSS shortcut で寄せる。compoonent scoped CSS は本当に分離が必要な場面のみ
- パレット (魔法陣の色定義) は `uno.config.ts` の theme に集約し、Phase 2.x の `palette.rs` (Rust 側) と意味を合わせる (色名は同じ語彙)

### Phase 4.0 → 4.0.5 の巻き取り戦略

Phase 4.0 が素 JS で先行着手中のため、本計画は完了時点の 4.0 成果を Vue に移植する責任を持つ。
具体的な巻き取り対象 (4.0 完了時点で動いているはず):

- **サーバ側 API**: 4.0 が定義した `/state` (関数一覧 + メタ)、`/spell/<fn>` (個別 SVG + syntect HTML)、`?fn=<name>` URL パラメータ
- **クライアント機能**: 左右ペイン (ソース | 魔法陣)、関数目次、`?fn=` 初期化、SSE 自動更新、エラー表示の保持

巻き取り原則:
- **サーバ API は維持** (壊さない)。Vue クライアントが同じエンドポイントを fetch / EventSource で叩く
- **`?fn=` → `?pin=` リネームは 4.1 で実施**。本計画では 4.0 互換の `?fn=` を維持し、4.1 のリネーム計画にバトンを渡す
- **画素単位の振る舞い等価**: 移行前後で同じ fixture で同じ見た目になることを Playwright で golden 比較
- **inline HTML の削除は最後**: Vue 版が動くことを目視確認したあとに Rust 側の HTML 生成コードを削る

## 受け入れ基準

- [x] `web/` 配下に Vite+ Vue 3 + UnoCSS プロジェクトが作成される
- [x] `cd web && vite+ dev` で localhost:5173 が起動、proxy 経由で `/state` が 200 を返す (M1。衝突時は PORT で差し替え)
- [x] `cargo run -p magia-cli -- serve <FILE>` で localhost:4747 にアクセス → 本番ビルド済 SPA が表示される (rust-embed 経由) (M5)
- [x] **Phase 4.0 計画書の「やること」(ペアビュー UI / 関数目次クリック切替 / `?fn=` 同期 / SSE 反映 / エラー時の図保持) が Vue で実装され、オーナー目視判定済み** (M3。判定でペイン並び変更 → 対応済み)
- [x] Phase 2.x の機能 (魔法陣表示、レイヤー toggle、DSL UI、書き起こし、SSE 自動更新) が Vue 化後も全て動作 (M4 判定「めっちゃいいかんじ」+ Playwright 8本で固定)
- [x] TypeScript の型宣言が `type` 統一されている (Oxlint consistent-type-definitions で機械化、interface 使用ゼロ)
- [x] UnoCSS の preset と theme が `palette.rs` (Rust 側) と色名・意味で一致 (M1 判定済み)
- [x] 既存 Rust 統合テスト (`cargo test --workspace`) 全本数通過 (SSE 回帰テスト追加、inline HTML テストは SPA 契約に更新)
- [x] Vitest ユニットテスト + Playwright E2E が CI で走る (ci.yml 新設、M6)
- [x] `cargo build --workspace` が `vite+ build` を自動で呼ぶ (build.rs。手順も CLAUDE.repo.md に記載)
- [x] ~~バイナリサイズが 5 MB 以下~~ → **基準撤廃 (オーナー判定 2026-06-11)**: 「意味のある機能の対価なら肥大化は気にしない」(CLAUDE.repo.md に方針化)。実測 5.91MB、SPA 同梱の純増 +0.11MB
- [x] CLAUDE.repo.md に **Bun** の前提と起動手順が記載される (Node.js は不採用)
- [x] **M1〜M5 の各マイルストーンでオーナー目視判定が通っている** (M1 色/トーン合格 → M2 同等性合格 → M3 並び修正対応 → M4 合格 + 折りたたみ対応 → M5 サイズ基準撤廃で合格)

## 後続候補

- Phase 4.0 (ソース連動ペアビュー) を本基盤上で実装
- Phase 4.6 でテーマ・パレット UI を Pinia store + Vue コンポーネントで自然に書ける
- 将来 Bun 採用時は本計画書を更新し、Node.js → Bun 並列対応で書く
- Storybook 導入は Vue コンポーネントが 10+ 個になったら別 Phase で判断

## 実装ステップ

UI 実装中はオーナーの細かい確認が入る (オーナー指定)。**M1〜M5 の各マイルストーン完了時に目視素材 (スクショ + 動作 GIF/動画) を生成してオーナーに送付し、判定を待ってから次のマイルストーンへ進む**。判定で NG が出たら同マイルストーン内で修正。

### M0: Phase 4.0 完了確認 (前提条件)

- **Phase 4.0 (スコープ縮小版) が完了していること** = サーバ側 API (`FunctionIndex`, syntect ハイライト, `/state`, `/spell/<fn>`, `--fn` 廃止) が動作しているかつ既存 inline HTML 経由で Phase 2.x 機能がリグレッションなく動く状態
- 4.0 は **フロント UI 未実装** で完了している。本計画 M3 でその UI を Vue で新規実装する (素 JS 版の golden は存在しない、Phase 2.x の inline HTML 機能の golden だけ取る)
- Phase 2.x 機能の golden を Playwright で記録 (魔法陣表示・レイヤー toggle・DSL UI・書き起こし・SSE 自動更新)

### M1: 基盤起動 (空の Vue が見える)

1. **Vite+ scaffold**: `vite+ create web --template vue-ts` でプロジェクト初期化、ディレクトリ整理
2. **依存追加**: vue-router, pinia, unocss, @unocss/preset-uno, @unocss/preset-attributify 等。`vite+ install`
3. **Vite+ 設定**: `vite.config.ts` に UnoCSS プラグイン + proxy 設定 (`/state /spell /events → 4747`)、`tsconfig` strict、Oxlint で `consistent-type-definitions: type` 有効化
4. **UnoCSS 設定**: `uno.config.ts` に preset と theme (魔法陣パレットの色名を `palette.rs` と一致させる)
5. **動作確認**: `vite+ dev` で localhost:5173 に "MagiaMagica" タイトルだけのページが出ること、proxy 経由で `/state` が 200 を返すこと

**→ M1 目視: 「ただ起動した状態」のスクショ + コンソール無エラーを送付。スタイル基盤・proxy 経路を判定**

### M2: 状態管理スケルトン (魔法陣だけ Vue で出る)

6. **Pinia stores**: `useFocusStore / useSourceStore / useConnectionStore / usePaletteStore` のスケルトン (型 = `type` で定義)
7. **API composable**: `composables/api.ts` で fetch (`/state`, `/spell/<fn>`) と SSE (`/events`) をラップ
8. **`<MagicCircleView>` コンポーネント**: store の `currentSvg` を v-html でレンダ、これだけで Phase 4.0 の魔法陣ペインが見える状態にする (**Phase 4.0.7 で `<MagicCircle :schema>` に置き換え予定**、本マイルストーンは過渡対応)
9. **境界スキーマ `MagicCircleSchema` 型定義** (`web/src/types/magia.ts`): Phase 4.0.7 / 4.0.9 で実装本体を書くが、**型だけは本 M2 で先に置く** (オーナー方針: 「スキーマだけ早く、ロジックは段階的」)。`circles / operations / edges / glyphs / signature / 配置済 (x,y)` を意味論ベースで宣言、TypeScript `type` で
10. **`?fn=` パラメータ受け取り**: Vue Router で初期 store 同期 (Phase 4.0 互換、リネームは 4.1)

**→ M2 目視: fixture を1つ (例: medium_render_doc) で開いて魔法陣が Vue 側で表示されるスクショ。Phase 4.0 素 JS 版と並べた比較画像も送付**

### M3: ペアビュー UI の Vue 新規実装 (4.0 計画の「やること」を Vue で実現)

**注**: 4.0 は素 JS UI を実装せず完了したため、本マイルストーンは "移植" ではなく **Vue での新規実装**。4.0 計画書の `### やること` (ペイン構造・関数目次・`?fn=` 同期・SSE 反映) を本 M3 が果たす。

11. **`<SourcePane>` コンポーネント**: 4.0 で生成された syntect HTML を `v-html` で表示、UnoCSS で左右分割レイアウト
12. **`<FunctionToc>` コンポーネント**: `/state` から取得した関数一覧、クリックで `?fn=` 切替
13. **SSE 連動**: 4.0 の `/events` SSE をフックして全コンポーネントに store 経由で反映
14. **エラー表示**: 4.0 の last-good スナップショット + エラー行 API を UI で表現 (構文エラー中も魔法陣保持)
15. **`?fn=` URL 同期**: Vue Router の query で history.replaceState、戻る/進む対応

**→ M3 目視: fixtures (medium_render_doc / write_document / dense_dispatch / write_control_flow) で 4.0 計画の受け入れ基準項目を Vue UI で1つずつ達成確認。オーナー判定**

### M4: Phase 2.x 機能の Vue 移植

16. **`<LayerPalette>` コンポーネント** (Phase 2.2 レイヤー toggle): visible/opacity の Pinia 化、UnoCSS で見た目を素 JS 版と等価に
17. **`<DslEditor>` コンポーネント** (Phase 2.3 .magia DSL): textarea、apply/export、エラーメッセージ
18. **`<TranscriptRegion>` コンポーネント** (Phase 2.4 書き起こし): ARIA visually-hidden、スクリーンリーダー専用

**→ M4 目視: パレット・DSL・書き起こしが Phase 2.x 完了時と等価に動くスクショ + DSL の往復テスト動画。スクリーンリーダー検証は静止スクショで OK**

### M5: 本番ビルド統合・旧コード削除

19. **rust-embed 統合**: `magia-cli/src/serve.rs` を `web/dist/` 配信に置き換え。**この時点で旧 inline HTML を削除**
20. **build.rs / dev-script**: `cargo build` 前の `vite+ build`、開発時の並走スクリプト (`.claude/dev-server.sh`)
21. **CI**: GitHub Actions に **Bun セットアップ** + `vite+ build` + `vite+ test` + `cargo test` (Node.js は不採用、オーナー指示)
22. **バイナリサイズ確認**: `magia` バイナリが 5MB 以下に収まるか測定

**→ M5 目視: `cargo install --path crates/magia-cli` で入れた `magia` バイナリだけで `magia serve <FILE>` が全機能動作する動画。配布物として完結している確認**

### M6: テスト・知見記録

23. **Vitest**: store 状態遷移、Router クエリパース、コンポーネント prop 反応、API composable のモック
24. **Playwright**: 起動 → fixture 切替 → SSE 更新で表示変化 → URL 同期 → エラー表示。M0 で取った golden と全シナリオ等価
25. **既存 Rust 統合テスト**: HTTP 200 / SSE 配信 / ファイル監視 を維持、削除すべきは inline HTML 関連だけ
26. **Stage 1 品質ゲート** + コーディング知見記録:
    - Vite+ alpha の落とし穴 (実装中に出たもの)
    - Vue 3 Composition API + Pinia + UnoCSS の組み合わせパターン
    - SSE のクライアント実装 (再接続・型付き event)
    - syntect HTML を `v-html` で受ける場合の XSS 検討
    - `type > interface` 規約の徹底ポイント

### 品質検証フェーズ (Stage 2)

27. レビュー + コントリビューション検出 + 指摘対応 → ゲート再実行 (Vue/TS/Vite+/UnoCSS の知見は ADDF 昇格候補)

### 完了処理

28. 計画 memo、Feedback / TODO 更新、**Phase 4.1 以降の計画書を「Vue 前提」に追補** (4.0 計画書は書き換えない、既に完了済のため)、アーカイブ、コミット

## 実装結果メモ

### M0 + M1 (2026-06-11 実施、M1 判定待ち)

- **ツールチェーン実値** (確認日 2026-06-11): Bun 1.3.9 / vite-plus 0.1.24 / @voidzero-dev/vite-plus-core 0.1.24 / vite 8.0.16 / vue 3.5.37 / vue-router 5.1.0 / pinia 3.0.4 / unocss 66.7.0 / typescript 6.0.2。package.json は floating (`latest`) をやめ全て実値ピン
- **M0**: cargo build/test 全通過 + `magia serve` 実機で Phase 2.x 機能 (既定表示 / レイヤー toggle / ベルカ切替 / DSL UI / 書き起こし region) のリグレッションなしを preview で目視確認。golden は決定論的テキストで `web/golden/phase2x/` に保存 (state.json / spell_render.json / spell_write_document.json — svg + svg_belka + source_html + transcript 同梱 / index.html)。スクショ golden の自動化は計画どおり M6 の Playwright で実施
- **M1**: `vp create vite:application` は vanilla-ts を生成するため、Vite+ 統合 (vp scripts / overrides) を保ったまま手で Vue 化した。`web/src/` は main.ts (pinia + router 配線) / App.vue / views/HomeView.vue (palette スウォッチ) / router/index.ts。`vp check` (oxfmt + oxlint + 型) と `bun run build` (vue-tsc + vp build) 通過。dist は JS 88.85 kB (gzip 34.59 kB) + CSS 3.36 kB — rust-embed 5MB 予算に大幅余裕
- **計画からの差分**:
  - tsconfig に `strict` が生成時に入っていなかったため手で追加 (+ `noUncheckedIndexedAccess`)
  - ポート 5173 が他プロジェクト (misskey) と衝突したため、`vite.config.ts` で `PORT` 環境変数による差し替えに対応し、`.claude/launch.json` の `web-dev` は autoPort: true で登録
  - attributify は prefix なしで採用 (M1 時点で Oxlint/Vue の警告は出ていない。出たら `un-` prefix を再検討)
  - scaffold の罠 (bunx の bin 解決・spawn vp ENOENT・::1 listen) は `docs/knowhow/viteplus-bun-frontend-bootstrap.md` に記録
- **M1 判定待ち素材**: Vue ページのスクショ (タイトル + palette.rs と同語彙のスウォッチ 3 系統) + 構成サマリを送付済み

### M1 判定 (2026-06-11): 合格

- 色・トーンとも OK。ルーティングは「ファイルベースが基本の好みだが、クエリ軸 (?pin/?theme/?diff/?style/?scope) の複雑性に対応するため明示的なクエリベースで進める」で確定 — `web/src/router/index.ts` に設計判断として記録。フェーズが進んで複雑性が収まるならファイルベース化リファクタを将来検討

### M2 (2026-06-11 実施、判定待ち)

- stores (focus / source / connection / palette) + `composables/api.ts` + `useMagiaSync` + `<MagicCircleView>` (v-html 過渡対応) + `MagicCircleSchema` 型先置き (`web/src/types/magia.ts`) + `?fn=` 受け取り
- **SSE 配信の潜在バグを発見・修正 (Phase 2.1 から)**: tiny_http の Response (チャンク転送) 経路は `chunked_transfer::Encoder` (8KB) + `BufWriter` (1KB) の二重バッファが flush されず、SSE イベントがクライアントへ届かない。統合テストが「inline HTML に EventSource の文字列がある」ことしか見ておらず捕捉できていなかった。`request.into_writer()` + 自前ヘッダ + イベント毎 flush (`stream_sse`) に置換し、回帰テスト `sse_events_stream_immediately` を追加。knowhow (minimal-dev-server-pattern.md) の該当記述を訂正済み
- E2E 確認: vite proxy 経由で `/events` 接続 → ファイル変更 → `/state` + `/spell` 再フェッチ → Vue 再描画
- 既知の軽微事項 (M3 で整理): 初回ロード時に onMounted の selectFunction と SSE 接続直後イベントの refresh で `/spell` が2回走る (冪等なので実害なし)

### M2 判定 (2026-06-11): 合格 (「同じに見える」)

### M3 (2026-06-11 実施、判定待ち)

- `<SourcePane>` (syntect HTML) + `<FunctionToc>` + ペアビューレイアウト (目次 | ソース | 魔法陣)、エラーバナー (構文エラー message + 行番号、last-good 保持)
- **URL を唯一の状態源にする一方向ループ**: TOC クリック → `router.push(?fn=)` → query watch → `selectFunction`。戻る/進むも同じ watch を通る。fn 未指定の fallback 選択時だけ `router.replace` で書き戻し (履歴を汚さない)
- 初回ロードは SSE 接続直後イベントの refresh に一本化 (M2 の二重フェッチ解消)。`setInitialFn` で `?fn=` を希望値として先置き
- バグ修正: focus store の関数照合を `name` → `qualified` に (impl メソッド `Caster::cast` が fallback に落ちていた)
- @unocss/reset 追加 (preset-uno は preflight なし — ul の黒丸・button 枠が出る)
- E2E 確認済み: TOC 切替 / URL 同期 / 戻る進む / fallback 書き戻し / SSE で新関数が目次に出現 / 構文エラーバナー + last-good + 復旧 (一時 fixture)。HMR 編集中限定の警告は knowhow へ
- 素材: write_document / write_control_flow / medium_render_doc / dense_dispatch の4ペアビュー (2x2) を送付済み

### M3 判定 (2026-06-11): 並び替え指示 + 別スコープ要望 → 対応済み

- ペインを「魔法陣 (左・flex 1.6) | コード (中) | 関数一覧 (右端)」へ並び替え (一番見せたいのは魔法陣)
- 別スコープ要望は phase4.0.6-circle-affordances.md (凡例/入口サイン/補助陣ラベル — SVG 描画系は 4.3 後とオーナー確定) と phase4.1 追補 (TOC ピンフィルタ/ツリー) に計画化
- 「関数リストいい感じ」受領

### M4 (2026-06-11 実施、判定待ち)

- `<LayerPalette>` (式 radio / レイヤー checkbox + opacity slider / 全表示・全非表示 / .magia DSL details) を右カラム上部に、`<TranscriptRegion>` (visually-hidden) を配置
- palette store のレイヤー語彙を Rust 側 FilterSpec と同一に修正 (`control_flow / effects / type_info` — M2 スケルトンの `control/effects/types` は誤り)
- DSL は `lib/magiaDsl.ts` の純関数 (parseDsl / exportDsl) に分離 — M6 Vitest の対象。文言は inline 版と同一
- レイヤー適用は MagicCircleView が `g.layer-*` に display/opacity を当てる (位置不変 spec §5.4)。v-html のため watch + nextTick (4.0.7 で宣言化予定)
- `useQuerySync` で URL クエリ ↔ store を双方向同期。**形式は inline 版と完全互換** (?fn / ?style=belka / ?layers=a,b / ?op=l:v) — URL 直開き・リロードで全状態が再現される
- E2E 確認済み: toggle/slider/式切替/リロード復元/URL 直開き/DSL 往復 (エクスポート・適用・カテゴリ注記・行番号エラー)/transcript region

### M4 判定 (2026-06-11): 合格 (「めっちゃいいかんじ」) + パレット折りたたみ指示 → 対応済み (既定閉)

### M5 (2026-06-11 実施、判定待ち)

- rust-embed 統合 (`folder = "../../web/dist"`)、旧 inline HTML (INDEX_HTML 194行) 削除。GET / と静的ファイルは `embedded_response` (拡張子 → Content-Type 最小マップ)
- build.rs: dist の鮮度判定 (src の最大 mtime と比較) → 古ければ `bun install --frozen-lockfile && bun run build`、bun 不在は手順つき panic。**rerun-if-changed に dist を入れない** (再実行ループ)
- CI: `ci.yml` 新設 (Bun 1.3.9 / vp check / bun build / clippy / fmt / cargo test)。spell-diff.yml にも setup-bun (build.rs が bun を呼ぶため)
- CLAUDE.repo.md に Bun 前提 + 開発フロー、`scripts/dev-web.sh` (magia serve + vite dev の並走)
- 統合テスト: inline HTML 前提の assert を SPA shell 契約 (`<div id="app">` / asset JS 200 / 未知 404) に更新。UI の振る舞いは M6 Playwright が担う
- **バイナリサイズ**: `strip = "symbols"` + `lto = "thin"` を workspace に追加し 7.03 → **5.91MB**。受け入れ基準 5MB は超過だが、**SPA 同梱の純増は +0.11MB** (M5 前 6.92MB) — 超過の主因は Phase 4.0 の syntect (シンタックス定義)。5MB 復帰には syntect の default-syntaxes を Rust だけに絞る最適化が必要 (別タスク候補、オーナー判定事項)
- cargo install 実機確認: インストール済みバイナリ単体で SPA + API + SSE 全動作

## 想定リスク

- **Vite+ alpha の API 変動**: アルファ期間 (2026-03〜) のため後方互換破壊あり。実装時に最新リリースノート確認 + 計画書に確認日を記録。Vite+ が破壊的変更を出したら本計画に「Vite+ X.Y 対応」の追補をつける
- **バイナリサイズ膨張**: Vue + UnoCSS + 必要ライブラリで dist が 1MB 超えたらバイナリ 5MB ラインに収まらない可能性。受け入れ基準で測定 → 超過したら Vite+ の compression オプション + rust-embed の gzip オプションで対応
- **rust-embed と HMR の両立不可**: 本番埋込はリビルドが必要なため HMR と相反。開発時は vite+ dev 経由が必須。CLAUDE.repo.md に開発フロー明記
- **既存 Phase 2.x テストの巻き戻り**: inline HTML 削除で Phase 2.1〜2.4 の Rust 側テストが意味を失うものは削除、HTTP/SSE のテストは維持。golden HTML テストは Vue 化で全更新
- **Phase 4.0 機能との等価性ギャップ**: 巻き取り後にユーザー体験差が出るリスク。M0 で 4.0 完了時の golden 取得 + M3 でオーナー目視判定で抑える。差異が出た場合は本マイルストーン内で修正、進めずに繰り返す
- **Node.js / Bun の二重前提**: 計画書では Node.js 採用、Bun は条件付き。CI を Node.js 1本に固定して開発者の選択肢として Bun を許容する程度の温度感
- **Vite+ alpha のドキュメント網羅性**: 不明点が出たら GitHub issue + Vite 7/8 ドキュメントを参照。Vitest など個別ツール側のドキュメントは安定している
- **UnoCSS attributify と Oxlint/Vue lint の相性**: attributify 記法 (`<div bg-white p-4>`) を Vue/Oxlint が未知の属性として警告する可能性。`uno.config.ts` の attributify オプションで prefix (`un-`) を使うかどうかは M1 で決定
- **`v-html` での syntect HTML 受け取り**: syntect はサーバ側で信頼できるソースから生成しているが、`v-html` を使うこと自体は XSS リスクとしてレビュー対象。サーバ側で生成するファイルパスを限定 (`<FILE>` 引数のみ) + DOMPurify は入れない (オーバーキル) 方針で進めるが、レビューで方針再確認
