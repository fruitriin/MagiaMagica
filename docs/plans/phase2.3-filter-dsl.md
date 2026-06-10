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

- [x] `.magia` ファイルで show/hide + effects 絞り込みが効く (CLI は render 時適用、dev-server は可視性を往復し effects[] は CLI 案内 — 計画どおり)
- [x] パースエラーが行番号つきで報告される (Rust / JS 両側、preview で実機確認)
- [x] `--layers` の既存挙動が変わらない (回帰テスト維持。実装は後処理から render 時ゲートに置換)
- [x] `cargo test --workspace` (141本) / clippy 警告0

## 後続

- Phase 3 で `highlight: changed_in_pr` / `filter: complexity > 7` などメトリクス条件を拡張

## 実装結果メモ (2026-06-11)

### 設計判断の確定

- 語彙 (LayerName / EffectCategory) は enum + FromStr (候補一覧つきエラー) で
  CLI / DSL / serve UI の三者共有。palette は category_of / color_of に分離し、
  フィルタは「色」でなく「分類」で絞る
- API は `render_with(graph, layout, style, &FilterSpec)` (layout_with の前例踏襲)。
  `--layers` は `FilterSpec::show_only` の糖衣となり、Phase 1.7 の SVG 行単位
  後処理 (filter_layers) は廃止
- 予約語 (highlight:/filter:) は「Phase 3 で導入予定」と明示エラーで案内
- serve の DSL ボックスは可視性のみ適用し、effects[] があれば
  「CLI --filter で適用される」旨を表示 (spec §5.3 の CSS 切替原則と整合)

### レビュー対応 (Stage 2)

- 修正: show の同一レイヤー重複をエラー化 (H1、後続カテゴリの無言ドロップ防止 →
  spec §8 にも明記) / 空ブラケット・末尾カンマの専用エラー (M1) /
  カテゴリ絞り込みテストを effects 層要素に限定 (M2、control_flow の黒との誤判定防止) /
  JS 側でも hide のカテゴリ指定を拒否 (M3、Rust とのセマンティクス一致) /
  `FilterSpec::show_only` コンストラクタ (L1) / serve の拡張点コメント (L3)
- 受理: LayerName::ALL の要素数ハードコード (L2、enum 追加時に型と両方更新する
  Rust の制約。コメント済み)
