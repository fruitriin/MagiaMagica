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

### Phase 4.3 — 静止画レンダ Vue SSR + Bun 一本化 (docs/plans/phase4.3-composite-still-render.md)

**設計確定 (着手時)**: 計画書どおり配布形態は案 A (`magia-render` 独立バイナリ、bun build --compile)。マイルストーン分割で進める — M1/M2 は機能 (Stage 1 ゲートのみ)、M3 (ベルカ Vue 移植) / M4 (Spell Diff Vue 移植) は意匠判定ゲート。Rust SVG レンダラ削除 [break] は M3/M4 の等価判定合格後 (M5/M6)。

**M1 — SSR 基盤の技術検証 + 確立**
- [x] 1. spike 成功: `vue/server-renderer` で `<MagicCircle>` が SVG 文字列になるか + `bun build --compile` が single-file executable を吐くか (想定リスクの本丸を先に踏む)
- [x] 2. web/src/render/ssr.ts (renderSpellSvg + toStandaloneSvg 正規化 — viewbox 小文字化 / hydration コメント / 値なし data-v (XML 無効) / 空 style を除去) — stdin で IR JSON (`{ir, style, focus_layout?}`) → irToSchema → renderToString → stdout に SVG。エラーは stderr + 非0 exit
- [x] 3. build:render スクリプト + build.rs 統合 (target/magia-render、59MB、ウォーム 30ms — 基準 200ms クリア。50MB 基準は 9MB 超過 → サイズ方針で許容) (ローカル darwin-arm64) + 起動時間計測 (目標 200ms 以下)
- [x] 4. vitest 5本 (XML validity / 要素数一致 / 決定論 / 正規化単体)。完全 DOM 等価は M5 の golden 切替で確認: serve の動的 UI が描く SVG と SSR 出力の DOM 等価テスト (vitest)

**M2 — Rust 統合 + magia render CLI (ミッドチルダ経路)**
- [x] 5. ssr.rs: magia-render spawn (パス解決 4段: env → 同dir → 親dir → PATH)、stderr 伝達 (パス解決: MAGIA_RENDER_PATH → 同 dir → PATH)、stdin/stdout 配管、エラー伝達
- [x] 6. magia render の基本経路 (midchilda + フィルタなし) を SSR に切替。62ms。IR の nz() を2桁丸めに拡張 (sin/cos の 1e-15 ノイズが SVG に漏れていた) (--style midchilda のみ。belka は M3 まで Rust 経路温存)
- [x] 7. 統合テスト (XML 契約) + Stage 1 全通過 (cargo 17 / clippy / fmt / ADDF / vp check / vitest 38 / playwright 18)

**M3 — ベルカ式 Vue 移植 (意匠判定ゲート)**
- [x] 8. belka.rs に belka_ir() (射影 project / 配置 place_poles を再利用、SVG 文字列化なし) + BelkaIr 型 (pole 語彙 genesis/transmute/consume — 色・ラベルは Vue 側テーブル)。矢じりは tip 座標だけ IR に載せ羽の形は Vue 計算 (belka.rs の射影モデルを IR 化) + Vue コンポーネント (BelkaCircle ツリー)
- [x] 9. serve に belka_ir 追加、MagicCircleView を BelkaCircle に切替 (svg_belka は比較用に温存 — M5 で削除)。SSR 対応は M5 の --style belka 移行で
- [x] 10. 等価素材 (loop_accumulate 新旧) 送付済み — 判定待ち

**M4 — Spell Diff Vue 移植 (意匠判定ゲート)**
- [x] 11. ir_export::diff_spell_ir (after 基準 SpellIr + viewBox ゴースト拡張 + DiffMarkIr 列。配置・半径は Rust / 色・破線は Vue) + MagicCircle の overlay prop + ssr.ts の diff_overlay + magia diff --svg の基本経路を SSR 化 (フィルタ付きは M5 まで Rust 温存)。SSR 出力の数値2桁丸め (Vue 計算のエッジ端点ノイズ対応) を toStandaloneSvg に追加 (diff_status: added/removed/modified + ゴースト座標) + Vue overlay (金ハロー/シアン/灰破線)
- [x] 12. 等価素材 (process_order 新旧並置 — viewBox 拡張まで一致) 送付済み — 判定待ち。既存 diff_svg_writes_overlay_channel が SSR 経路の契約テストとしてそのまま通過

**M5 — 経路統一 + Rust SVG レンダラ削除 [break]**
- [ ] 13. magia diff / magia ci を SSR 経路に書き換え、golden 更新
- [ ] 14. midchilda.rs / belka.rs の SVG 出力関数を削除 (レイアウト計算・IR 加工は残す)。色定数の Rust 残置を grep 確認
- [ ] 15. CI: setup-bun + bun build --compile + 統合テスト

**M6 — ドキュメント + 完了処理**
- [ ] 16. CLAUDE.repo.md (Bun 前提・magia-render 配布・ビルド手順) 更新
- [ ] 17. Stage 1 全ゲート + Stage 2 レビュー + 指摘対応
- [ ] 18. 計画 memo、knowhow、Feedback / TODO 更新、アーカイブ、コミット
