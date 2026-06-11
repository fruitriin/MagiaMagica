# Process Feedback

開発プロセスの振り返りと改善を記録する。

## 記録方法

タスク完了時や問題発生時に、以下のいずれかのセクションに追記する。

## オーナーフィードバック

- **報告 (Phase 4.0.6 前半完了, 2026-06-11)**: シンボル凡例パネルが判定合格で完了しました (配置は判定どおり魔法陣ペインの下・横並び3カラム)。記号サンプルは実描画コンポーネントの再利用なので、意匠を調整すると凡例も自動追従します。後半 (入口・回転方向サイン / 補助陣ラベル) は計画どおり 4.3 後です。**次は 4.1 (ピン中心ビュー)**
- **報告 (Phase 4.0.9 完了, 2026-06-11)**: IR JSON エクスポート + Vue IR ビルダ (案2) が判定合格で完了しました (「左右同じに見える」)。**「Vue 1本化」コア系譜 (4.0.5 → 4.0.7 → 4.0.9) が完成** — SVG パーサは削除され、魔法陣は Rust の配置済み IR (spec v0.3 §16) を Vue が描画します。ベルカ式は判定どおり SVG 温存 + 保守方針コメント済み (4.3 でリメイク)。**4.1 (ピン中心ビュー) と 4.3 (Vue SSR) の前提が揃いました**。次タスクは 4.0.6 前半 (凡例パネル — 先行可) か 4.1 です
- **報告 (Phase 4.0.7 完了, 2026-06-11)**: SVG → 境界スキーマ + Vue コンポーネントツリー (案1) が判定合格で完了しました (等価性「同じに見える」、ホバー/選択「いいね」)。v-html は撤去され、魔法陣は `MagicCircleSchema` を受ける `<MagicCircle>` ツリーで描画されます。SVG パーサは converters/ に隔離済みで、**次の 4.0.9 (案2: IR JSON エクスポート + Vue ビルダ) ではこのファイルだけ差し替えれば移行完了**します。計画書の前提 (data-* 属性) が実装と不一致だったため class + 出現順 id 方式に変更した点は計画書の実装結果メモに記録済みです
- **報告 (Phase 4.0.5 完了, 2026-06-11)**: フロントエンド実行基盤 (Vue 3 + Vite+ + UnoCSS + Bun) が**全マイルストーン判定合格で完了**しました。`magia` バイナリ単体で Vue UI 全機能が動く配布形態です (旧 inline HTML 削除済み、CI も Bun 化、Vitest 18本 + Playwright E2E 8本)。判定での決定事項は反映済み: ルーティングは明示的クエリベース / ペイン並びは魔法陣最優先 / パレット折りたたみ / バイナリサイズ基準撤廃 (CLAUDE.repo.md に方針化) / SVG 描画系アフォーダンスは 4.3 後。**次は 4.0.7 (案1: SVG → 境界スキーマ + Vue コンポーネント)** です。リリースを行う場合は `/addf-release` を実行してください
- **報告 (SSE 潜在バグ修正, 2026-06-11)**: Phase 2.1 から SSE (`/events`) がクライアントに配信されていなかった潜在バグを M2 の E2E 確認中に発見・修正しました (tiny_http チャンク経路の二重バッファ問題)。live-reload は実際には動いていませんでした。修正後は E2E でファイル保存 → Vue 自動再描画を確認済み、回帰テストも追加済みです
- **報告 (Phase 3 完了, 2026-06-11)**: Phase 3 (3.0〜3.5) が全完了しました — Spell Diff (構造差分 + 視覚強調)、CI 統合 (PR 自動コメント + unsafe チェック)、データフロー IR、ベルカ式 (三角力場)。リリース (バージョン採番・チェンジログ) を行う場合は `/addf-release` を実行してください。次サイクルからはオーナー要望の Phase 4.0 (ソース連動ビュー) に着手します
- **意匠判定依頼 (Phase 3.5, 2026-06-11)**: ベルカ式の素材3点を送付済み — ①loop_accumulate ②自己ホスティング measure ③Reducer 形。色 (空色/琥珀/臙脂)・三角の歪み・力場の濃淡・「Reducer 形で生成極が空円」の見せ方が判定ポイント。調整は `render/belka.rs` 冒頭の定数群と `palette.rs` の `BELKA_*` で対応
- **意匠判定依頼 (Phase 3.2, 2026-06-11)**: 視覚的 Spell Diff の素材2点を送付済み — ①合成 fixture (process_order)、②自己ホスティング実例 (Phase 3.1 の metrics_sentence リファクタ実 diff)。意匠規約: 金ハロー=追加 / シアンハロー=変更 / 灰破線ゴースト=削除。色・線幅・破線パターンの調整指示があれば `render/palette.rs` の `DIFF_*` と `layout/constants.rs` の `DIFF_HALO_*` で対応
- **意匠判定 (Phase 1.8 で対応済み・再判定待ち)**: Phase 1.6 の判定で「write_document 級はお洒落・write_control_flow 級は破綻」と確定 → Phase 1.8 で衝突回避を実装。**write_document の見た目はほぼ不変**（重なっていた青記号1個が 1.6px 退避したのみ、8px 閾値の回帰テストで固定）、**write_control_flow はリング重なり解消**。再判定素材3本（write_control_flow / write_document / dense_dispatch）を送付済み。次の調整指示があれば `layout/constants.rs` の定数または新規計画で対応
- **報告 (Phase 1.7)**: Phase 1 (M1〜M7) が完了し、`cargo install --path crates/magia-cli` で配布可能な状態です。リリース（バージョン採番・チェンジログ）を行う場合は `/addf-release` を実行してください
- **確認依頼 (Phase 3 計画立案 2026-06-11)**: notes §12 のロードマップから Phase 3 計画 6 本 (3.0 仕様 v0.3 / 3.1 差分エンジン / 3.2 視覚 Spell Diff / 3.3 CI 統合 / 3.4 データフロー IR / 3.5 ベルカ式) を起こし、暫定優先度で TODO に登録しました。**優先度の変更・計画への注文があればお知らせください**。「ベルカ式を先に見たい」なら 3.4 → 3.5 を先頭にもできます (Spell Diff 系と独立)。リリース (/addf-release) の指示も引き続き受け付けています
- **確認依頼 (計画立案 2026-06-11)**: Phase 1.8 + Phase 2.0〜2.4 の計画 6 本を起こしました。暫定優先度（1.8 レイアウト衝突回避を先頭）で TODO に登録済み。**優先度の変更や計画への注文があればお知らせください**。何もなければ次サイクルから 1.8 に着手します

