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

### Phase 4.0.7 — SVG → Vue 境界スキーマ (docs/plans/phase4.0.7-svg-to-vue-schema.md)

**前提訂正 (2026-06-11 着手時確認)**: 計画書の「data-* 属性で IR ノード id が埋まっている」は実装と不一致 (SVG に data-* は無い、意味論は class のみ)。**Rust 無変更で class + 出現順 id から復元**する方針 (パーサは 4.0.9 で捨てる前提のため Rust にパーサ専用属性を足さない。SigilId は外部契約に出さない Phase 3.2 方針とも整合)。復元できない要素 (ベルカ式含む) は raw 素通しで表示維持。

**実装フェーズ**
- [x] 1. MagicCircleSchema 型の拡張: SchemaEdge は座標ベース (from/to は 4.0.9 で埋まる optional)、RawElement (素通し)、layer 所属、**z (描画順 — 元 SVG の出現順で z-order を再現)**、Signature は円弧/直線 (ベルカ) 両対応
- [x] 2. svgToSchema.ts パーサ (converters/ 隔離) + Vitest 10本 (golden svg fixture、取り溢しゼロ検証含む)。happy-dom 環境追加 (test 設定で e2e/ が拾われる罠は exclude で対処)
- [x] 3. コンポーネントツリー: MagicCircle (z 順の単一描画リスト) / RingCircle / OperationDot / GlyphDot / EdgeLine / SignatureArc (id 衝突回避に useId) / RawFragment
- [x] 4. MagicCircleView の v-html 置換 (svgToSchema 参照はここ1箇所)。レイヤー適用は宣言的に — 既定値ではスタイルを付けない (opacity:1 明示は stacking context で AA が変わる)
- [x] 5. ホバー/選択: focus store に hoveredOperationId / selectedOperationId (出現順 id = セッション内一時、永続化しない)。シアン/金の輪郭強調 (Phase 3.2 ハロー原則 — 記号色に触れない)
- [x] 6. Playwright 9本通過 (toBeHidden の偽陽性を「toBeVisible → toggle → toBeHidden」の2段に強化、ホバー/選択テスト追加)
- [x] 7. 等価検証 (3 fixtures): 同版2回撮り 0 差のベースライン確認 → 版差は write_document 5328 / medium 3672 / dense 5568 px (0.3〜0.4%)、**fuzz 5% で 110〜155 px (0.01%) = AA ゆらぎのみ**。シフトテスト・最大クラスタ拡大で位置ズレ/形状差なしを確認 → **視覚的等価**。素材作成済み → 判定待ち
- [x] 8. Stage 1 ゲート全通過 (vp check / vitest 28 / build / playwright 9 / cargo 16 / ADDF — Rust 無変更) + 知見記録 (milestone-gated-ui-plan に画素等価検証の方法論、viteplus-bun にテスト環境の罠)

**品質検証 + 完了処理**
- [ ] 9. Stage 2 レビュー + コントリビューション検出 + 指摘対応
- [ ] 10. 計画 memo、Feedback / TODO 更新、アーカイブ、コミット
