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

### Phase 4.0.5 — フロントエンド実行基盤 (docs/plans/phase4.0.5-frontend-foundation.md)

**▶ 再開 (2026-06-11)**: 計画書の訂正を読み直し済み — ①**Bun 正式採用** (ランタイム・PM とも。Node.js/pnpm 不採用)、②**M2 で `MagicCircleSchema` 型定義を先置き** (4.0.7/4.0.9 への布石)、③CI は Bun セットアップ。前回までの進捗: ツールチェーン実値確認済み (bun 1.3.9 / vite-plus 0.1.24)、M0 の API golden 2点を web/golden/phase2x/ に保存済み (未コミット)。M1 scaffold は未着手。

**複数サイクル前提**: M1〜M5 の各マイルストーン完了時に目視素材を送付し、**オーナー判定を待ってから次へ進む** (計画指定)。1サイクル = 判定待ちゲートまで。

**今サイクル (M0 + M1)**
- [x] M0-1. Phase 4.0 完了確認（cargo build/test 全通過、/state /spell 200、?fn=write_document 表示確認）
- [x] M0-2. Phase 2.x 機能の golden 取得（preview で目視確認: 既定表示 / レイヤー toggle / ベルカ切替 / DSL UI 全て動作。ファイル golden は決定論的テキストで web/golden/phase2x/ に保存: state.json / spell_render.json / spell_write_document.json (svg + svg_belka + source_html + transcript 含む) / index.html。Playwright golden は M6 で同シナリオを自動化）
- [x] M1-1. Vite+ scaffold（`web/`。vp create は vanilla-ts 生成のため Vite+ 統合を保って手で Vue 化。確認日 2026-06-11 を計画書に記録）
- [x] M1-2. 依存: vue / vue-router / pinia / unocss (+preset-uno, preset-attributify) / @vitejs/plugin-vue / vue-tsc。全て実値ピン
- [x] M1-3. 設定: vite.config.ts（UnoCSS + proxy /state /spell /events → 4747 + PORT 差し替え）、tsconfig strict、Oxlint consistent-type-definitions: type
- [x] M1-4. uno.config.ts の theme に palette.rs と同語彙の色名（effect 6色 / diff 3色 / belka 3色）
- [x] M1-5. 動作確認: vp dev 起動（autoPort で 52669）+ proxy 経由 /state 200 + コンソール無エラー + vp check / bun run build 通過
- [x] M1-6. 素材送付（スクショ + 構成サマリ）→ **M1 判定待ちでサイクル終了**（Stage 1 ゲート全通過: clippy / fmt / cargo test 16 スイート / ADDF テスト — web 追加分の cargo 影響なし）

**M1 判定 (2026-06-11): 合格** — ①色よさそう (後から差し替え容易な構成も評価)、②トーンよさそう、③ルーティングは「ファイルベースが基本の好みだが、クエリ軸の複雑性に対応するため明示的・プログラマブルな現行方式」で確定 (router/index.ts に設計判断を記録、複雑性が収まればファイルベース化リファクタも将来検討)。

**M2 (同サイクル続行、2026-06-11)**
- [x] M2-1. Pinia stores スケルトン: useFocusStore / useSourceStore / useConnectionStore / usePaletteStore (型 = type)
- [x] M2-2. api composable (composables/api.ts): fetchState / fetchSpell / connectEvents (SSE)
- [x] M2-3. MagicCircleView (v-html 過渡対応、4.0.7 で schema 化予定)
- [x] M2-4. 境界スキーマ MagicCircleSchema 型定義 (web/src/types/magia.ts) — circles / operations / edges / glyphs / signature / 配置済 (x,y) を意味論ベースで先置き
- [x] M2-5. ?fn= パラメータ受け取り (Vue Router → focus store 初期同期)
- [x] **【バグ修正】SSE が Phase 2.1 から配信されていなかった潜在バグを発見・修正** — tiny_http のチャンク経路は二重バッファ (Encoder 8KB + BufWriter 1KB) が flush されず、イベントが永遠に届かない。`request.into_writer()` + 自前ヘッダ + イベント毎 flush の `stream_sse` に置換。回帰テスト `sse_events_stream_immediately` 追加 (serve_integration.rs)。E2E でファイル変更 → Vue 再フェッチを確認
- [x] M2-6. Stage 1 ゲート (cargo 一式 + vp check + bun run build) 全通過
- [ ] M2-7. 素材送付 (Vue 版 vs inline 版の比較スクショ) → **M2 判定待ち**
- [ ] M3: ペアビュー UI（SourcePane / FunctionToc / SSE / エラー表示 / URL 同期）→ 判定待ち
- [ ] M4: Phase 2.x 機能の Vue 移植（LayerPalette / DslEditor / TranscriptRegion）→ 判定待ち
- [ ] M5: rust-embed 統合 + 旧 inline HTML 削除 + build.rs + CI + バイナリサイズ → 判定待ち
- [ ] M6: Vitest + Playwright + Stage 1 ゲート + 知見記録
- [ ] Stage 2 レビュー + 完了処理（4.1 以降の計画書へ Vue 前提を追補）
