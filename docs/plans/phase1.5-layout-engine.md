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

`magia-core/Cargo.toml`:
- `petgraph = "0.6"`
- `kurbo = "0.11"`

## 受け入れ基準

- [ ] 決定論性テストが通る (10回連続で同じ出力)
- [ ] MainRing 中央配置・極座標 AuxRing・SummonGlyph 配置の3段階が分離した関数として実装されている
- [ ] 交差最小化が **オプション** として ON/OFF できる (テスト容易性のため)
- [ ] `cargo test -p magia-core` 全通過
- [ ] `cargo clippy` 警告0

## 後続

- Phase 1.6 で `LayoutResult` を入力として SVG を生成する
