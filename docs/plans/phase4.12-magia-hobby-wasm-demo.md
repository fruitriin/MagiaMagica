# Phase 4.12 — magia-hobby: WASM クライアント完結のデモ版 (SNS 拡散用)

## 出典

- オーナー発案 (2026-06-24): 「Magia をどこかでホスティングして、GitHub の URL を渡すと
  パブリックリポジトリから魔法陣を作れたら、一気に遊びとして使ってもらえそう。
  フル機能でなくてもよさそう」
- そこからの設計対話 (同日) で以下に収束:
  - 実装方式は **クライアント完結の WASM**（サーバ clone / functions 経由ではなく）
  - **GitHub Pages / Vercel / Cloudflare Pages 等の静的ホスティング**にデプロイ
  - 第一弾の機能は **ソースコードのペースト + ローカルファイル読み取り**に絞る
  - `magia-hobby` crate は **magia-core / magia-rust に依存する薄いラッパー**に保つ
  - positioning は **「デモ版でぱっと使わせて SNS で Share」→ フル機能はチェックアウト**

## 目的

`magia serve` はローカル前提（`notify` でファイル監視・CWD 走査・`git` シェルアウト・
SSE ライブリロード・rust-embed 同梱バイナリ）で組まれている。**インストール障壁が
あり、初見の人に「ぱっと触らせる」ことができない**。

本計画では、Magia のパース→IR→レイアウト→描画パイプラインを **WASM に切り出して
ブラウザ単体で完結**させ、静的ホスティングに置く **デモ版 (`magia-hobby`)** を作る。
URL を踏めば即・自分のコードが魔法陣になり、リンクで共有できる。フル機能 (ライブ追従・
diff・俯瞰・呼び出しジャンプ・private のローカル丸ごと閲覧) は引き続き
`cargo install` した `magia serve` の領分とし、**デモはその入口 (funnel)** に位置づける。

## 設計が成立する根拠 (コードベース調査済み, 2026-06-24)

1. **パース層がすでにソース文字列入口** — ファイルパスでなく `&str` を受ける公開関数が
   揃っている。WASM ラッパーは変換を噛ませる必要がない:
   - `magia_rust::list_functions(source: &str) -> Result<Vec<String>, Error>`
   - `magia_rust::parse_function(source: &str, fn_name: &str) -> Result<MagiaGraph, Error>`
   - `magia_core::layout::layout(...)` → `magia_core::render::ir_export::spell_ir(graph, layout) -> SpellIr`
2. **magia-core / magia-rust にネイティブ依存が無い** — `std::process` / `Command` /
   `notify` / `SystemTime` / `Instant::now` / `getrandom` を grep した結果、これらは
   **すべて magia-cli 側に隔離**されている (gitio.rs / serve.rs)。`wasm32-unknown-unknown`
   でビルドが通る見込みが高い (M1 で導通スパイク確認)。
3. **SPA は元々 JSON を消費** — Vue 側は `/spell/...` 等から JSON を受け取って描画する。
   よって WASM 境界も **JSON in / JSON out** にすれば、既存の Vue パーサ・描画
   コンポーネントを無修正で流用でき、`fetch()` を WASM 呼び出しに差し替えるだけで済む。

## スコープ ((a) 案 — 薄さ優先)

対話で確定した「(a) で出して、必要になったら (b)」を採用する。

**第一弾に含める:**
- 入力プロバイダ: **ソースコードのペースト** / **ローカルファイル読み取り** (`.rs` 1枚を
  D&D または `<input type="file">`、`FileReader` で文字列化)
- `list`（ファイル内の関数一覧）→ 関数選択 → `spell`（素の魔法陣 IR）+ 凡例
- `#code=<deflate+base64>` による URL 状態共有 (Playground 方式)
- prefill 例（初回 paint で空フォームでなく魔法陣を出す。自己ホスト例を推奨）
- 固定 OG カード1枚

**第一弾に含めない (= チェックアウトへ誘導):**
- ライブ追従 (watch / SSE)、Spell Diff、ワークスペース俯瞰、呼び出しジャンプ、
  ピン中心ビュー (neighbors) / 周辺チップ抜粋 (excerpts)、private リポジトリの
  ローカル丸ごと閲覧、リモートリポジトリ URL 供給
