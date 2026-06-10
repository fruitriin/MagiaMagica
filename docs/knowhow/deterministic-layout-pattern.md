# 決定論的レイアウトエンジンの定型

> Phase 1.5 (レイアウトエンジン) で確立。spec §6.1.4 の「同じ IR から常に同一出力」
> 要件を満たすための実装パターン集。

## 発見した知見

- **出力コンテナは `HashMap` でなく `BTreeMap`**: positions を `HashMap<SigilId, Point>`
  で返すと、後段 (SVG レンダラ) が走査したときの要素順が実行ごとに揺れ、
  スナップショットテストが壊れる。キー参照だけなら HashMap でも決定論的だが、
  「いずれ誰かが iterate する」公開コンテナは最初から BTreeMap にする
- **「隣接要素の入れ替え」型の交差最小化は放射状配置では無意味**: 接続線が全て
  「親中心 → 子中心」の放射線のとき、同一親の子どうしをスロット間で入れ替えても
  線分集合の幾何は一切変わらない (同じ中心から同じスロットへの線分が残るだけ)。
  最適化の自由度は**ファン全体の回転**にあるので、回転角の貪欲 hill-climbing に
  置き換える。探索順 (親 SigilId 順)・ステップ・パス上限を固定すれば決定論的
- **petgraph の `neighbors()` は挿入逆順**: そのまま使うと Edge 追加順に依存する。
  子リストは必ず `sort_by_key(|s| s.id)` してから処理する
- **kurbo で bounding box は自前実装不要**: `Rect::union` / `Rect::inflate` /
  `Rect::contains`、`Point + Vec2`、`Vec2 * f64` が揃っている。
  `Point::ZERO` / `Rect::ZERO` も使える
- 浮動小数の決定論性は「同じ演算を同じ順序で」やれば成立する (IEEE754)。
  乱数・HashMap 走査・並列化を避ければ bit 単位で一致する
- **対になる属性 (中心と半径など) は Map 2本に分けず1レコードに**: `positions` と
  `radii` を別 Map で持つと「片方にだけある」不整合状態が型で防げず、
  `unwrap_or(0.0)` のサイレントフォールバックが入り込む (Phase 1.5 レビュー指摘)。
  `struct PlacedSigil { center, radius }` の1本にする
- 端点同一視などの許容誤差は `f64::EPSILON` (機械イプシロン ≈2.2e-16) でなく、
  座標スケールに見合う `1e-9` 等を使う。sin/cos 経由の座標は機械イプシロンを超えてずれる

## プロジェクトへの適用

- `magia-core/src/layout/` — `layout()` / `layout_with(options)`。
  3段階 (MainRing 中央固定 → AuxRing 極座標 → SummonGlyph 放射) を関数分離
- 半径・ギャップ定数は `layout/constants.rs` に集約し、M6 の SVG レンダラが同じ値を参照する
- 防御規約: `positions` は常に全 Sigil を覆う (MainRing 欠落・到達不能 Sigil は原点フォールバック)

## 注意点・制約

- 決定論性テストは「10回連続で完全一致」のような反復で書く。1回の比較では
  HashMap 由来の揺れを検出できないことがある
- 交差判定は外積符号の「真の交差」のみ数え、端点共有 (放射線の中心) は除外する。
  共線・端点接触の退化ケースは「交差なし」に倒す (カウント用途では十分)

## 参照

- `crates/magia-core/src/layout/mod.rs`
- `crates/magia-core/src/layout/crossing.rs`
- `crates/magia-core/tests/layout_engine.rs`
- `project-docs/magia/spec-v0.1.md` §6.1.4
- `project-docs/magia/tech-selection-v0.1.md` §2.2 (petgraph), §2.3 (kurbo)
