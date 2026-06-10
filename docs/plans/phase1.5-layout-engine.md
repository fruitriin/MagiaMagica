# Phase 1.5 — レイアウトエンジン (M5)

## 出典

- `project-docs/magia/spec-v0.1.md` §6.1.4 (安定レイアウト要件)
- `project-docs/magia/tech-selection-v0.1.md` §2.2 (petgraph), §2.3 (kurbo), §2.7 (乱数不使用), §3 M5

## 目的

`magia-core::layout` モジュールに、IR (`MagiaGraph`) を入力として各 `Sigil` の2次元座標 (`f64, f64`) を決定論的に算出するレイアウトエンジンを実装する。レンダリング (SVG 生成) は M6 で別実装する。

## スコープ

### やること

- 公開 API: `layout(graph: &MagiaGraph) -> LayoutResult`
  - `LayoutResult { positions: HashMap<SigilId, Point>, canvas: Rect }`
- spec §6.1.4 の優先順位で位置決定:
  1. **MainRing を画面中央 `(0, 0)` に固定**
  2. **AuxRing を MainRing 上の制御フロー位置に基づき極座標で配置**
     - MainRing の3時を起点 (Mystical 原典に従う) に反時計回り
     - Operation 列上の位置 (0..N) から角度を均等割
     - 半径は MainRing 半径 + 一定 (Phase 1 は定数でよい)
  3. **SummonGlyph を最も関連の深いリングから一定距離の位置に配置**
     - 親 Sigil の中心から外向きにオフセット
     - 同一親に複数 SummonGlyph がある場合は親の中心を通る放射状に等間隔配置
  4. **線の交差を最小化する局所最適化を最後に適用**
     - Phase 1 は単純な hill-climbing (隣接 SummonGlyph の角度入れ替え) でよい
     - 探索回数は固定上限 (例: 50回)、初期配置は決定論的
- すべての計算は `f64` で `kurbo::Point` / `kurbo::Vec2` を使う
- `SigilId` でタイブレーク。乱数は一切使わない
- `petgraph::Graph` を内部で構築して隣接情報を扱う (IR 自体は petgraph に依存させない)
- ユニットテスト:
  - 同じ IR から2回呼んで `positions` が完全一致 (決定論性)
  - MainRing が `(0, 0)` にある
  - AuxRing が MainRing から想定半径上に位置する
  - SummonGlyph 0個 / 1個 / 5個での配置が壊れない
  - 入れ子の AuxRing (M3 の成果物) でも位置が決まる

### やらないこと

- 力学モデル (force-directed) — Phase 2 以降
- 多関数同時レイアウト (Phase 1 は関数1つ = SVG 1枚)
- 立体配置 (Phase 6)
- レイヤーごとの位置切替 — Phase 2 の制約 (位置共有) はそもそも単一レイアウトしか持たないので自動的に満たされる

## 設計上の判断

- 半径・オフセットの定数は `magia-core::layout::constants` にまとめる (M6 で SVG 描画時に同じ値を参照する)
- 局所最適化は **明示的に決定論的アルゴリズム** (順序固定の貪欲法) を採用。`Vec` の `sort_by` で安定ソート
- `LayoutResult.canvas` は全 Sigil を包含する bounding box にマージンを足したもの
- `magia-core` 内に閉じる: `magia-rust` には依存しない

## 依存ライブラリの追加

`magia-core/Cargo.toml` (実装時の実値。計画当初の 0.6 / 0.11 から世代更新):
- `petgraph = "0.8"` (実値 0.8.3)
- `kurbo = "0.13"` (実値 0.13.1)

## 受け入れ基準

- [x] 決定論性テストが通る (10回連続で同じ出力)
- [x] MainRing 中央配置・極座標 AuxRing・SummonGlyph 配置の3段階が分離した関数として実装されている
- [x] 交差最小化が **オプション** として ON/OFF できる (テスト容易性のため)
- [x] `cargo test -p magia-core` 全通過
- [x] `cargo clippy` 警告0

## 後続

- Phase 1.6 で `LayoutResult` を入力として SVG を生成する

## 実装結果メモ (2026-06-11)

### 計画からの変更点

- **`LayoutResult.positions` は `HashMap` でなく `BTreeMap`**: M6 レンダラが走査して
  SVG 要素を出力するとき、HashMap だと出力順が実行ごとに揺れてスナップショットテストが
  壊れるため。決定論要件 (spec §6.1.4) の自然な帰結として変更
- **交差最小化は「隣接 SummonGlyph の角度入れ替え」でなく「ファン全体の回転」**:
  接続線が全て「親中心 → 子中心」の放射線のため、同一親の glyph をスロット間で
  入れ替えても線分集合の幾何が一切変わらず、計画のアルゴリズムでは交差数が動かない。
  回転角の貪欲 hill-climbing (探索順・ステップ・パス上限固定) に置き換えた

### 設計判断の確定

- 角度系: 数学系 (+x 右、+y 上、反時計回り正)。3時起点は outward=0。SVG の y 反転は
  M6 の責務 (素朴な反転は時計回りに見える点を mod.rs の doc に明記済み)
- AuxRing 角度 = 親 outward + anchor/content_len の全周均等割 + ordinal × 扇状ステップ。
  入れ子は BFS で外側に伸びる傾向
- SummonGlyph は親ごとの GlyphFan (全周等間隔)。1個なら outward 方向。交差回避で
  ファンごと回転しうる (1個ファンも対象、意図された挙動)
- 防御規約: `positions` は常に全 Sigil を覆う (MainRing 欠落・到達不能は原点フォールバック)

### レビュー対応 (Stage 2)

- 修正済み: positions/radii の2本 Map 管理を `PlacedSigil { center, radius }` の1本に統合
  (不整合状態を構造で排除) / `usize_to_f64` の expect panic 経路を `as f64` + 許容 lint に変更
  (カウントは 2^53 未満) / 交差判定の端点同一視を機械イプシロン → 1e-9 に変更 /
  `place_aux_rings` に root を明示引数化 / 1-glyph ファンの回転挙動と y 反転の注意を doc 化
- 先送り (Phase 1.6 の目視検証で再評価):
  - 回転ステップ 0.2 rad の粒度 (局所最小で十分かは見た目で判断、定数は pub なので調整容易)
  - AuxRing と SummonGlyph の重なり回避は未実装 (Phase 1 スコープ外、目視で問題なら対応)
