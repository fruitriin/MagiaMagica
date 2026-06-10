# Phase 1.2 — syn → IR (M2)

## 出典

- `project-docs/magia/spec-v0.1.md` §10.3 (Phase 1 で収集する情報)
- `project-docs/magia/tech-selection-v0.1.md` §2.1, §3 M2

## 目的

`magia-rust` クレートに、Rust ソースの単一関数を syn 2.x で解析し、`MainRing` (関数本体を表す `Sigil`) と `Operation` 列を持つ `MagiaGraph` を構築するアダプタを実装する。

## スコープ

### やること

- `magia-rust/src/lib.rs` に公開 API:
  - `parse_function(source: &str, fn_name: &str) -> Result<MagiaGraph, Error>`
  - `list_functions(source: &str) -> Result<Vec<String>, Error>`
- syn 2.x の `Visit` トレイトで AST 走査
- 関数本体の各文 (`syn::Stmt`) を Phase 1 で対応する `Operation` に変換:
  - 通常式: `OperationKind::Compute`
  - return: `OperationKind::Return`
  - `?` 演算子: `OperationKind::Return` (早期リターンとして) — ただし spec §6.1.3 の「早期リターン記号」のために専用フラグを残す
- 関数の `SigilKind::MainRing` Sigil を1つ作成し、Operation 列を `content` に格納
- spec §10.3 のうち本マイルストーンで埋めるもの:
  - 関数名 / シグネチャ (引数型、戻り値型) → `LayerData.type_info`
  - async fn 判定 → `SigilKind` ではなく `LayerData.concurrency` に最小限のフラグ
  - unsafe ブロックの位置 → `EffectSet.unsafe_block = true` (Operation 単位)
  - Result / Option 戻り値型の判定 → `type_info` 内のフラグ
- `--emit-ir` 相当の JSON 出力 (CLI 統合は M7 だが、`magia-rust` のテストで IR JSON を確認可能にする)
- ユニットテスト:
  - `fn foo() -> i32 { 42 }` → MainRing + 単一 Compute Operation
  - `fn bar() { return; }` → Return Operation を含む
  - `async fn baz() { ... }` → concurrency フラグが立つ
  - シグネチャ抽出 (`fn add(a: i32, b: i32) -> i32` の引数型・戻り値型)

### やらないこと

- 制御構造の AuxRing 化 (M3)
- 呼び出し先の抽出 (M4)
- マクロ展開後の構造 / lifetime / generic / dyn Trait (spec §10.3 後段で明記された除外項目)
- 呼び出し先のフルパス解決 (M4 で同ファイル内 `use` 文を機械的に展開する近似実装)

## 設計上の判断

- tech-selection §2.1 の Phase 1a に従い、**syn 単体**で進める。`rust-analyzer` (`ra_ap_*`) は導入しない
- syn の features は `["full", "visit", "extra-traits"]`
- `SourceSpan` は `proc_macro2::Span` から行/列を取得して埋める。列情報が取れないケースは `Option<u32>` の `None` で表現する (Phase 1.1 で `SourceSpan.start_column/end_column` を `Option<u32>` に変更済み)
- `SigilId` の採番は `parse_function` 内で 0 から決定論的に増やす (乱数禁止)。フィールド `pub u32` への直接代入で採番を破壊しないよう、内部に `SigilIdAllocator { next: u32 }` を用意し採番を一箇所に集約する
- エラーは `thiserror` で `magia_rust::Error` を定義 (構文エラー / 関数未発見 など)

## Phase 1.1 レビューからの持ち越し事項 (Info)

Phase 1.2 着手時に検討すべきメモ:

- **ID 採番の一元管理**: `SigilId(pub u32)` / `ModuleId(pub u32)` のフィールドが `pub` なので直接構築が可能。M2 着手時に `Allocator` パターンを導入する判断をする
- **`ProjectMetadata.root_path` の `PathBuf` 化**: 現状 `String`。Phase 1.2 で実ファイルパスを書き込み始めるタイミングで `PathBuf` への移行を検討
- **ファクトリ関数の `#[must_use]`**: `parse_function` 等は戻り値が必ず使われるので `#[must_use]` を付ける
- **spec §4.2 への `OperationPayload` バックポート**: `OperationPayload` は仕様書の擬似 Rust 構文に未記載。Phase 1.2 着手時に spec-v0.1.md (またはバージョン上げ) へ追記する候補

## 依存ライブラリの追加

`magia-rust/Cargo.toml`:
- `syn = { version = "2", features = ["full", "visit", "extra-traits"] }`
- `proc-macro2 = "1"`
- `thiserror = "1"`
- `magia-core = { path = "../magia-core" }`

## 受け入れ基準

- [ ] `parse_function` が単一の通常関数で MainRing + Operation 列を生成する
- [ ] async / unsafe / Result-Option 戻り値が IR に反映される
- [ ] シグネチャ (引数型・戻り値型) が `LayerData.type_info` に入る
- [ ] 関数未発見 / 構文エラー時に明確なエラーを返す
- [ ] `cargo test -p magia-rust` の全テストが通る (最低5ケース)
- [ ] IR の JSON シリアライズ結果が決定論的 (同じ入力で同じ出力)

## 後続

- Phase 1.3 で AuxRing (制御構造) を追加
- Phase 1.4 で召喚記号 (呼び出し) を追加
