# Phase 1.1 — IR スケルトン (M1)

## 出典

- `project-docs/magia/spec-v0.1.md` §4 (中間表現スキーマ)
- `project-docs/magia/spec-v0.1.md` §5 (レイヤー定義)
- `project-docs/magia/tech-selection-v0.1.md` §3 M1

## 目的

`magia-core::ir` モジュールに、Phase 1〜6 全てを通じて使う IR の型定義を**最初から全フィールド込みで**実装する。Phase 1 で埋めないフィールドも `Option` または空コレクションでスキーマ上の場所を確保する (spec §4.1 原則1)。

## スコープ

### やること

- `magia-core/src/ir/mod.rs` に spec §4.2 の擬似 Rust 構文に従って型を実装:
  - `MagiaGraph`, `Module`, `Sigil`, `Operation`, `Edge`
  - `SigilKind`, `OperationKind`, `EdgeKind` の enum
  - `EffectSet`, `LayerData`, `EdgeLayerData`, `Cardinality`, `SourceSpan`
  - `LayerData` の全フィールド (control_flow / data_flow / type_info / lifetime / concurrency / test_coverage / profile / git_churn / security / ai_annotations)
  - 内側の構造体 `ControlFlowInfo`, `DataFlowInfo`, `TypeInfo`, `LifetimeInfo`, `ConcurrencyInfo`, `CoverageInfo`, `ProfileInfo`, `ChurnInfo`, `SecurityInfo`, `AiAnnotation`, `CustomEffect`
  - Phase 1 で値が入らないフィールドは struct 定義のみで OK (内部は空でよい)
- ID 型は newtype として実装:
  - `ModuleId(u32)`, `SigilId(u32)` を `#[derive(...)]` で構築
- `serde` 派生で JSON シリアライズ可能に
- ユニットテスト:
  - `MagiaGraph` の空インスタンス生成
  - JSON round-trip (`to_string` → `from_str` で同一性)
  - Phase 1 のレイヤーのみ埋めた例の round-trip

### やらないこと

- AST → IR 変換 (M2)
- 解析・メトリクス計算
- レイヤー内構造体の具体的フィールド設計の精緻化 (Phase 2 以降で詰める)

## 設計上の判断

- IR 内部はグラフライブラリ (`petgraph`) の型に**依存させない** (tech-selection §2.2)。解析時に射影する
- ID は決定論的に採番できる単純な `u32` newtype とする。乱数は使わない (tech-selection §2.7)
- `EffectSet.custom` のような将来拡張用フィールドは `Vec<CustomEffect>` を空ベクタで提供
- `serde_json` の round-trip テストは spec §6.1.4 の決定論要件への布石

## 依存ライブラリの追加

`magia-core/Cargo.toml`:
- `serde = { version = "1", features = ["derive"] }`
- `serde_json = "1"`
- (dev) `pretty_assertions = "1"` (任意、JSON 比較のため)

## 受け入れ基準

- [x] spec §4.2 の全型が定義されている
- [x] `LayerData` の Phase 1 で未使用フィールドも `Option` または空コレクションで存在する
- [x] `cargo test -p magia-core` が round-trip テストを含めて通る (5 テスト)
- [x] `cargo clippy --workspace --all-targets -- -D warnings` が警告0
- [x] `MagiaGraph` のドキュメントコメントが各型の役割を1行で説明している

## 実装結果メモ (2026-06-10)

- IR は `magia-core/src/ir/` 配下のサブモジュール分割で実装 (graph / sigil / operation / edge / layers / source の 6 ファイル)
- spec §4.2 に擬似 Rust 構文として記載のない `OperationPayload` / `Cardinality` / `EdgeLayerData` / `ProjectMetadata` を補完。spec バックポートは Phase 1.2 計画書に持ち越し
- `f64` を含む型 (`Edge`, `EdgeLayerData`, `CoverageInfo`, `ProfileInfo`, `Cardinality`) は `PartialEq` のみ派生 (`Eq` は派生しない)
- `EffectSet` は spec §4.2 直結の直交フラグ集合のため `#[allow(clippy::struct_excessive_bools)]` を局所適用
- レビュー指摘で取り込んだ改善:
  - `SourceSpan.start_column/end_column` を `u32` (0=未取得センチネル) から `Option<u32>` に変更
  - `ControlFlowInfo.branch_count` の集計単位コメントを具体化
  - `deny_unknown_fields` を意図的に付けない旨を `ir/mod.rs` に明記
  - テスト名 `round_trip_is_deterministic` を `same_value_serializes_twice_identically` にリネーム (実質を反映)
- Phase 1.2 に持ち越した Info: ID 採番の一元管理、`ProjectMetadata.root_path` の `PathBuf` 化、ファクトリ関数の `#[must_use]`、spec §4.2 への `OperationPayload` バックポート (phase1.2-syn-to-ir.md に追記済み)
- 持ち越し未スケジュール: `EdgeLayerData` のレイヤー設計が `LayerData` と非対称な件 → Phase 3 でデータフロー解析を導入する際に再設計 (`Feedback.md` に記録)

## 後続

- Phase 1.2 (syn → IR) でこのスキーマに値を埋め始める
