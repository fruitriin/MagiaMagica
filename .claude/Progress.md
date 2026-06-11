# 進捗表

## 運用ルール

### タスク開始時
1. `.claude/Feedback.md` を読み、前回の改善アクションで未対応のものがあれば考慮する
2. 以下の手順で Markdown チェックリストを作成する
   1. 1ショットで作業できる範囲にサブタスクを分割する
   2. 並行作業できる粒度でさらに分割する
   3. 各サブタスクにテスト作成・統合テスト・Lint・ビルドが必要か検討し、必要なら追加する
   4. 必要に応じて 2.1〜2.3 を再帰的に適用する

### 作業中
3. サブタスク着手時に `- [x]` でチェックしていく。並列可能なタスクはコンテナオーケストレーションを利用する
4. 実装フェーズの最終サブタスク完了時、以下の知見を `/addf-knowhow` で記録する（既存 knowhow の更新も含む）:
   - **コーディング知見**: 実装中に発見した再利用可能なパターン、落とし穴、技術的判断とその根拠

### エージェント起動時の共通ルール
- エージェントチーム（TeamCreate）やサブエージェント（Agent）を作成するとき、各エージェントへのプロンプトに **最初に `/addf-knowhow-index` を実行する** よう指示を含めること
- これにより各エージェントがプロジェクトの知見ベースを把握した状態で作業を開始できる

### タスク完了時 — 品質検証

4. プロジェクトのビルド・Lint・テストコマンドを実行する
   - **失敗した場合 → 実装に差し戻す**。原因分析 → 修正 → 再実行
5. `addf-code-review-agent` でコードレビューを実施する
6. `addf-contribution-agent` で ADD フレームワークへのコントリビューション候補を検出する
7. レビュー指摘への対応:
   - **Critical/High**: 必ずこのフェーズ内で修正する（先送り禁止）
   - **Medium**: 原則修正。先送りする場合は独立計画を起こす
   - **Low/Info**: Plan に記録し、必要に応じて独立計画で対応
   - **バグ分離**: 発見されたバグが現在のプランと関心事が異なる場合は、修正せずに新しいプラン（`docs/plans/`）を書き起こし、`TODO.md` に追加するのみで現在のプランを完了させる
   - 修正後、ビルド・Lint・テストを再実行して通過を確認する
8. 品質ゲートで得た知見を `/addf-knowhow` で記録する:
   - **品質ゲート知見**: レビューエージェントが検出したパターン（セキュリティ、コード品質、分離パターン違反等）のうち、他のタスクでも再発しうるもの

#### ノウハウ蓄積

9. 投入されたタスクのPlanに実装完了状況を反映する
10. タスク全体の総括知見を `/addf-knowhow` で記録する:
    - **タスク総括**: 計画と実装のギャップ、想定外だった点、次回同種タスクへの教訓。コーディング・品質ゲートで既に記録した知見と重複しないこと

#### フィードバック記録

11. `.claude/Feedback.md` にPlan, TODO, Progress推進エンジンの問題の記録・改善アクションを追記する。反映済みの項目は削除する
12. `.claude/Feedback.md` にプロジェクト進行上の問題の記録・改善アクションを追記する。反映済みの項目は削除する
13. Progress 推進エンジン自体に関するフィードバック・ノウハウがあれば、テンプレート（`.claude/templates/ProgressTemplate.md`）の改善案を `.claude/Feedback.md` に記録する

#### アーカイブとコミット

14. `.claude/Progresses/YYYY-MM-DD-プラン名.md` にリネームして移動し、`.claude/templates/ProgressTemplate.md` から新規の Progress.md を作成する
15. コミットする

---

## タスク

### Phase 4.1 — ピン中心ビュー (docs/plans/phase4.1-pinned-focus-view.md)

**設計確定 (着手時)**: 4.0.9 世界に揃える — Rust が focus_layout (周辺の x/y/scale/opacity) を計算して IR に載せ、Vue は `<g transform>` の CSS transition でピン遷移。スタブ近接度は trait でなく関数 (1実装の段階で trait は POSD 的に過剰 — 4.2 で中身を差し替え)。周辺の縮小3段階は スタブでは距離 1/2 のみのため「チップ (円 + 名前 + 操作概要)」1種 + scale/opacity 差で開始。

**実装フェーズ (Rust)**
- [x] 1. magia-core::proximity: スタブ分類 + ユニットテスト3本 (trait でなく関数 — 1実装の段階の抽象化は POSD 的に過剰、4.2 で中身差し替え)
- [x] 2. ir_export::focus_layout: 中央 viewBox + リング配置 (12時起点・等角度・決定論)、view_box 拡張。チップ概要 (op_count) は見送り — 全関数パースのコストが要るため名前+シグネチャのみ (4.2 で再検討)
- [x] 3. serve: /spell/<fn>?with=neighbors (なしは従来互換)
- [x] 4. serve 統合テスト (focus_layout 構造 / 距離 / 既定互換)

**実装フェーズ (Vue)**
- [x] 5. ?fn → ?pin リネーム (フォールバックなし)
- [x] 6. NeighborChip (円 + 名前 + signature ツールチップ、button 化) + MagicCircleView をピンビュー化 (単一リスト + 同一 key の g)
- [x] 7. ピン遷移: style.transform の CSS transition 320ms (SVG の transform 属性は transition 不可 — knowhow 化) + reduced-motion で即時
- [x] 8. FunctionToc: 表示中のみフィルタ (既定 ON、距離順シグネチャ) + ツリートグル骨格 (無効表示)
- [x] 9. F キー (入力中ガード付き) + チップの Tab/Enter はブラウザ標準
- [x] 10. エラー時は last-good 機構がそのまま機能 (focus_layout も last-good から配信)
- [x] 11. Playwright 13本 (ピン遷移 + 履歴 / reduced-motion の transition 0s / TOC フィルタ切替 追加)、Vitest 30 通過
- [x] 12. 素材 (write_document 28チップ / Wand::cast 距離1・2混在) 作成。**実機ではオーナーが既にチップクリックでピン遷移を試遊済み**
- [x] 13. Stage 1 ゲート全通過 (cargo 17 / ADDF / vp check / vitest 30 / build / playwright 13)

**追加要望 (オーナー、4.1 サイクル中 2026-06-11): 召喚印インスペクタ — Phase 4.4 の前倒し**
- [x] A1. GlyphIr に call_target (IR の OperationPayload.call_target を露出 — Rust 変更はこれだけ)
- [x] A2. focus store: inspectedCall + resolveCall (呼び出し名 → 同ファイル関数の解決はクライアントで関数一覧と照合。`.method` / `macro!` の正規化付き)
- [x] A3. CallInspector (Teleport ポップオーバー): 呼び出し名 + 解決先のコード断片 (既存 /spell API の source_html、縦可変 max-h-96) + コードクリック or ボタンでピン遷移。未解決は「外部呼び出し」案内。外側クリック / Esc で閉じる
- [x] A4. GlyphDot クリックで起動 (callTarget があるときだけ cursor: pointer)
- [x] A5. e2e 2本 (解決 → コード表示 → ピン遷移 / 外部呼び出し案内)。SSE refresh がインスペクタを閉じる誤挙動を修正 (名前ベースなので再採番の影響なし → クリアしない)、ピンビュー svg に .pin-view クラス (テストセレクタ安定化)
- [x] A6. ゲート再実行全通過 (playwright 15)

**品質検証 + 完了処理**
- [ ] 14. Stage 2 レビュー + 指摘対応
- [ ] 15. 計画 memo、Feedback / TODO 更新、アーカイブ、コミット