- これらは `serve.rs` の組み立てロジック (`render_spell` / `spell_json` / `state_json` /
  `excerpt_maps` / `workspace_json`、計200行超) や `git` に依存し、WASM 単体に乗らない。
  制約は **「インストール不要・コードが外に出ない・一瞬」**と裏返して訴求する。

### 「薄いラッパー」を保つ罠 (重要)

リッチ機能 (neighbors / excerpts / 俯瞰) の組み立てロジックは現状 **magia-cli の
serve.rs に住んでいる** (magia-core ではない)。第一弾でこれを hobby にコピーすると、
Feedback.md 記載の **「EFFECT_BY_COLOR 3箇所手動同期」と同型の drift** を生む。

- **(a) 第一弾**: 機能を `list` + 素の `spell` に絞り、serve.rs の組み立ては再利用しない
  → `magia-hobby` は誇張なく数十行で済む
- **(b) 後続でリッチ機能が欲しくなったら**: serve.rs の組み立てを **共有レイヤーに抽出**
  (`magia-core` に `view` / `assembly` モジュール) してから、**serve も hobby も両方
  その上の薄いラッパーにする**。組み立てを2つ持たない。

## マイルストーン

### M1 — `magia-hobby` crate + WASM 導通スパイク

- `crates/magia-hobby` を workspace に追加 (`crate-type = ["cdylib"]`、`wasm-bindgen` 依存)
- 薄ラッパー (理想形、これ以上太らせない):
  ```rust
  #[wasm_bindgen]
  pub fn list(source: &str) -> Result<String, JsError> {
      Ok(serde_json::to_string(&list_functions(source)?)?)
  }
  #[wasm_bindgen]
  pub fn spell(source: &str, fn_name: &str) -> Result<String, JsError> {
      let graph = parse_function(source, fn_name)?;
      let placed = layout(&graph);
      Ok(serde_json::to_string(&spell_ir(&graph, &placed))?)
  }
  ```
- **導通スパイク**: `wasm32-unknown-unknown` ターゲットで magia-core / magia-rust が
  ビルドできることを最優先確認。落ちる場合の代表原因と対処:
  - `proc-macro2` は fallback モードで使う (`proc-macro` feature を有効化しない)
  - 乱数が要る箇所があれば `getrandom` の `js` feature
  - 時刻依存があれば排除 (現状 grep ヒットなし)
- 出力 JSON が serve の `/spell/` (素の spell、neighbors/diff なし) と等価であることを
  テストで固定 (既存 Rust の spell_ir 経路と突き合わせ)

### M2 — hobby 用 Vite 別ビルドターゲット + Vue 描画流用

- `web/` に **hobby 専用エントリ**を追加 (serve 同梱の rust-embed SPA とは別ターゲット。
  静的1枚を吐く)。`magia-render` の SSR は不要、**動的描画の Vue コンポーネントだけ流用**
- **data provider を抽象化**: `HttpDataSource` (dev/serve、既存) / `WasmDataSource`
  (hobby、`magia-hobby` の wasm を呼ぶ) のインターフェースに分離
- 入力 UI: ペースト textarea + ファイル D&D / `<input type="file">`。得た文字列を
  `WasmDataSource.list()` → 関数ピッカー → `WasmDataSource.spell()` → `<MagicCircle>`
- 凡例パネル (4.0.6 前半) を流用

### M3 — 共有 + 初速 + デプロイ

- **`#code=` URL 状態共有**: 入力ソース (+ 選択関数) を deflate+base64 で fragment に
  載せ、ロード時に復元。長大ファイルは URL 長で諦める (デモらしい割り切り)
- **prefill 例**: 初回 paint で魔法陣が出るよう、自己ホストの関数等を初期投入
- **固定 OG カード**: 「コードを魔法陣に」汎用カード1枚 (静的 meta)
- **デプロイ**: **Vercel または Cloudflare Pages を推奨** (GitHub Pages でなく)。
  理由はサブパス問題の回避に加え、後続の OG 用 function (下記) を同じ土俵に足せること。
  GitHub Pages でも (a) の範囲は動くが、Vite の `base` 設定が要る

### 後続 (本計画外、別 Phase 候補)

