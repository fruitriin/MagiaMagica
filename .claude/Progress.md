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

**追加要望 2 (オーナー、2026-06-12): インスペクタに呼び出し式 (レシーバ + 引数、改行込み) を表示**
- [x] B1. ring.rs source_span(): proc_macro2 の列情報を SourceSpan に埋める (0-based → 1-based、end は exclusive と規約明記)。式 span はレシーバ込み (`node.span()`)
- [x] B2. GlyphIr に source_span (SpanIr — 行・列とも揃ったときだけ Some)
- [x] B3. serve: call_excerpt (列クリップ + 継続行の共通インデント除去、文字単位で UTF-8 防御) + 応答に call_excerpts (glyph id → syntect HTML)。unit 3本
- [x] B4. CallInspector: 「呼び出し式」ブロックを解決の成否に関わらず表示 (解決済みは「定義」ラベルで併記)。inspectedCall に glyphIrId 追加
- [x] B5. e2e 更新 (解決済み `helper(sum)` / 外部 `format!("Hello, {name}")` の式表示)。playwright 15 通過
- [x] B6. Stage 1 ゲート全通過 (cargo 17 / ADDF / vp check / vitest 30 / build / playwright 15)
- [x] B7. 素材撮影 (.map のチェーン全体 sigil.layers...map(|role| role.kind) 改行込み / write_defs の呼び出し式 + 定義併記) → 送付
- [x] B8. Stage 2 レビュー (追加要望2分): Critical/High なし。対応 — W1 proc_macro2 の end().column が既に exclusive である旨をコメント明記 / W2 逆転列の回帰テスト + 空 excerpt はサーバ側で除外 / S3 SSE refresh で glyphIrId が旧 id を指す問題を修正 (同名一意なら付け替え・曖昧なら閉じる、vitest 3本)。S1 Clone/Copy derive は「将来のため」のみが根拠で見送り (POSD)、S2 凡例の irId: -1 は selectable: false で参照されず実害なし、S4 は clippy 通過済みで非該当。contribution 検出: 候補なし (ADDF 仕組みに非接触)
- [x] B9. ゲート再実行全通過 (cargo 17 / clippy / fmt / vp check / vitest 33 / build / playwright 15)

**追加要望 3 (オーナー、2026-06-12): ホバープレビュー化 + 操作ドットにも断片 + シグネチャのクリック判定除去**
- [x] C1. SignatureArc: pointer-events: none (円弧テキストが召喚印のクリックを奪う問題)
- [x] C2. Rust: OperationPayload.source_span + ring.rs の Operation 生成3経路に span (plain = 文全体 / 制御 = キーワード〜ガード式の source_span_between 合成 — Span::join が stable に無いため行・列で合成 / call は式全体)
- [x] C3. OperationIr に source_span、serve に op_excerpts (`<ring_id>-<出現順>` = Vue の Operation.irKey と同語彙 → syntect HTML)。call_excerpt は span_excerpt にリネーム (用途拡大)
- [x] C4. focus store: hoverExcerpt (読み専用プレビュー)。固定 (inspectedCall) と2層 — ホバーが上 (z-60 > z-50、オーナー指定)。selectFunction 成功時にクリア (図差し替えで mouseleave が来ない対策)
- [x] C5. HoverPreview コンポーネント (pointer-events: none — プレビュー自身がマウスを奪うチラつき防止) + GlyphDot / OperationDot の mouseenter 連動。glyph プレビューに「クリックで固定」ヒント (固定しないとピン操作できない表現)
- [x] C6. CallInspector: 薄幕廃止 → window click/Esc で閉じる (固定中も他ノードへホバーが届く)。glyph クリックとポップオーバー内クリックは stopPropagation
- [x] C7. テスト: serve 統合 (op_excerpts 契約 — syntect はトークン分割するため HTML 構造のみ照合、中身は span_excerpt unit 4本が担保)、e2e 3本追加 (op-dot 断片 / 固定+ホバー併存 / シグネチャ pointer-events)。Stage 1 全通過 (cargo 17 / clippy / fmt / ADDF / vp check / vitest 33 / build / playwright 18)。素材2点送付

**品質検証 + 完了処理**
- [ ] 14. Stage 2 レビュー + 指摘対応 (4.1 全体 — 最終判定が出たら実施)
- [ ] 15. 計画 memo、Feedback / TODO 更新、アーカイブ、コミット
