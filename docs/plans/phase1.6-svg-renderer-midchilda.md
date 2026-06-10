# Phase 1.6 — SVG レンダラ (ミッドチルダ式 ConcentricRings) (M6)

## 出典

- `project-docs/magia/spec-v0.1.md` §6.1.3 (記号体系・色相規約)
- `project-docs/magia/spec-v0.1.md` §6.1.5 (出力形式: `<g>` 分離と class 属性)
- `project-docs/magia/tech-selection-v0.1.md` §2.3 (kurbo), §2.4 (自前 SVG ビルダー), §2.6 (insta), §6 (ミッドチルダ式 ConcentricRings)
- `project-docs/magia/phase2plus-notes-v0.1.md` §7.2 (ミッドチルダ式 = Call Graph 可視化)

## 目的

`magia-core::render` モジュールに、`MagiaGraph` + `LayoutResult` を入力として SVG 文字列を生成するレンダラを実装する。Phase 1 のフラッグシップ式である**ミッドチルダ式 ConcentricRings バリアント**のみを実装する。`RenderStyle` enum は将来の式追加に開いた形で定義する。

## スコープ

### やること

- `magia-core::render::RenderStyle` enum を定義:
  - `MidchildaConcentric` (Phase 1 で実装)
  - `Belka`, `Yagami`, ... (variant 名のみ define、`unimplemented!()` で stub)
- 公開 API: `render(graph: &MagiaGraph, layout: &LayoutResult, style: RenderStyle) -> String`
- SVG 生成は `std::fmt::Write` ベースの自前ビルダーで実装 (`svg` クレートは不採用)
- レイヤーごとの `<g class="layer-control-flow">` / `<g class="layer-effects">` / `<g class="layer-type-info">` 分離 (spec §6.1.5)
- 記号体系 (spec §6.1.3) を ConcentricRings として実装:
  - MainRing: `<circle>` (色は黒)
  - AuxRing: MainRing の外側に配置された `<circle>` (制御構造種別ごとに微小な内部記号)
  - SummonGlyph: 小さな `<circle>` または `<polygon>`。色相は EffectSet に応じて:
    - pure → `#000000`
    - io → `#1f4dff`
    - network → `#7b3ff5`
    - db → `#1fa341`
    - filesystem → `#7a4a1c`
    - unsafe → `#d92626`
  - 制御構造の内部記号: 二股の合流点 / 円形補助リング内の矢印 / 早期リターン矢印
  - async fn → MainRing を二重線 `<circle>` (stroke 2本)
  - Result / Option 戻り値 → MainRing からの分岐線 (正常 / 異常)
  - 関数シグネチャ → MainRing 最外周に `<textPath>` で円弧ラベル
- Mystical 原典に従い**反時計回り**で配置 (3時起点)
- 属性順序を固定し、insta スナップショットが安定するようにする
- `kurbo::BezPath::to_svg()` で曲線パスを文字列化
- ゴールデンテスト (insta) — **2段構え**:
  - **(a) 合成 fixture** (最小単位の動作確認用):
    - `fixtures/simple_compute.rs` (純粋関数1つ)
    - `fixtures/if_branch.rs`
    - `fixtures/match_arms.rs`
    - `fixtures/async_io.rs`
    - `fixtures/unsafe_block.rs`
  - **(b) 自己ホスティング fixture** (オーナー確定: 「このリポジトリそのもののスナップショット」方針):
    - `magia-core` / `magia-rust` / `magia-cli` の代表関数 (例: `parse_function`, `layout`, `render`) を fixtures から参照し、自身のコードを描画する
    - これにより「魔法陣として綺麗か」をオーナーが目視で判定する素材を確保する (= ドッグフーディング)
    - M6 完了時点では (a) のみで insta スナップショットを確定し、(b) は M7 で `magia render` の手動実行用サンプルとして提供
  - 各 fixture から `parse_function → layout → render` の SVG を確定
- **意匠判定の方針**: ミッドチルダ式の細部 (装飾の濃さ、線の太さ、ルーン的記号の有無) は spec §6.1.3 の最小要件で実装し、オーナーが (b) の自己ホスティング fixture を目視して調整を入れる (Appendix A v0.2 の精密化は Phase 1.6 完了時の振り返りで判断)
- 出力 SVG はブラウザで開いて視覚確認可能であること (CLI は M7 だが手動 `cargo run --example render` で見られるようにする)

### やらないこと

- ベルカ式 / 夜天の書式 (Phase 3+, Phase 6+)
- 立体ビュー (Phase 6)
- 対話的レイヤー切替 (Phase 2)
- アニメーション / SVG `<animate>` 要素
- フォント埋め込み (デフォルトフォントを参照するに留める)