- **OG プレビューカードの動的化**: 断片ごとに魔法陣プレビューを出したい場合、SSR で
  魔法陣を PNG 化する **function を1個**足す (`magia-render` の SSR を PNG 化に流用)。
  OG クローラは JS も `#fragment` も読まないため、per-snippet カードにはこの関数が要る。
  **functions が再登場する唯一の場所** — デモ本体は静的のまま、関数は OG 専用に分離
- **リモート供給プロバイダ**: 単一 raw URL / gist (fetch 1発) → リポジトリ URL
  (ツリー走査)。リポジトリ巡回はレート制限 (無認証 60req/h) が効くため、トークン
  (5000req/h) と commit SHA キャッシュを持てる **WASM-in-Worker** が要る。ここで初めて
  サーバ的要素が正当化される
- **ローカルフォルダ供給**: `webkitdirectory` / File System Access API で clone 済み
  リポジトリを丸ごと読む。private を外に出さずローカル閲覧できる (俯瞰が要るので (b)
  の共有レイヤー抽出が前提)

## positioning (funnel)

- **デモ (本計画)**: 摩擦ゼロ・インストール不要・コードがブラウザから出ない・一瞬で
  魔法陣 + リンク共有。SNS で「自分のコードが魔法陣になった」を撒く入口
- **フル機能 (`magia serve`)**: ライブ追従・diff・俯瞰・呼び出しジャンプ・private の
  ローカル丸ごと。`cargo install` した人の領分
- デモの「できない」は劣化でなく **「ちゃんと使うなら入れてね」導線**として明示する

## 依存関係

- **4.0.5 → 4.0.7 → 4.0.9 (Vue 一本化) が前提** — Vue 描画コンポーネントと
  `MagicCircleSchema` / `SpellIr` 契約をそのまま流用する
- **magia-core / magia-rust の wasm クリーンさが前提** (M1 で確証する)
- (b) の共有レイヤー抽出を将来やる場合、serve.rs の組み立てを magia-core に移すため
  serve 側も同時に薄くする (両方を薄いラッパーにする、が抽出の条件)

## 実装結果メモ

### M1 完了 (2026-06-26)

**導通スパイクは全て合格**:
- `magia-core` / `magia-rust` は `wasm32-unknown-unknown` で**無修正ビルド通過**
  (syn / petgraph / kurbo / serde 含む)。ネイティブ依存ゼロの事前調査どおり、対処は
  一切不要だった (proc-macro2 fallback も getrandom も触らずに済んだ)。
- `magia-hobby` crate を追加 (`cdylib` + `rlib`)。`wasm-bindgen` は
  `[target.'cfg(target_arch = "wasm32")'.dependencies]` に置き、**native の cargo test は
  wasm-bindgen を一切引かない**。pure 関数 `list_json` / `spell_json` を native でテスト
  (4本パス)、`#[wasm_bindgen]` 薄ラッパーは wasm32 限定 `mod wasm` に隔離。
- wasm32 リリースビルド + `wasm-bindgen --target web` で JS グルー生成まで通し、
  `list(source)` / `spell(source, fn_name)` のエクスポートと `.d.ts` を確認。
  生成 wasm は約 1.9M (wasm-opt 未適用。M3 で `-Oz` 余地あり)。
- `scripts/build-hobby-wasm.sh` で wasm ビルド → `web/src/hobby/wasm/` へ配置を再現可能に。
  生成物は `.gitignore` 済み (ソースから再生成)。

**スコープ判断**: 1サイクルでは M1 (crate + 導通 + テスト) に集中。リッチ機能組み立ての
再利用は意図的に避け、`spell_json` は `{ qualified, signature, ir, transcript, belka_ir,
start_line }` に絞った (serve `/spell/` 契約のサブセット)。`source_html` (syntect) は
magia-cli 依存なので除外 — デモではハイライトなし、または将来クライアント側で。

**次サイクル (M2)**: web に hobby 専用 Vite エントリ + `WasmDataSource` (生成 wasm を
`import` して `list`/`spell` を呼ぶ) + ペースト/ファイル UI。Vue の `<MagicCircle>` /
凡例 / transcript パネルを流用。`spell_json` がリッチ拡張 (neighbors/excerpts) を欠く点は、
欲しくなったら (b) 案 (serve.rs の組み立てを magia-core へ抽出) で対応。

