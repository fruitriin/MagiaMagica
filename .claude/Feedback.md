# Process Feedback

開発プロセスの振り返りと改善を記録する。

## 記録方法

タスク完了時や問題発生時に、以下のいずれかのセクションに追記する。

## オーナーフィードバック

- **意匠判定 (Phase 1.8 で対応済み・再判定待ち)**: Phase 1.6 の判定で「write_document 級はお洒落・write_control_flow 級は破綻」と確定 → Phase 1.8 で衝突回避を実装。**write_document の見た目はほぼ不変**（重なっていた青記号1個が 1.6px 退避したのみ、8px 閾値の回帰テストで固定）、**write_control_flow はリング重なり解消**。再判定素材3本（write_control_flow / write_document / dense_dispatch）を送付済み。次の調整指示があれば `layout/constants.rs` の定数または新規計画で対応
- **報告 (Phase 1.7)**: Phase 1 (M1〜M7) が完了し、`cargo install --path crates/magia-cli` で配布可能な状態です。リリース（バージョン採番・チェンジログ）を行う場合は `/addf-release` を実行してください
- **確認依頼 (計画立案 2026-06-11)**: Phase 1.8 + Phase 2.0〜2.4 の計画 6 本を起こしました。暫定優先度（1.8 レイアウト衝突回避を先頭）で TODO に登録済み。**優先度の変更や計画への注文があればお知らせください**。何もなければ次サイクルから 1.8 に着手します

## 問題の記録

- Phase 1.0 で `.claude/skills/addf-gui-test.md` が `.gitignore` に手書きで追加されていた。これは ADDF 側で初期化時に挿入されるテンプレートに含めるべきもの。ADDF 本体への PR 候補
- Phase 1.0 で「rust-cargo-workspace-bootstrap」ノウハウは現状プロジェクト固有 (`docs/knowhow/`) に置いたが、内容は ADDF 利用 Rust プロジェクト全般に役立つ。将来 `docs/knowhow/ADDF/` または ADDF 本体への昇格を検討
- Phase 1.3 のコントリビューション検出より: `addf-knowhow.exp.md` に記録した「統合先ファイルの冒頭メタコメントも実態に合わせて更新する」「INDEX は reindex を待たず手動同期してよい」は汎用的な教訓。ADDF 本体の `addf-knowhow.md` Phase 3 チェックリストへの追記 PR 候補
- Phase 1.1 で `EdgeLayerData` のレイヤー設計が `LayerData` (Option<XxxInfo> による Z 軸的拡張) と非対称 (フラットな data_volume / call_frequency 直値) になっている。Phase 3 でデータフロー解析が入るときに破壊的変更になりうるため、Phase 2/3 計画書を起こすタイミングで方針を明記する
- Phase 1.1 で spec §4.2 に未記載の補助型 (`OperationPayload`, `Cardinality`, `EdgeLayerData`, `ProjectMetadata`) を実装側で補完した。`project-docs/magia/spec-v0.2.md` を起こすときに正式追記する
- Phase 1.3 で `AuxRingRole` / `AuxRingKind` / `LoopKind` を spec 未記載のまま実装側で補完した (spec §5.2「分岐種別・ループ種別」の具体化)。match アームガードの扱い (通常アームと同一視) も spec-v0.2 で明記する
- Phase 1.3 の `AuxRingKind::LoopBody(LoopKind)` だけ serde 表現がオブジェクト形式 (`{"LoopBody":"For"}`) で他バリアントと不統一。Phase 2 dev-server の JSON コンシューマー実装前に `#[serde(tag)]` 等での統一を検討する
- Phase 1.3 の `AuxRingRole.anchor_operation` は親 content から導出可能な情報の直接保持。content の並び替えが起きる変更では同期が必要 (EdgeLayerData 非対称問題と同カテゴリ)。Phase 1.5 レイアウト実装時に再確認

- Phase 1.4 で `format!` 系マクロを io 効果に倒した (オーナー確定 2026-06-10 の様子見方針)。実コードを描画して紫/青の記号が過剰に感じられたら pure への変更を再判断する
- Phase 1.4 時点の効果テーブルは `tokio::io` 未登録 (tokio::net / tokio::fs と非対称)。false negative が目立ったら追加 (effects.rs に TODO あり)

- Phase 1.5 の `layout/constants.rs` の半径・ギャップ値は仮置き。Phase 1.6 の自己ホスティング SVG では破綻していないが、オーナー目視判定の結果次第で調整する（交差最小化の回転ステップ 0.2 rad、glyph 全周配置によるノード重なりも同タイミングで再評価）
- Phase 1.6 のコントリビューション検出より: `docs/knowhow/svg-deterministic-rendering.md` は SVG を生成する Rust プロジェクト全般に通用する汎用知見。ADDF 本体への昇格候補（rust-cargo-workspace-bootstrap 等と同じ扱い、Phase 1 完了後の節目で判断）
- Phase 1.7 のコントリビューション検出より: `docs/knowhow/clap-cli-integration-pattern.md` の汎用部分（予約語フラグ・value_delimiter・assert_cmd・エラー責務分担）も ADDF 昇格候補。**Phase 1 が完了したため、昇格候補 4 件（cargo-workspace-bootstrap / syn-visitor-patterns / svg-deterministic-rendering / clap-cli-integration）の一括昇格を「品質向上計画」の一部として Phase 2 計画立案時に検討する**

## 改善アクション

- Phase 1.3 のレビュー指摘「`unwrap_or(u32::MAX)` センチネル禁止」が anchor 1箇所だけ修正され、同型の4箇所が Phase 1.4 レビューで再指摘された。**レビュー指摘を受けたパターンは修正時に `grep` で同型箇所を全て掃く**こと（パターン名で knowhow 化済みでも、既存コードの残留掃除は別作業として明示する）
- 計画書の依存バージョン世代ずれが Phase 1.5 でも発生（petgraph 0.6→0.8、kurbo 0.11→0.13）。`cargo add --dry-run` で実値確認 → 計画書を実値で上書きする運用が機能した。Phase 1.6 以降も継続

- 計画書に「実装結果メモ」セクションを追記する形が機能した。テンプレ化候補
- 計画書記載の workspace 配置 (`magia/` 配下) と実装判断 (リポジトリルート直下) に差が出た。Phase 1.7 の `cargo install --path crates/magia-cli` 記述と整合するルート配置が正解だった。後続 Phase の計画書がルート配置を前提にしていることを再確認
- 計画書の依存ライブラリ表記 (`thiserror = "1"`) が実装時に最新 stable (`= "2"`) と乖離した。Phase 1.3 以降の計画書も同様の世代ずれが起きる可能性あり。実装時に必ず計画書の依存セクションも実値で上書きする運用を明文化済み (Phase 1.2 実装結果メモ参照)

## 完了済み