## 設計上の判断

- 色相規約は `magia-core::render::palette` モジュールに分離し、変更しやすくする
- 記号サイズや stroke 幅は `magia-core::layout::constants` と共有 (重複定義を避ける)
- SVG の root `<svg>` の `viewBox` は `LayoutResult.canvas` から導出
- `<g>` の class 名は `layer-<snake_case>` で固定 (Phase 2 の CSS 切替の前提)
- ConcentricRings 以外の variant は `unimplemented!("not implemented in Phase 1")` でよい
- insta スナップショットは `fixtures/snapshots/` に格納し、レビュー時は SVG diff を目視で確認

## 依存ライブラリの追加

`magia-core/Cargo.toml`:
- `kurbo = "0.11"` (M5 で既に追加済み)
- (dev) `insta = { version = "1", features = ["yaml"] }`

## 受け入れ基準

- [x] 5個の合成 fixture から SVG が出力され、insta スナップショットが確定する (+ レビュー対応で for_loop を追加し計6本)
- [x] 自己ホスティング fixture (`magia-core` 自身の関数) から SVG が出力できる (insta は任意、目視確認できればよい)
- [ ] オーナーが自己ホスティング fixture の SVG を目視し、合格判定または調整指示を出す — **SVG 4本を送付済み、判定待ち**
- [x] 同じ IR から2回 render して文字列が**完全一致**する (決定論性)
- [x] 各効果カテゴリで色が正しく出る
- [x] async fn が二重線で描画される
- [x] `<g class="layer-*">` が3つ存在し、それぞれに対応する要素が入っている
- [x] 出力 SVG をブラウザで開いて崩れない (xmllint で XML valid 確認、qlmanage の PNG でレイアウト破綻なしを確認)
- [x] `cargo clippy` 警告0

## 後続

- Phase 1.7 で CLI から `magia render <FILE> --fn <NAME>` で本レンダラを呼び出せるようにする

## 実装結果メモ (2026-06-11)

### 計画からの変更点

- **insta は magia-rust の dev-deps に追加** (計画の magia-core 案から変更): E2E ゴールデンテスト
  (parse_function → layout → render) は magia-rust が必要で、magia-core に置くと dev-dep 循環になる。
  IR 直組みの単体テストは magia-core 側 (`tests/render_svg.rs`)、fixture E2E は magia-rust 側
  (`tests/render_golden.rs`) に分離
- **自己ホスティングは `examples/render_self.rs`** (`cargo run -p magia-rust --example render_self`)。
  parse_function / collect_calls_in_stmt / layout_with / count_crossings の4関数を
  `target/self-hosting/*.svg` に出力

### 設計判断の確定

- レイヤー対応: control-flow = リング・接続線・制御記号 (async 二重線含む) /
  effects = Operation ドット (リング内周に反時計回り配置、spec §6.1.2) + 召喚記号 /
  type-info = シグネチャ円弧 + Result/Option 分岐線 (異常パスは破線)
- 色の優先順位 (EffectSet 複数フラグの潰し方): unsafe > network > db > filesystem > io > pure
- `kurbo::BezPath::to_svg()` の最短浮動小数表現はノイズを含むため `normalize_path_numbers`
  で2桁固定に後処理 (スナップショット安定性、レビュー W-1 対応)
- palette は `pub(crate)` (公開 API に色定数を露出させない。Phase 2 で必要になったら公開判断)

### レビュー対応 (Stage 2)

- 修正済み: kurbo パス数値の正規化 (W-1) / palette の pub(crate) 化 (W-2) /
  for_loop fixture 追加で sym-loop のカバレッジ空白を解消 (S-1) / コメント精緻化 (S-2〜S-4)
- 既知の見た目課題 (オーナー目視判定の材料):
  - AuxRing にぶら下がる glyph のファンが全周配置のため、親リング方向の glyph が
    接続線・リングと重なることがある (layout_with.svg で確認可能)。Phase 1.5 の
    交差最小化は線の交差のみ対象で、ノード重なりは未対応
  - 描画定数 (`layout/constants.rs`) は仮置き。調整指示があればここだけ変える

### 振り返り判断 (計画の宿題)

- 深さ制限 (Phase 1.3 の宿題): 自己ホスティング4関数では入れ子3段でも視覚破綻なし。
  深さ制限は導入しない (必要になったら再訪)
- 意匠の精密化 (Appendix A v0.2): オーナーの目視判定待ち。判定後に必要なら独立 Plan を起こす