#### M1 コードレビュー指摘 → M2 で対応 (Critical なし)

- **[Warning] `SpellResponse` 型との部分非互換**: `web/.../magia.ts` の `SpellResponse` は
  `source_html` / `call_excerpts` / `op_excerpts` / `ring_excerpts` を**必須**で持つが、
  `spell_json` はこれらを返さない。Vue の excerpt 系参照は `?.` で安全だが、
  `CallInspector.vue` の `spell.source_html` は必須参照。M2 で `WasmDataSource.spell()` を
  `SpellResponse` に流す時点で型エラーになる。**M2 着手時に二択を決める**:
  1. `HobbySpellResponse` を別型で定義しサブセットを明示 (drift を型で検出)
  2. `SpellResponse` の該当4フィールドを optional 化し serve 側も `?.` に統一
  → 推奨は 1 (serve の「常に返す」前提を型から消さない)。`source_html` は hobby では
     ハイライト無し方針なので、`CallInspector` の表示を optional 化する必要がある。
- **[Suggestion] `spell_json` の二重パース**: `function_index` で1回、`parse_function` 内部で
  もう1回 `syn::parse_str` する。WASM は JIT 無しで大ファイルのパースコストが出るため、
  M2 でユーザー体験を見て、必要なら1走査で entry+graph を返す経路を検討。
- **[Suggestion] `list_json` に `args` が無い**: パレットの引数表示 (`?args=name,type`) を
  hobby でも出すなら、`FunctionEntry.args` をレスポンスに追加する。M2 で機能可否を決定。

### M2 完了 (2026-06-26)

**別ビルドターゲットで serve と完全分離**:
- `web/src/hobby/` 一式 (HobbyApp.vue / main.ts / dataSource.ts / examples.ts) +
  `web/hobby.html` + `web/vite.config.hobby.ts` + `build:hobby` スクリプト。
  serve 同梱 SPA (`index.html` → `dist/`) はこの設定を読まないため、**serve バイナリに
  wasm は混入しない** (build 後 `dist/assets` に wasm 無しを確認)。出力は `dist-hobby/`
  (base `./` で任意ホスト配下に置ける)。
- データ供給は `makeWasmDataSource(wasm)` (純粋・フェイク注入でテスト可能) と
  `loadWasmDataSource()` (動的 import で生成 wasm を初期化) に分離。serve の `api.ts`
  (fetch) と対称な「WasmDataSource」。JSON 契約は serve のサブセットそのまま。
- 描画は既存 `circle/*` (MagicCircle / BelkaCircle / SymbolLegend) と palette store を
  **無修正流用**。focus store / router / SSE は持たない (デモは単一画面)。
- **M1 レビュー Warning 対応**: `HobbySpellResponse = Pick<SpellResponse, ...>` を追加。
  サブセットを型で明示し、本体 `SpellResponse` が変われば型エラーで drift を検出する。
- **生成物不在でも main ビルドが通る**: `web/src/hobby/wasm-glue.d.ts` の ambient
  ワイルドカード宣言で、wasm 未生成 (fresh checkout / CI) でも `vue-tsc` が通過。実体が
  あれば実体の .d.ts が優先される (両立を実測確認)。生成 wasm は `.gitignore`。

**検証**: vp check (55ファイル クリーン) / vp test (48 passed、dataSource 3本含む) /
main `vue-tsc` は wasm 有無どちらも通過 / bun での wasm 実行 smoke / Playwright 実機描画
(prefill `summarize_errors` が魔法陣に、コンソールエラーなし)。生成 wasm 1.95MB (gzip 482KB)、
アプリ JS 100KB (gzip 38KB)。

**次サイクル (M3)**: `#code=` deflate+base64 共有リンク + OG カードの動的化 (SSR PNG
function) + Vercel/CF Pages へデプロイ。`build:hobby` の成果物 `dist-hobby/` をそのまま上げる。
deploy 時の root ファイル名 (`hobby.html` → `index.html` リライト) は M3 で対応。

### M3 (共有リンク + デプロイ手順) 完了 (2026-06-27)

