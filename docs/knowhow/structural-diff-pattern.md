# ID 非依存の構造 diff パターン

> Phase 3.1 (Spell Diff エンジン) で確立、Phase 3.2 (overlay 描画) で拡張。
> リビジョン間で安定しない ID を含む木構造 IR の before/after を決定論的に
> マッチングして差分化し、SVG 上に強調として重ねるまでの定型。

## 発見した知見

### 1. 構造キーによるマッチング (ID は使わない)

`SigilId` は syn の走査順に採番されるためリビジョン間で安定しない (spec v0.3 §9.2)。
対応付けは「そのノードが構造上どこにいるか」を表すキーで行う:

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NodeKey {
    /// (種別の判別値, anchor_operation, ordinal)
    Ring(u8, u32, u32),
    /// 召喚記号: 呼び出し先 (所属リングはパスで表現される)
    Glyph(String),
}
```

- `Ord` を derive すると **BTreeMap のキー化と決定論的な反復順が同時に手に入る**
- enum バリアントに種別の「判別値 (u8)」を入れると、`AuxRingKind` のような
  ネストした enum を `Ord` 比較に持ち込まずに済む

### 2. 同キー複数の貪欲対応

同じ関数を2回呼ぶ・同条件の if が2つある等、同キーのノードは複数になりうる。
決定論的な解消は3段:

1. 木構築時に子を `(NodeKey, SigilId)` でソートする (SigilId = ソース出現順)
2. before/after それぞれをキーでグループ化し、同キー内は出現順に `min(len)` まで対応付ける
3. 余りを追加 (after 側) / 削除 (before 側) とし、部分木ごと平坦化して報告する

### 3. 共有メトリクスの一本化

before/after 両方が数値を報告する機能 (transcript の規模文と diff の MetricsDelta) は、
**集計を専用モジュール (`metrics::measure`) に一本化**して双方から呼ぶ。
二重実装は数字の食い違いに直結し、ツールの信頼性を毀損する
(「書き起こしは複雑度3と言うが diff は4と言う」が最悪のバグ)。

### 4. ライフタイム付き参照を返すクロージャは関数に逃がす

```rust
// NG: 外側のスコープに 'a が無いのでコンパイル不能
let group = |tree: &'a Tree<'a>| -> BTreeMap<NodeKey, Vec<&'a Tree<'a>>> { ... };

// OK: スタンドアロン関数ならライフタイムパラメータを宣言できる
fn group_children<'t, 'a>(tree: &'t Tree<'a>) -> BTreeMap<NodeKey, Vec<&'t Tree<'a>>> { ... }
```

クロージャはライフタイムパラメータを宣言できない。参照を返すヘルパは最初から関数で書く。

### 5. JSON 出力契約は CLI 側で明示構築する

core の diff 型 (`SpellDiff`) に `Serialize` を生やさず、CLI 側で
`serde_json::json!` により明示的に組み立てる (`diff_to_json`)。

- 内部表現のリファクタが外部 JSON 契約を壊さない (情報隠蔽、POSD)
- 契約の形が CLI のコードに1箇所で見える
- IR (`emit-ir`) のように「内部表現そのものが契約」のものは逆に derive でよい —
  使い分けの軸は「型の形 = 公開契約か?」

### 6. 差分 overlay を既存レンダラに足す定型 (Phase 3.2)

- **強調チャネルの独立性**: overlay は基底描画と独立した `<g class="overlay-diff">` として
  最後に描き、レイヤーの show/hide ゲートを**通さない** (spec v0.3 §8)。基底レンダラへの
  変更は描画関数に `Option<&OverlayContext>` を1引数足すだけに抑えると、diff なし経路の
  出力が完全不変になりスナップショット互換が自動的に守られる
- **ID の橋渡し**: diff 結果 (経路文字列) と描画位置 (SigilId → layout) の突き合わせは、
  diff 型に**出自リビジョン側の SigilId** を持たせるのが最小 (added = after 側、
  removed = before 側、changed = 両方)。SigilId はリビジョン間不安定なので
  JSON 等の外部契約には経路文字列のみを出し、ID は同一プロセス内専用とする
- **ゴーストと viewBox**: 削除要素のゴーストは before レイアウトの座標に描くため、
  after のキャンバスからはみ出しうる — `Rect::union` で viewBox を必要分だけ拡張する。
  テストは「ゴースト円が viewBox 矩形に内包される」を数値で検証する形が頑健
- **強調の意匠**: 記号の色変更でなく**輪郭ハロー / 破線ゴースト**にすると効果カテゴリの
  色相規約と衝突しない。色は palette に `DIFF_*` として追加し、既存6色と異なる色相
  (金=追加 / シアン=変更 / 灰=削除) を選ぶ
- **目視素材の説得力**: 合成 fixture に加えて「自リポジトリの実コミットの before/after」
  (`git show REV~1:path` で取得) を使うと機能の説得力が一気に上がる
  (metrics_sentence のリファクタで入れ子ループ群がゴーストになって消える図が撮れた)

## プロジェクトへの適用

- `crates/magia-core/src/diff.rs` — 木構築 → NodeKey マッチング → SpellDiff
- `crates/magia-core/src/metrics.rs` — transcript / diff 共有の `measure`
- `crates/magia-cli/src/main.rs` の `diff_to_json` — JSON 契約の明示構築
- fixture 設計: `fixtures/diff/before.rs` / `after.rs` は **1ペアで4象限
  (追加/削除/変更/不変) を全て踏む**よう設計し、fixture 冒頭コメントに意図を書く

## 注意点・制約

- anchor_operation はメインリングの操作列に対する添字なので、**操作列の前方に
  挿入があると後続の分岐の anchor がずれて「削除+追加」として報告される**。
  これは構造マッチングの原理的限界 (Phase 3.1 では許容、spec §9.2 の貪欲対応の範囲)
- 部分木の平坦化 (`collect_subtree`) は経路文字列を親から組み立てるため、
  ノードラベルの書式変更はテストの期待文字列も連動して変わる
- 空モジュール (`MagiaGraph.modules` が空) は両側 None 扱いで空 diff —
  パニックさせない
- カウント系の `u32::try_from(..)` は `unwrap_or(u32::MAX)` でなく `expect` で落とす
  (プロジェクト全体の規約。Phase 1.3/1.4 に続き Phase 3.1 の metrics.rs でも再発した —
  syn-visitor-patterns.md にある規約だが magia-rust 固有文書なので見落としやすい。ここにも明記)

## 参照

- `crates/magia-core/src/diff.rs`
- `crates/magia-core/src/metrics.rs`
- `crates/magia-core/src/render/midchilda.rs` の `render_diff` / `write_overlay_diff` (Phase 3.2)
- `crates/magia-rust/tests/spell_diff.rs` (4象限テスト + overlay テスト)
- `project-docs/magia/spec-v0.3.md` §8 (highlight: changed) / §9.2 (差分エンジンの契約)
- SVG 側の決定論の落とし穴は `svg-deterministic-rendering.md` を参照
