# Phase 3.1 — IR 差分エンジン (Spell Diff の中核)

## 出典

- `project-docs/magia/phase2plus-notes-v0.1.md` §4.2 (差分エンジン)
- spec v0.3 §9 (Phase 3.0 で確定する契約)

## 目的

2つの `MagiaGraph` (PR 前後の同一関数) を比較し、構造変化とメトリクス変化を
決定論的に算出する差分エンジンを実装する。CI 統合 (3.3) と視覚 diff (3.2) の中核。

## スコープ

### やること

- `magia-core::diff` 新設: `diff(before: &MagiaGraph, after: &MagiaGraph) -> SpellDiff`
  - `SpellDiff { added: Vec<...>, removed: Vec<...>, changed: Vec<...>, metrics: MetricsDelta }`
  - **対応付けは SigilId でなく構造マッチング**: ID はリビジョン間で安定しないため、
    (kind, role.anchor/ordinal, call_target 等) のキーで突き合わせる (詳細は spec v0.3 契約)
  - メトリクス delta: 複雑度・副作用カテゴリ数・リング数・glyph 数・早期リターン数
    (transcript と同じ集計関数を共有する)
- CLI: `magia diff <BEFORE.rs> <AFTER.rs> --fn <NAME>` — テキストレポート
  (transcript 風の日本語文面 + 変化のみ列挙)。`--json` で機械可読出力
- git 連携の糖衣: `magia diff --git <REV> <FILE> --fn <NAME>` は Phase 3.3 と合流して判断
  (本計画ではファイル2つの比較のみ)
- テスト: fixture ペア (変更前後) を fixtures/diff/ に置き、追加/削除/変更/不変の4象限 +
  決定論性 + 「同一入力なら空 diff」

### やらないこと

- 視覚的 diff (SVG 上の強調) — 3.2
- PR コメント投稿 — 3.3
- 関数の追加・削除 (ファイルレベル diff) — 単一関数の中身の比較に限定

## 設計上の判断

- 構造マッチングの曖昧性 (同 anchor に同種リングが複数) は「ordinal 順の貪欲対応 +
  余りは added/removed」で決定論的に倒す
- メトリクス集計は transcript の関数を共有し、二重実装を避ける (必要なら集計部を
  `metrics` モジュールへ抽出する小リファクタを含む)

## 受け入れ基準

- [ ] 追加/削除/変更/不変の4象限テストが通る
- [ ] 同一入力で空 diff、決定論性 (5回一致)
- [ ] `magia diff` がテキストと `--json` で動く
- [ ] `cargo test --workspace` / clippy 警告0

## 後続

- 3.2 が SpellDiff を SVG 強調に、3.3 が PR コメントに変換する
