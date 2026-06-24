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

(着手時に追記)
