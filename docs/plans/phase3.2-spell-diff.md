# Phase 3.2 — 視覚的 Spell Diff と highlight:

## 出典

- `project-docs/magia/phase2plus-notes-v0.1.md` §4.2 (視覚的 diff)、§3.2 (`highlight:`)
- spec v0.3 §9 / highlight 文法 (Phase 3.0 で確定)
- Phase 3.1 の `SpellDiff`

## 目的

差分を魔法陣の上に**強調**として描く。「PR で何が変わったか」が図形の変化として
一目で分かる状態にする (notes: 構造変化のハイライト)。

## スコープ

### やること

- レンダラ拡張: `SpellDiff` を受け取り、追加 Sigil に `diff-added`、削除に
  `diff-removed` (ゴースト表示 = 半透明破線で残す)、変更に `diff-changed` の
  class / 強調スタイルを付ける。レイヤーとは独立した**強調チャネル** (`<g class="overlay-diff">`)
- DSL: 予約語だった `highlight: changed` を解禁 (diff 文脈があるときのみ有効。
  ないときは「diff 文脈がありません」のエラー)
- CLI: `magia diff --svg -o out.svg` で before を基準に after を強調描画
- レイアウトの安定性を活用: 変更が局所なら図形も局所変化 (spec §6.1.4 の前提が活きる)。
  added/removed の位置は after/before それぞれのレイアウトから取り、ゴーストは
  before の位置に重ねる
- ゴールデンテスト + 目視素材の生成 (オーナー判定向け)

### やらないこと

- A/B 重ね合わせの対話 UI (Phase 3 後半以降、serve への統合は別計画)
- レイヤーごとの差分分解表示 (notes §4.2 後段) — 最小形ができてから

## 設計上の判断

- 強調は色変更でなく**輪郭・背景ハロー**で表現する (効果カテゴリの色相規約と衝突させない)
- diff 文脈は render の新しい入力 (`Option<&SpellDiff>`) として渡す
  (FilterSpec と同様の `render_with` 系拡張)

## 受け入れ基準

- [ ] 変更前後 fixture で added/removed/changed の強調が SVG に現れる
- [ ] `highlight: changed` が diff 文脈で機能し、文脈なしでは案内エラー
- [ ] 決定論性・既存スナップショット不変 (diff なし時の出力は従来どおり)
- [ ] `cargo test --workspace` / clippy 警告0

## 後続

- 3.3 が PR コメントにこの SVG を貼る
