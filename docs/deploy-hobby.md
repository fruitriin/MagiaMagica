# magia-hobby (WASM デモ版) デプロイ手順書

Phase 4.12 M3。`magia serve` を立てずに、**静的ホスティングへ上げて URL を配って遊べる**
状態にするための作業手順。デモ本体はクライアント完結 (WASM) なのでサーバ計算は不要、
静的ファイルを置くだけで動く。

> この手順は「実行ではなく手順を成果にする」方針 (オーナー指示 2026-06-27)。実デプロイは
> オーナーのアカウント・認証で行う。下記コマンドをそのままなぞれば再現できる。

## 成果物

`web/dist-hobby/` 一式 (静的ファイル):

```
dist-hobby/
  index.html                     # エントリ (base="./" で任意パス配下でも動く)
  assets/
    hobby-*.js                   # アプリ本体 (gzip 約 38KB)
    hobby-*.css
    magia_hobby-*.js             # wasm-bindgen グルー
    magia_hobby_bg-*.wasm        # WASM 本体 (約 1.95MB / gzip 約 482KB)
```

- **サーバ不要**。`/state` `/spell` 等の API も叩かない (それは serve 版の領分)。
- コードはブラウザから一切出ない (パース・描画は WASM がローカルで行う)。
- `index.html` がルートなので、Vercel / Cloudflare Pages / GitHub Pages 等で**リライト設定
  なしにそのまま**配信できる。

## ビルド (ローカル / CI 共通)

前提ツール (初回のみ):

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version <Cargo.lock の wasm-bindgen と同じ版>
# 例: cargo install wasm-bindgen-cli --version 0.2.126
# (scripts/build-hobby-wasm.sh が版不一致を検出して案内する)
```

ビルド:

```bash
cd web
bun install            # 初回 / 依存更新後
bun run build:hobby    # = wasm 生成 → vue-tsc → vite build → index.html へリネーム
```

成果物は `web/dist-hobby/`。`bun run build:hobby` は次を順に行う:
1. `scripts/build-hobby-wasm.sh` — `cargo build --target wasm32-unknown-unknown --release`
   + `wasm-bindgen` で `web/src/hobby/wasm/` に JS グルー生成 (wasm-opt があれば `-Oz`)
2. `vue-tsc` — 型チェック (生成 wasm の実体型で検証)
3. `vp build --config vite.config.hobby.ts` — `dist-hobby/` へバンドル (`base: "./"`)
4. `dist-hobby/hobby.html` → `dist-hobby/index.html` にリネーム

## デプロイ先別の手順

> **推奨は C 案 (事前ビルド)**。A/B 案はデプロイのたびに `cargo install wasm-bindgen-cli` を
> ソースから再ビルドする (約2〜5分) うえ、Vercel / CF Pages は `~/.cargo` を必ずしも
> キャッシュしない。Rust ツールチェインをビルド環境に入れる手間も要る。**ローカル or GitHub
> Actions で `dist-hobby/` を作って上げる C 案が速く・再現性が高い**。
>
> 注: `build:hobby` は `bash` と `mv` を使う (Linux / macOS / WSL 前提)。CI は全て Linux なので
> デプロイ経路では問題ない。Windows ネイティブでローカルビルドするなら WSL を使う。

### A. Vercel

**ダッシュボードで設定する場合** (推奨 — モノレポなので Root Directory に注意):

1. New Project → リポジトリ `fruitriin/MagiaMagica` を import
2. **Root Directory**: `web`
3. **Build Command**: `bun run build:hobby`
   - Vercel のビルド環境に Rust + wasm target + wasm-bindgen-cli が要る。標準イメージに
     Rust は入るが wasm target / wasm-bindgen は入らないので、Build Command の前段で
     用意する。Install Command を次に上書きする:
     ```
     rustup target add wasm32-unknown-unknown && cargo install wasm-bindgen-cli --version <版> && bun install
     ```
     (Rust が無いプランなら下記「事前ビルド成果物を上げる」方式にする)
4. **Output Directory**: `dist-hobby`
5. Deploy

**`vercel.json` で設定する場合** (リポジトリに置く例。Root=web 前提):

```json
{
  "buildCommand": "bun run build:hobby",
  "outputDirectory": "dist-hobby",
  "installCommand": "rustup target add wasm32-unknown-unknown && cargo install wasm-bindgen-cli --version 0.2.126 && bun install"
}
```

### B. Cloudflare Pages

1. Create application → Pages → リポジトリ接続
2. **Build command**:
   `cd web && rustup target add wasm32-unknown-unknown && cargo install wasm-bindgen-cli --version <版> && bun install && bun run build:hobby`
3. **Build output directory**: `web/dist-hobby`
4. 環境変数で Rust ツールチェインが要る場合は `RUST_VERSION` 等を設定 (CF の build image に
   Rust が無ければ下記「事前ビルド」方式)

### C. 事前ビルド成果物を上げる方式 (ビルド環境に Rust を入れたくない / 入らない場合)

ローカル or GitHub Actions で `bun run build:hobby` まで済ませ、`web/dist-hobby/` だけを
静的ホスティングに上げる。最も移植性が高い。

- **Vercel**: `vercel deploy --prebuilt` 相当 / もしくは Output Directory に既存 `dist-hobby`
  を指定し Build Command を空に。
- **Cloudflare Pages**: `npx wrangler pages deploy web/dist-hobby`
- **GitHub Pages**: `dist-hobby/` を `gh-pages` ブランチに push (または Actions の
  `actions/deploy-pages`)。サブパス (`/<repo>/`) 配信でも `base: "./"` なのでアセットは解決する。

GitHub Actions の例 (事前ビルド → 任意の deploy アクションへ渡す):

```yaml
- uses: dtolnay/rust-toolchain@stable
  with: { targets: wasm32-unknown-unknown }
