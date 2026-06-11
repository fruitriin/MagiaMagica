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
- [x] M2-7. 素材送付 (Vue 版 vs inline 版の比較スクショ) → **M2 判定: 合格** (「同じに見える」)

**M3 (同サイクル続行、2026-06-11)**
- [x] M3-1. SourcePane (syntect HTML v-html) + FunctionToc (クリック → ?fn= push) + ペアビューレイアウト (TOC | ソース | 魔法陣)
- [x] M3-2. URL 同期: query watch → selectFunction の一方向ループ (URL が唯一の状態源)。戻る/進む対応 (push)、fallback 時の書き戻し (replace)
- [x] M3-3. エラー表示: 構文エラーバナー (message + 行番号) + last-good 保持 + 復旧の E2E 確認 (一時 fixture で実証)
- [x] M3-4. focus store の照合を qualified に修正 (impl メソッド `Caster::cast` が name 照合で fallback に落ちるバグ)、初回ロードを SSE 接続直後イベントに一本化 (M2 の二重フェッチ解消)
- [x] M3-5. @unocss/reset 追加 (preset-uno は preflight を含まない)
- [x] M3-6. ゲート: vp check + bun run build 通過 (Rust 側は今回無変更 — M2 で全通過済み)。E2E: TOC 切替 / URL 同期 / 戻る進む / SSE 新関数反映 / エラーバナー全て確認、クリーンロードでコンソールエラーなし (HMR 編集中限定の警告は knowhow に記録)
- [x] M3-7. 素材送付 (4 fixture のペアビュー 2x2) → M3 判定受領
- [x] M3-8. **判定対応**: ペイン並び替え (魔法陣を左端 + flex 1.6 で最大幅、コード中央、関数一覧右端)。別スコープ要望4点は計画化 — ①②③ (凡例 / 入口・回転サイン / 補助陣関数名) → phase4.0.6-circle-affordances.md 新規、④ (TOC ピンフィルタ / ツリー表示) → phase4.1 に追補。TODO 登録済み
- [x] M3-9. 並び替え対応済み。SVG 描画系アフォーダンス (入口サイン・補助陣ラベル) は **4.3 後で確定** (オーナー判定) — 4.0.6 計画書・TODO に反映済み

**M4 (同サイクル続行、2026-06-11)**
- [x] M4-1. palette store 拡張: レイヤー語彙を Rust 側 FilterSpec と同一に修正 (control_flow / effects / type_info)、setVisible/setOpacity/showAll/hideAll/setVisibleSet
- [x] M4-2. lib/magiaDsl.ts: parseDsl / exportDsl 純関数 (Vitest 対象として分離、M6)。エラーメッセージは inline 版と同文言 (行番号付き、hide カテゴリ禁止、未知レイヤー名)
- [x] M4-3. LayerPalette (式 radio + checkbox + opacity slider + 全表示/全非表示 + .magia details)。右カラム上部 (TOC の上) に配置
- [x] M4-4. TranscriptRegion (visually-hidden、role=region、spec §15 準拠の完全形)
- [x] M4-5. MagicCircleView にレイヤー適用 (g.layer-* へ display/opacity、位置不変 spec §5.4。v-html のため watch + nextTick — 4.0.7 で宣言化)
- [x] M4-6. useQuerySync: URL クエリ ↔ store 双方向同期 (?fn / ?style / ?layers / ?op、inline 版と完全互換形式)。HomeView の watch 2本を統合
- [x] M4-7. E2E 確認: toggle → ?layers= / slider → ?op= / ベルカ切替 → ?style= / リロード完全復元 / URL 直開きで状態再現 / DSL エクスポート→適用→カテゴリ注記→エラー行番号 / transcript region (visually-hidden + 内容)。vp check + build 通過
- [x] M4-8. 素材送付 → **M4 判定: 合格** (「めっちゃいいかんじ」+ パレット折りたたみ指示 → 対応済み、既定は閉)

**M5 (同サイクル続行、2026-06-11)**
- [x] M5-1. LayerPalette を折りたたみ式に (M4 判定対応。既定閉、開閉はローカル UI 状態で URL に載せない)
- [x] M5-2. rust-embed 統合: `#[derive(Embed)] folder="../../web/dist"`、GET / と静的ファイルを embedded_response で配信 (拡張子 → Content-Type 最小マップ)。**旧 inline HTML (INDEX_HTML 194行) 削除**
- [x] M5-3. build.rs: dist が src より新しければスキップ / 古ければ bun install + build / bun 不在は手順つき panic。rerun-if-changed は src のみ (dist を監視すると再実行ループ)
- [x] M5-4. CI: ci.yml 新設 (Bun 1.3.9 + vp check + bun build + clippy/fmt/test)、spell-diff.yml に setup-bun 追加 (build.rs が bun を要求)
- [x] M5-5. CLAUDE.repo.md に Bun 前提・開発フロー (二段構成 + rust-embed) 記載、scripts/dev-web.sh (並走 1コマンド)
- [x] M5-6. 統合テスト更新: inline HTML の assert → SPA shell (`<div id="app">` + asset JS 200 + 未知ファイル 404)。UI の振る舞い検証は M6 Playwright に移譲
- [x] M5-7. バイナリサイズ: strip + thin LTO 追加で 7.03 → **5.91MB**。**SPA 同梱の純増は +0.11MB** (M5 前 6.92MB — 5MB 超過は Phase 4.0 の syntect 由来と切り分け済み)。5MB 復帰には syntect のシンタックス定義削減が必要 → オーナー判定事項
- [x] M5-8. cargo install 実機確認: インストール済み `magia` バイナリ単体で SPA + /state + SSE 全動作。Stage 1 ゲート全通過
- [ ] M5-9. 素材送付 → **M5 判定待ち**
- [ ] M6: Vitest + Playwright + Stage 1 ゲート + 知見記録
- [ ] Stage 2 レビュー + 完了処理（4.1 以降の計画書へ Vue 前提を追補）
