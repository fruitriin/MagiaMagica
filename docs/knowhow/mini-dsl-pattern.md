# 小さな DSL を Rust に足すときの定型

> Phase 2.3 (フィルター言語 `.magia`) で確立。行指向の最小 DSL をパーサジェネレータ
> なしで足し、CLI / DSL / UI の三者で語彙を共有するパターン。

## 発見した知見

- **語彙は enum + FromStr (候補一覧つきエラー) で一元化する**: レイヤー名・カテゴリ名の
  ような有限語彙を文字列のまま扱うと、候補提示とタイポ検出が CLI / DSL / UI に散らばる。
  `enum + ALL 定数 + FromStr` にすると「未知の名前 → 使用可能な候補つきエラー」が
  全経路で同じになる
- **将来の予約語は黙って無視せず専用エラーで案内する**: `highlight:` / `filter:` を
  パース対象外として無視すると、利用者は「効いている」と誤解する。
  「Phase 3 で導入予定の予約語です」と明示的に拒否する
- **分類と着色を分離する**: `EffectSet → カテゴリ` (分類) と `カテゴリ → 色` (着色) を
  別関数にしておくと、フィルタの絞り込みが「色」でなく「分類」を再利用でき、
  色変更がフィルタ語彙に波及しない
- **公開 API の拡張は `xxx_with` の前例踏襲**: `render` を壊さず
  `render_with(…, &FilterSpec)` を足す (`layout` / `layout_with` と同型)。
  既定値は `FilterSpec::default()` = フィルタなし
- パースエラーは「行番号 + 日本語メッセージ」の構造体 (`{ line, message }`) にし、
  CLI 層でファイル名を前置する (`bad.magia: 2行目: ...`)

## プロジェクトへの適用

- `crates/magia-core/src/filter.rs` — FilterSpec / LayerName / EffectCategory。
  `hide` が `show` に優先、`effects[...]` は effects レイヤーのみ
- レイヤー単位は `<g>` 出力ゲート、カテゴリ単位は記号ごとの skip で render 時適用
- serve のパレットは可視性のみ DSL と往復し、effects[] は CLI を案内する

## 注意点・制約

- clap の排他オプションは手書き `bail!` でなく `conflicts_with` 属性を使う
  (ヘルプ・エラーメッセージに自動反映される) — clap-cli-integration-pattern.md にも追記済み

## 参照

- `crates/magia-core/src/filter.rs`
- `crates/magia-cli/src/main.rs` (`build_filter`)
- `project-docs/magia/spec-v0.2.md` §8
