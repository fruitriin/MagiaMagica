# Phase 1.0 — Cargo Workspace ブートストラップ

## 出典

- `project-docs/magia/spec-v0.1.md` §3.2 (モジュール構成)
- `project-docs/magia/tech-selection-v0.1.md` §1, §4

## 目的

MagiaMagica Phase 1 の実装基盤として cargo workspace を立ち上げ、ビルド・Lint・テストコマンドを `CLAUDE.repo.md` に登録するところまでを完了する。以後のマイルストーン (M1〜M7) はこの土台に積み増す。

## スコープ

### やること

- `magia/` 配下に cargo workspace を作成
  - `crates/magia-core/` (lib)
  - `crates/magia-rust/` (lib)
  - `crates/magia-cli/` (bin)
  - `fixtures/` ディレクトリ (空でよい)
- `Cargo.toml` (workspace) の resolver = "2" / edition = "2024" / 共通 lints 設定
- `rustfmt.toml` / `clippy` `-D warnings` を最初から有効化
- 最小の通る `cargo build` / `cargo test` を確保 (各クレートに dummy 関数 + 1 テスト)
- `CLAUDE.repo.md` の「ビルド・Lint・テスト」セクションを以下で更新:
  - ビルド: `cargo build --workspace`
  - Lint: `cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --check`
  - テスト: `cargo test --workspace`
- `.gitignore` に `target/` 追加

### やらないこと

- IR / 解析 / レンダラの実装 (M1 以降)
- CI 設定 (GitHub Actions 等) — 別計画で扱う

### ライセンスと配布方針 (オーナー確定: 2026-06-10)

- **ライセンス: MIT** — `LICENSE` ファイルをリポジトリルートに配置し、`Cargo.toml` の `license = "MIT"` を全クレートに付与
- **crates.io publish の意図**: 将来公開する想定で名前を予約しておく。Phase 1.0 では実 publish はしないが、以下を満たしておく:
  - 全クレートに `description` / `license` / `repository` / `readme` フィールドを記入
  - `magia-core` / `magia-rust` / `magia-cli` の名前を `cargo search` で確認し、衝突があれば早めに対応
  - 公開時のためのバージョン規約: `0.1.0` から開始 (workspace.package.version で一元管理)
- **MSRV**: 最新の安定版 Rust に追従する。`rust-version` は `Cargo.toml` (workspace.package) に固定 (例: `1.85`) し、CI も同バージョンで実行

## 設計上の判断

- IR は M1 時点では `magia-core::ir` モジュールとして開始する (tech-selection §1)。`magia-ir` への分離は Phase 2 着手時に判断する
- 依存関係の追加はマイルストーン側で行う。M0 では `Cargo.toml` の `[dependencies]` は空でよい

## 受け入れ基準

- [x] `LICENSE` (MIT) がリポジトリルートに存在する
- [x] 全クレートの `Cargo.toml` に `license = "MIT"` が入っている (workspace.package 継承)
- [x] `workspace.package.rust-version` が pin されている (1.94)
- [x] `cargo build --workspace` が通る
- [x] `cargo test --workspace` が通る (各クレートに1テスト)
- [x] `cargo clippy --workspace --all-targets -- -D warnings` が警告0で通る
- [x] `cargo fmt --check` が通る
- [x] `CLAUDE.repo.md` のビルド・Lint・テストコマンドが上記で記載されている
- [x] `magia-cli` バイナリが `cargo run -p magia-cli` で起動する (no-op でよい)

## 実装結果メモ (2026-06-10)

- ワークスペース配置は計画書記載の `magia/` 配下ではなく**リポジトリルート直下**に変更。リポジトリ自身が MagiaMagica なので冗長なネストを避け、Phase 1.7 の `cargo install --path crates/magia-cli` 表記と整合させた
- README.md は Phase 1.7 で整備するため、`workspace.package.readme` フィールドは Phase 1.0 では未指定
- レビュー指摘で取り込んだ改善:
  - `workspace.lints.rust.unsafe_code = "deny"` (warn から強化)
  - path 依存に明示的 version (`{ path = "../magia-core", version = "0.1.0" }`) を付与し crates.io publish 準備
  - workspace.package に `keywords` / `categories` を追加
  - `magia-cli` にスモークテスト (`greeting_mentions_phase`) を追加
  - `rustfmt.toml` から冗長な `use_small_heuristics = "Default"` を削除

## 後続

- Phase 1.1 (IR スケルトン) に着手するための前提を満たす
