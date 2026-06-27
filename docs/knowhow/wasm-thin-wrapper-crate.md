# WASM 薄ラッパー crate のパターン (クライアント完結デモ版)

Phase 4.12 (`magia-hobby`) で、既存の Rust パイプラインをブラウザ単体で動かす
WASM デモ版を作ったときの定型。

## 発見した知見

- **ネイティブ依存を1つの crate に隔離しておくと wasm 化が無修正で通る**。
  `magia-core` / `magia-rust` は `wasm32-unknown-unknown` で**一切の修正なしにビルド成功**
  (syn / petgraph / kurbo / serde / serde_json / thiserror すべて wasm 対応)。
  `std::process` / `Command` / `notify` / `tiny_http` / `SystemTime` / `getrandom` といった
  ネイティブ依存は `magia-cli` 側 (gitio.rs / serve.rs) に閉じ込めてあったのが効いた。
  事前に `grep` で core/rust 側にこれらが無いことを確認 → スパイク一発で通った。

- **wasm-bindgen はターゲット限定依存にして native テストから締め出す**:
  ```toml
  [lib]
  crate-type = ["cdylib", "rlib"]   # cdylib=wasm本体, rlib=native単体テスト

  [target.'cfg(target_arch = "wasm32")'.dependencies]
  wasm-bindgen = "0.2"
  ```
  ロジックは `pub fn list_json(&str) -> Result<String, String>` のような **pure 関数**として
  書き、`#[wasm_bindgen]` エクスポートは `#[cfg(target_arch = "wasm32")] mod wasm` に隔離して
  `JsError` へ変換するだけにする。こうすると:
  - native の `cargo test` は wasm-bindgen を**一切引かない** (ビルドが軽い・依存が綺麗)
  - ロジックは native でテストできる (wasm ランタイム不要)
  - wasm 境界はターゲット固有として閉じ込まる (POSD: ターゲット固有はそのターゲットに)

- **JSON in / JSON out の境界**: `&str` を受け `String` (JSON) を返す。既存の HTTP API
  (serve の `/state` `/spell/`) と**同じ JSON 契約のサブセット**に揃えると、フロントの
  描画・パーサを無修正で流用できる。リッチな型を wasm-bindgen 境界に晒さない。

- **生成パイプライン**: `cargo build --target wasm32-unknown-unknown --release` →
  `wasm-bindgen <wasm> --out-dir <dir> --target web` で ESM グルー (`*.js`) + `*_bg.wasm` +
  `*.d.ts` が出る。`--target web` はバンドラ無しで `import` できる形。生成物は `.gitignore` し
  ビルドスクリプト (`scripts/build-hobby-wasm.sh`) で再生成可能にする。

## プロジェクトへの適用

- 既存ロジックを別ランタイム (ブラウザ) で動かしたいときは、まず **core が wasm32 で
  ビルドできるか直接スパイク** (`cargo build -p <core> --target wasm32-unknown-unknown`)。
  通れば薄ラッパー crate を足すだけで済む。
- 薄さを保つのが要件のとき、**サーバ層の組み立てロジック (serve.rs の neighbors/excerpts/
  diff/syntect) をコピーしない**。コピーは Feedback.md の「EFFECT_BY_COLOR 3箇所手動同期」
  と同型の drift を生む。再利用が要るなら共有レイヤーに抽出して**サーバもラッパーも両方
  薄くする** ((b) 案)。第一弾は機能を絞って薄ラッパーで出す ((a) 案)。

## フロントエンドからの利用 (Vite + 別ビルドターゲット, Phase 4.12 M2)

- **別ビルドターゲットで本体と分離**: デモ用 wasm アプリは、本体 SPA (rust-embed 同梱の
  `index.html` → `dist/`) とは別の Vite config (`vite.config.hobby.ts`、`base: "./"`,
  `outDir: dist-hobby`, `input: hobby.html`) でビルドする。本体ビルドはこの config を
  読まないので、**本体バイナリ (serve) に wasm が混入しない** (build 後 `dist/assets` に
  wasm が無いことを確認)。`vp build --config vite.config.hobby.ts` で起動。
- **データ供給を注入で純粋化**: `makeWasmDataSource(wasm)` は wasm 関数を引数で受け、JSON
  パースだけする純粋関数 (フェイク注入で vitest 可能)。実際の wasm ロードは
  `loadWasmDataSource()` が**動的 import** で行う (`await import("./wasm/x.js")` +
  `import("./x_bg.wasm?url")`)。テストは前者だけ叩き wasm をロードしない。serve の `api.ts`
  (fetch) と対称な構造。
- **生成物不在でも型チェックを通す ambient 宣言**: 生成 wasm を `.gitignore` すると、
  fresh checkout / CI の本体 `vue-tsc` が `Cannot find module "./wasm/x.js"` で落ちる。
  対策は**ワイルドカード ambient 宣言** (`declare module "*/wasm/magia_hobby.js" { ... }`)
  を別ファイルに置く。TS は**実体があれば実体の .d.ts を優先、無ければワイルドカードを
  フォールバック**として使う (両立・競合しない — 実測確認済み)。これで本体ビルドは wasm
  未生成でも通り、hobby ビルド時 (wasm 生成後) は実体の正確な型で検証される。
- `?url` import は vite-plus/client (vite client 型) が宣言済みで追加不要。
- **erasableSyntaxOnly**: tsconfig で有効だとクラスのパラメータプロパティ
  (`constructor(private x)`)・enum・namespace が使えない。データ供給は**関数ファクトリ**
  (`type WasmDataSource = {...}` + `function make(): WasmDataSource`) で書く。
- 描画コンポーネント (circle/*) と Pinia store は**無修正流用**できる。境界 JSON を本体の
  HTTP 契約のサブセットに揃えてあるため。レスポンス型は `Pick<FullResponse, ...>` で
  サブセットを明示すると drift を型で検出できる。

## 注意点・制約

- 生成 wasm は約 1.9M (syn 込み・wasm-opt 未適用)。初回ロードのコストになるので、配布時は
  `wasm-opt -Oz` やブラウザの gzip/br 圧縮を効かせる。ビルドスクリプトは wasm-opt があれば
  自動適用する作りにしてある (無くても成立)。
- `wasm32` ターゲットと `wasm-bindgen` CLI のバージョンは揃える
  (`rustup target add wasm32-unknown-unknown` / `cargo install wasm-bindgen-cli --version <X>`)。
  CLI と crate のバージョンずれは実行時エラーになる。
- syntect (`source_html`) は magia-cli 依存なので wasm デモには載らない。ソースハイライトが
  欲しければクライアント側 (JS) で行うか、wasm 対応のハイライタを別途検討する。

## 参照

- `crates/magia-hobby/src/lib.rs` — 薄ラッパー本体 (pure 関数 + wasm32 限定 mod wasm)
- `crates/magia-hobby/Cargo.toml` — ターゲット限定依存の書き方
- `scripts/build-hobby-wasm.sh` — wasm ビルド + wasm-bindgen 生成
- `web/src/hobby/dataSource.ts` — 注入で純粋化した WasmDataSource + 動的 import ロード
- `web/src/hobby/wasm-glue.d.ts` — 生成物不在でも通すワイルドカード ambient 宣言
- `web/vite.config.hobby.ts` / `web/package.json` の `build:hobby` — 別ビルドターゲット
- `docs/plans/phase4.12-magia-hobby-wasm-demo.md` — 計画と (a)/(b) 案の判断
