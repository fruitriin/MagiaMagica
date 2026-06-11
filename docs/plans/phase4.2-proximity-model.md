# Phase 4.2 — 近接度モデル (フォーカスからの距離を定義する)

## 出典

- オーナー要望 (2026-06-11): 「フォーカスから遠いほど小さく薄く」。何を「遠い/近い」と判定するかの基盤が必要
- 親計画: Phase 4.1 (ピン中心ビュー) のスタブ ProximityIndex を本実装に差し替える
- 流用: Phase 3.4 (データフロー IR) — 呼び出しエッジから近接度を引く

## 目的

「ある関数 (focus) から見て、他の関数がどれくらい近いか」を数値化する共通モジュールを作る。
動的UI (4.1 ピン中心ビュー)・静止画 (4.3 複合レンダ)・呼び出しジャンプ (4.4) など Phase 4 系列の複数計画で消費する基盤。

距離の意味論を一箇所に集約し、UI 側は「距離 → 描画属性 (scale / opacity / リング所属)」のマッピングだけ持つ設計に統一する。

## スコープ

### やること

- **`magia-core::proximity` モジュール**:
  - `Proximity` トレイト: `fn distance(&self, focus: FunctionId, other: FunctionId) -> Option<f32>`
  - 距離の単位 = 「ホップ数」相当の連続値。`None` = 到達不能 (本計画では同ファイル外)
- **デフォルト実装 `LocalProximity`**:
  - 同 impl/trait ブロック: 距離 0.5
  - 同ファイルだが impl 違い: 距離 1.0
  - 呼び出し関係 (focus が呼ぶ、または focus を呼ぶ): 距離 0.7
  - 呼び出し関係の2ホップ先: 距離 1.4
  - 同 impl ∧ 呼び出し関係 = 最小値 (近い方を採用)
- **`ProximityIndex` 生成**:
  - `LocalProximity::index_for(focus: FunctionId, all: &[FunctionEntry], dataflow: &DataflowIr) -> ProximityIndex`
  - 結果は `Vec<(FunctionId, f32)>` を距離昇順でソート済み
- **Phase 4.1 のスタブ差し替え**: トレイト経由で透過的に差し替わる
- **UI 側マッピング (4.1 と連動)**:
  - 距離 ≤ 0.6 → 内リング
  - 距離 ≤ 1.1 → 中リング
  - 距離 ≤ 1.5 → 外リング
  - それ以上 → 表示しない (リング外)
- **テスト**:
  - 同 impl 2 関数の距離 = 0.5
  - 同ファイル別 impl の距離 = 1.0
  - 直接呼び出しの距離 = 0.7
  - 2ホップ呼び出しの距離 = 1.4
  - 循環呼び出し (A → B → A) で無限ループしない
  - 同名関数 (impl 違い) は別 FunctionId として扱う

### やらないこと (別計画)

- 同ファイル外への到達 (Phase 4.5 ワークスペース俯瞰)
- 型レベルの「似ている関数」検出 (引数型・戻り値型が一致するもの → 兄弟扱い)。将来の意味的近接 (Phase 4.6+) で検討
- 重み学習や利用統計ベースの近接度 (out of scope)
- IDE 統合的な「実行履歴ベースのホット関数」(out of scope)

## 設計上の判断

### なぜ「ホップ数」連続値か、リング離散値ではないか

リング所属は UI の都合 (4.1)。モデル側は連続値で持つ:
- 将来 UI 側が「リング 3 段階」から「滑らかなスケール」に変わる可能性 (Phase 4.6 のテーマ拡張)
- 静止画レンダ (4.3) と動的 UI (4.1) で別のリング境界を採るかもしれない
- 連続値なら境界値を UI 側パラメータとして外出しできる (POSD 一般的インターフェース)

### 同 impl > 呼び出し関係 > 同ファイル の順で「近い」

直感:
- 同 impl/trait は「同じオブジェクトの別側面」= 最も意味的に近い
- 呼び出し関係は「動作上連動する」= 動的にも一緒に追いたい
- 同ファイルは「人間が一緒に置きたいと判断した」= 静的に近い
- 距離 0.5 / 0.7 / 1.0 はこの直感を素直に序列化したもの。実装時に fixture で違和感が出たら調整可能 (`Feedback.md` に判定 hook を入れる)