オーナー指示「デプロイする部分は作業手順を成果にする」に従い、**実装する部分 (共有リンク)**
と**手順書にする部分 (デプロイ)** に分けた (実デプロイは認証が要る外向き操作なので runbook 化)。

- **`#code=` 共有リンク**: `web/src/hobby/share.ts` (純粋・テスト可能)。base64url(utf8) で
  `encodeShare`/`decodeShare`、`buildShareUrl`/`parseShareHash`。圧縮 (deflate) は将来
  `CompressionStream` に**この関数だけ差し替え**で対応できる (短い関数なら base64 で URL 長に
  十分収まるため、まずは依存なし・同期・全ブラウザの素の base64url。(a) 案の精神)。
  HobbyApp は onMounted で `#code=` を prefill より優先して復元、「共有リンクをコピー」ボタンを
  追加 (`history.replaceState` で URL バー反映 + clipboard、非対応/失敗時は案内)。
- **ゼロ設定デプロイ**: `build:hobby` の末尾で `dist-hobby/hobby.html` → `index.html` に
  リネーム (`base: "./"` なので同ディレクトリ内リネームで参照は保たれる)。各ホストがルートで
  そのまま配信できる。
- **デプロイ手順書**: `docs/deploy-hobby.md` (Vercel / Cloudflare Pages / 事前ビルド方式、
  MIME/圧縮/キャッシュ/COOP不要/OG の注意)。OG 動的化 (SSR PNG function) は「functions が
  再登場する唯一の場所」として手順書と計画に残置 (本サイクルでは実装せず)。

**検証**: vp check (57ファイル クリーン) / vp test (55 passed、share 6本含む) / Playwright で
`#code=...&fn=...` URL を踏むとソース復元 + 関数選択 + 描画 (エラーなし) / build:hobby が
index.html を出力。M3 レビュー Warning (W1 hashchange→replaceState / W2 base の `#` 防御 /
W3 async ハンドラ明示) と Suggestion (atob パディング・JSDoc・手順書注記) を同サイクルで対応。

**残置 (後続・別 Phase 候補)**: OG プレビューカードの動的化 (SSR PNG function)、リモート
供給プロバイダ (raw URL / リポジトリ URL = WASM-in-Worker)、ローカルフォルダ供給。
**Phase 4.12 のデモ系譜 (M1〜M3) はこれで一区切り** — 実デプロイはオーナーが手順書に沿って実施。

### PR #3 レビュー対応 (2026-07-03)

8観点マルチエージェントレビュー (検証通過 8件) を全て修正:

- **共有リンクの正確性**: `decodeShare` を `TextDecoder("utf-8", { fatal: true })` に —
  SNS で途中切断されたリンクが U+FFFD 文字化けで「成功」せず、prefill フォールバックに
  正しく倒れる。共有 URL に `style=belka` を追加 (非既定時のみ) — ベルカ式の見た目ごと
  共有できる。共有時に前回のエラー表示をクリア
- **UI 挙動**: file input の値リセット (同一ファイル再選択で change が発火)。構文エラー時は
  last-good の陣・関数一覧を保持 (serve の spec §7 と同じ規約 — 陣が消えない)
- **wasm-bindgen init**: 非推奨の裸 URL 位置引数 → `{ module_or_path }` 形式。ambient 宣言
  (`wasm-glue.d.ts`) も現行形式だけを型に残し回帰を防止
- **serve/hobby 契約の一点化 (根本修正)**: `FunctionEntry::summary_json` (magia-rust) と
  `SpellResponseBase` (magia-core ir_export) を新設し、serve `/state` `/spell/` と hobby の
  `list_json` / `spell_json` が同一実装・同一 struct から直列化 — M1 受け入れ基準
  「serve と等価」を等価性テストではなく**型共有で構造的に保証**する形に昇格
- **二重パース解消**: `parse_function_with_entry` (magia-rust) で index + IR を1パースに
  (「同じソースを2回パースする公開 API を増やさない」Phase 4.2 規約に整合)。さらに
  dataSource (web) が同一ソースの list/spell をメモ化 — 関数ドロップダウン往復が O(1) に
- **手順書**: Cloudflare Pages の `npx` → `bunx` (Bun 統一規約)