## 問題の記録

- **knowhow のアップストリーム昇格は中止 (オーナー判定 2026-06-11)**: 一括昇格 PR (#13 Rust 系7本 / #14 Web 系3本) を作成したが、**「このノウハウは MagiaMagica 固有文脈なので ADDF アップストリームに入れない」**との判断で両 PR とも close (オーナー操作)。knowhow は固有文脈 (魔法陣の語彙・spec 参照・実コード断片) とセットで価値があり、汎用化すると薄まる。**今後の方針**: docs/knowhow/ はプロジェクト内に蓄積し続け、昇格はしない。コントリビューション検出 (addf-contribution-agent) は「knowhow の昇格候補」ではなく **ADDF の仕組みそのもの (スキル・テンプレート・lint・scaffold) への改善提案**に絞る。仕組み系の残候補は維持: addf-gui-test.md の .gitignore テンプレート (Phase 1.0)、launch.json.example の scaffold 化 (Phase 2.1)

- Phase 4.0.5 M2 で発覚: Phase 2.1 の SSE は統合テストが「HTML に EventSource の文字列が含まれる」という静的チェックだけだったため、**配信自体が壊れていることを2フェーズ以上見逃した**。ストリーミング系の機能は「実際にストリームを読むテスト」を受け入れ基準に含めること (回帰テスト `sse_events_stream_immediately` が定型、knowhow 訂正済み)。また「ブラウザで見た目が動く」確認は live-reload の動作確認の代わりにならない (初回ロードだけで画面は出る)
- Phase 1.0 で `.claude/skills/addf-gui-test.md` が `.gitignore` に手書きで追加されていた。これは ADDF 側で初期化時に挿入されるテンプレートに含めるべきもの。ADDF 本体への PR 候補
- Phase 3.3 で先送りした項目: spec §9.1「メトリクス変化のテーブル併記」は PR コメント内のテキスト行で代替中。Markdown テーブル化は運用フィードバック (Phase 3 振り返り) で判断
- Phase 4.0.9 のレビューで先送りした項目: EffectCategory 増加時の TS 側 (型 + COLOR_BY_EFFECT) 追従は手動 (型網羅で missing は検出される)。RawElement 型は IR 直結では常に空 — 4.3 のリメイク時に型ごと削除を判断
- Phase 4.0.7 のレビューで先送りした項目: 操作の安定参照 (出現順 id でなく IR 由来 id) は 4.0.9 で。`EFFECT_BY_COLOR` の3箇所手動同期 (palette.rs / uno.config.ts / svgToSchema.ts) も 4.0.9 の IR 直結で解消される
- Phase 4.0.5 のレビューで先送りした項目: SPA に Vue Router のパスベースビューを足すときは serve.rs の 404 → index.html フォールバックが要る (serve.rs にコメント済み)。`?fn=` 連打競合は世代ガードで対応済みだが、AbortController による fetch キャンセルは未実装 (必要になったら 4.1 で)
- Phase 1.3 の `AuxRingRole.anchor_operation` は親 content から導出可能な情報の直接保持。content の並び替えが起きる変更では同期が必要。Phase 1.5/1.8 では問題にならなかったが、content を並び替える変更を入れるときは再確認
- ~~spec 宿題 4 件 (Phase 1.1 補完型 / Phase 1.3 補完型・アームガード / LoopBody serde 不統一 / EdgeLayerData 非対称)~~ → **Phase 2.0 の spec-v0.2.md で解消** (§4.2 追補・§4.4 JSON 規約・§4.3 Phase 3 方針)

- Phase 1.4 で `format!` 系マクロを io 効果に倒した (オーナー確定 2026-06-10 の様子見方針)。実コードを描画して紫/青の記号が過剰に感じられたら pure への変更を再判断する
- Phase 1.4 時点の効果テーブルは `tokio::io` 未登録 (tokio::net / tokio::fs と非対称)。false negative が目立ったら追加 (effects.rs に TODO あり)

- Phase 1.5 の `layout/constants.rs` の半径・ギャップ値は仮置き。Phase 1.6 の自己ホスティング SVG では破綻していないが、オーナー目視判定の結果次第で調整する（交差最小化の回転ステップ 0.2 rad、glyph 全周配置によるノード重なりも同タイミングで再評価）
- Phase 2.1 のコントリビューション検出より: `.claude/launch.json`（preview サーバ定義）と `minimal-dev-server-pattern.md` は ADDF scaffold テンプレート（launch.json.example）/ ADDF knowhow への昇格候補。既存の昇格候補群と合わせて一括 PR を検討

## 改善アクション

- **フロントエンド技術スタック確定 (Phase 4.0.5, 2026-06-11)**: Vue 3 + Vite+ + UnoCSS + Pinia + Vue Router + TypeScript strict (`type > interface`)。bun:ffi は条件付き保留。**Phase 4.0 はサーバ側 API まで完了済** — UI は 4.0.5 で Vue 実装 (素 JS の新 UI はサイクル中の指示どおり撤回した)。**UI 実装中は M1〜M5 ごとに目視確認** (オーナー指定)
- **Phase 4 計画書の Vue 前提への追補は 4.0.5 完了処理で実施**: 4.0 は完了済として書き換えない、4.1 以降の素 JS 想定記述だけ追補
- Phase 1.3 のレビュー指摘「`unwrap_or(u32::MAX)` センチネル禁止」が anchor 1箇所だけ修正され、同型の4箇所が Phase 1.4 レビューで再指摘された。**レビュー指摘を受けたパターンは修正時に `grep` で同型箇所を全て掃く**こと（パターン名で knowhow 化済みでも、既存コードの残留掃除は別作業として明示する）
- 同パターンが Phase 3.1 の新規コード (metrics.rs) で**3度目の再発**。残留掃除でなく新規執筆時の再発であり、規約の置き場所が `syn-visitor-patterns.md`（magia-rust 固有文書）だったため magia-core 執筆時に参照されなかったのが要因。`structural-diff-pattern.md` にも明記して文書を跨いで冗長化した。プロジェクト横断規約が増えてきたら CLAUDE.repo.md への昇格を検討
- Phase 3.5 (belka.rs) で**4度目の再発** — 文書冗長化でも新規執筆時には効かないことが確定したため、**CLAUDE.repo.md の POSD 節に規約を昇格済み** (同サイクル内で対応)。以後はセッション開始時に必ず読まれる
- 計画書の依存バージョン世代ずれが Phase 1.5 でも発生（petgraph 0.6→0.8、kurbo 0.11→0.13）。`cargo add --dry-run` で実値確認 → 計画書を実値で上書きする運用が機能した。Phase 1.6 以降も継続

- 計画書に「実装結果メモ」セクションを追記する形が機能した。テンプレ化候補
- 計画書記載の workspace 配置 (`magia/` 配下) と実装判断 (リポジトリルート直下) に差が出た。Phase 1.7 の `cargo install --path crates/magia-cli` 記述と整合するルート配置が正解だった。後続 Phase の計画書がルート配置を前提にしていることを再確認
- 計画書の依存ライブラリ表記 (`thiserror = "1"`) が実装時に最新 stable (`= "2"`) と乖離した。Phase 1.3 以降の計画書も同様の世代ずれが起きる可能性あり。実装時に必ず計画書の依存セクションも実値で上書きする運用を明文化済み (Phase 1.2 実装結果メモ参照)

## 完了済み