### 距離の合成は min

(A: 同 impl, B: 同 impl の関数を呼ぶ) → A,B の距離は min(0.5, 0.7) = 0.5。
複数経路で近いほど近い。これは「2人の友達がいる人は1人だけの人より近く感じる」の素朴な拡張ではないが、画面上で「内リングに入れたい関数を取りこぼさない」ためにこれを採る。

### 呼び出し関係の取得

Phase 3.4 のデータフロー IR を流用:
- `magia-core::dataflow::Ir` から call_edges を抽出
- 同一ファイル内に閉じたものだけ採用 (本計画のスコープ)
- 静的解析で取れない動的ディスパッチ (trait method call) は best effort

### 呼び出しは方向性を持つ (caller/callee) が、近接度では無向

UI 上は「focus から見て近い関数を周辺に並べる」だけで、矢印の向きは別レイヤー (4.4 で扱う)。
近接度の計算では caller/callee を区別せず最短ホップを採る。

## 受け入れ基準

- [ ] `LocalProximity::index_for(...)` が距離昇順の `Vec<(FunctionId, f32)>` を返す
- [ ] テスト fixture (impl 2つ + 呼び出し関係2本) で期待値と一致
- [ ] 循環呼び出し fixture で無限ループせず正しく距離を返す
- [ ] Phase 4.1 のスタブが本実装に差し替わり、既存の Phase 4.1 テストが通る
- [ ] 距離計算は O(N log N) 程度に収まる (BFS + sort)
- [ ] `cargo test --workspace` / clippy / fmt 通過

## 後続候補

- Phase 4.5 で同ファイル外への到達 (モジュール階層を距離に組み込む)
- Phase 4.6 で型レベル類似 (return type / argument shape) を距離成分に追加
- 利用統計 (どの関数からどの関数に飛ぶか) を将来 telemetry として追加

## 実装ステップ

**実装フェーズ**

1. **magia-core**: `proximity` モジュール + `Proximity` トレイト + `LocalProximity` 構造体
2. **magia-core**: 同 impl 判定 (FunctionEntry.impl_context を比較)
3. **magia-core**: 同ファイル判定 (FunctionEntry.file_path を比較)
4. **magia-core**: 呼び出し関係 BFS — Phase 3.4 DataflowIr から call_edges を引いてグラフ化、距離 0.7 / 2ホップ 1.4
5. **magia-core**: 距離合成 (min)
6. **magia-core**: `ProximityIndex::sort_by_distance` (距離昇順 + 同距離は名前順)
7. **Phase 4.1 差し替え**: スタブを除去し本実装の `LocalProximity::index_for` を呼ぶ
8. **テスト**: 同 impl / 同ファイル / 直接呼び出し / 2ホップ / 循環 / 同名関数別 impl
9. **既存テスト**: Phase 4.1 の golden が新インデックスで通るよう更新 (距離変化で並び順が変わる場合の golden 差分はレビュー対象)
10. **Stage 1 品質ゲート** + コーディング知見記録 (近接度モデルは他プロジェクトでも有用な可能性 → 抽象化粒度を慎重に)

**品質検証フェーズ (Stage 2)**

11. レビュー + コントリビューション検出 + 指摘対応 → ゲート再実行

**完了処理**

12. 計画 memo、Feedback / TODO 更新、アーカイブ、コミット

## 想定リスク

- **距離係数 (0.5 / 0.7 / 1.0) のチューニング**: fixture で違和感が出たら調整。係数は `proximity::WEIGHTS` 定数に集約しておく
- **Phase 3.4 IR の API 変動**: dataflow IR の call_edge 取得 API が変わると本計画も追従が必要。`Ir::call_edges()` 等の最小インターフェースに依存を絞る
- **同名関数の区別**: `Foo::bar` と `Bar::bar` は別 FunctionId。`?pin=` URL では `impl_context::name` 形式 (Phase 4.0 で決定済)