- run: cargo install wasm-bindgen-cli --version 0.2.126
- uses: oven-sh/setup-bun@v2
- run: cd web && bun install && bun run build:hobby
# この後 web/dist-hobby を deploy-pages / wrangler / vercel に渡す
```

## 配信時の注意

- **MIME**: `.wasm` は `application/wasm` で返る必要がある。Vercel / CF Pages / GitHub Pages は
  既定で正しい Content-Type を付ける (手当て不要)。自前 nginx 等なら `types` に追加する。
- **圧縮**: wasm は gzip で 1.95MB → 約 482KB。CDN の自動 gzip/br が効く先を選ぶと初回ロードが軽い。
- **キャッシュ**: `assets/*` はハッシュ付きファイル名なので `Cache-Control: immutable` を長めに
  効かせてよい。`index.html` は短め (no-cache) にして更新を即反映。
- **COOP/COEP は不要**: SharedArrayBuffer / wasm threads を使っていないので、クロスオリジン
  分離ヘッダは要らない。
- **OG カード**: 現状は `hobby.html` (→ index.html) の固定メタ。SNS で**断片ごとの魔法陣
  プレビュー**を出したくなったら、SSR で魔法陣を PNG 化する serverless function を1個足す
  (計画 Phase 4.12 後続)。OG クローラは JS も `#fragment` も読まないため、per-snippet カードは
  この function が要る。ここが「functions が再登場する唯一の場所」。

## 共有リンク (`#code=`)

デプロイ後、デモ上の「共有リンクをコピー」ボタンが
`https://<host>/#code=<base64url(ソース)>&fn=<関数>` を生成する。これを踏むと同じソース +
関数の魔法陣が再現される (サーバ不要、`web/src/hobby/share.ts`)。短い関数なら URL 長に十分
収まる。将来さらに長い断片を載せたくなったら `share.ts` の `encodeShare` を
`CompressionStream` (deflate) 差し替えで対応できる (エンコードはこの関数に閉じている)。
