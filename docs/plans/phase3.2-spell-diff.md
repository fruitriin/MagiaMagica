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

- [x] 変更前後 fixture で added/removed/changed の強調が SVG に現れる
- [x] `highlight: changed` が diff 文脈で機能し、文脈なしでは案内エラー
- [x] 決定論性・既存スナップショット不変 (diff なし時の出力は従来どおり)
- [x] `cargo test --workspace` / clippy 警告0

## 後続

- 3.3 が PR コメントにこの SVG を貼る

## 実装結果メモ (2026-06-11)

- `SpellDiff` を構造拡張 (v1.0 前の破壊的変更): added/removed は `Vec<NodeRef>`
  (経路 + 出自側 SigilId)、NodeChange に before/after の SigilId。overlay が layout
  位置と突き合わせるための内部運搬で、JSON 契約には引き続き経路文字列のみを出す
- レンダラは `write_document` に `Option<&OverlayContext>` を1引数足す形に抑え、
  diff なし経路の出力を完全不変にした (既存スナップショット全部無変更で通過)
- 強調の意匠: 金ハロー=追加 / シアンハロー=変更 / 灰破線ゴースト=削除 (本体半径)。
  ゴーストが after キャンバスをはみ出す分は `Rect::union` で viewBox を拡張
- `--svg` は `--json` と排他、`--filter` と `-o` は `requires = "svg"`。
  `magia render` で highlight 指定時は「diff 文脈がありません (magia diff --svg で
  使用してください)」の案内エラー (spec v0.3 §8 の黙って無視しない原則)
- 目視素材2点をオーナー送付済み (fixture + **自己ホスティング実例**: Phase 3.1 の
  metrics_sentence リファクタ実コミット diff — 入れ子ループ群がゴーストとして消える図)。
  **意匠判定待ち**
- レビュー (Stage 2): Critical 0 / Warning 2 / Suggestion 5 → 全件対応。
  存在性確認を `overlay_anchor` に一本化 (kind と position の片方だけ見つかる入力で
  原点に無言描画しない)、kind 表の O(diff×sigil) 線形探索を BTreeMap 一括構築に、
  ゴースト不透明度を定数化、メッセージ・doc の表現統一
