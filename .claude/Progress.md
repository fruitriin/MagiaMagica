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

### Phase 4.3.7 — Spell Diff を Web に載せる (docs/plans/phase4.3.7-diff-on-web.md)

**設計確定 (着手時)**: 状態は focus store の diffRev (URL ?diff= と同期、replace)。fetchSpell が `&diff=<rev>` を付け、serve が gitio で before を取得して diff_spell_ir — **diff 時は ir 自体が viewBox 拡張済みの diff 版になる** (focus_layout もそれを入力にするため整合)。エラー・新規関数は diff_note (案内文) で会話を切らない。e2e は fixture dir を git init して live diff を本物の git で検証。

**実装フェーズ (Rust)**
- [x] 1. Shared に file: PathBuf 保持 + `?diff=<rev>` パース (percent_decode)
- [x] 2. render_spell の diff 文脈: gitio::show_file_at → parse → diff → diff_spell_ir。応答に diff_overlay / diff_report / diff_note (rev 不正・新規関数・git 外は note)
- [x] 3. serve 統合テスト (git fixture、正常/新規関数/不正rev/互換 — 一発通過) (git fixture: 正常 diff / 不正 rev 案内 / 新規関数案内 / ?diff なし完全互換)

**実装フェーズ (Vue)**
- [x] 4. focus store: diffRev + fetchSpell 配線 + useQuerySync (?diff=)
- [x] 5. MagicCircleView: overlay 配線
- [x] 6. パレット: diff 入力 (テキスト + HEAD~1/main プリセット + クリア) + diff_report/diff_note 表示 (テキストボックス + HEAD~1/main プリセット + クリア) + diff_report / diff_note 表示
- [x] 7. vitest 2本 (setDiffRev) + e2e 2本 (live diff: 保存→金ハロー出現を実 git で検証 / 不正 rev 案内)。serve-fixture.sh を git init 化 (git init fixture で live diff / 不正 rev 案内)

**仕上げ**
- [x] 8. Stage 1 全通過 (cargo 14 / clippy — render_spell 分割 excerpt_maps / vitest 41 / playwright 20)。素材 (serve.rs 自身の live diff) 送付 (自己ホスティング live diff) → 送付
- [ ] 9. Stage 2 レビュー + 指摘対応
- [ ] 10. 計画 memo、knowhow、Feedback / TODO 更新、アーカイブ、コミット
