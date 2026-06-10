# Phase 2.3 — フィルター DSL 最小実装

## 出典

- `project-docs/magia/phase2plus-notes-v0.1.md` §3 (フィルター言語)

## 目的

レイヤーの組み合わせをテキスト (`.magia` ファイル) として記述・保存・共有できるようにする。
notes §3.3 の「実行可能なチーム知識」の最小形。

## スコープ

### やること (notes §3.2 構文のサブセット)

- 最小構文のパーサ (`magia-core::filter` または独立モジュール):
  ```
  show: control_flow + effects[network, db]
  hide: type_info
  ```
  - `show:` / `hide:` の2ディレクティブ
  - レイヤー名 + `effects[カテゴリ, ...]` の効果カテゴリ絞り込み
  - `#` コメントと空行
- 適用セマンティクス: `show` に挙がったレイヤーのみ表示し、`effects[...]` は該当カテゴリの
  glyph / Operation ドットだけを残す。`hide` は show より優先
- CLI: `magia render ... --filter review.magia` (既存 `--layers` は show 単独の糖衣として残す)
- dev-server: パレットの状態を `.magia` 形式でエクスポート / `.magia` を読み込んでパレットへ反映
- パースエラーは行番号つきの日本語メッセージ
- fixtures: `fixtures/filters/effects-only.magia` 等のサンプル2〜3個

### やらないこと

- `highlight:` / `filter:` (complexity 等のメトリクス条件) — メトリクス基盤ごと Phase 3
- AND/OR の一般式・括弧 — 必要が立証されてから
- `.magia` のスキーマバージョニング — v1.0 前は破壊的変更で進める

## 設計上の判断

- 効果カテゴリの絞り込みは SVG 後処理ではなく **render 時に Operation/glyph 単位で適用**する
  (CSS では「青い glyph だけ残す」が表現できないため)。`render` の入力に
  `FilterSpec` を渡せる API 拡張を行う (v1.0 前の破壊的変更として許容)
- パーサは手書きの行指向で十分 (パーサジェネレータは導入しない)

## 受け入れ基準

- [ ] `.magia` ファイルで show/hide + effects 絞り込みが効く (CLI / dev-server 両方)
- [ ] パースエラーが行番号つきで報告される
- [ ] `--layers` の既存挙動が変わらない (回帰テスト)
- [ ] `cargo test --workspace` / clippy 警告0

## 後続

- Phase 3 で `highlight: changed_in_pr` / `filter: complexity > 7` などメトリクス条件を拡張
