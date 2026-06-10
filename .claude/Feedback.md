# Process Feedback

開発プロセスの振り返りと改善を記録する。

## 記録方法

タスク完了時や問題発生時に、以下のいずれかのセクションに追記する。

## オーナーフィードバック

## 問題の記録

- Phase 1.0 で `.claude/skills/addf-gui-test.md` が `.gitignore` に手書きで追加されていた。これは ADDF 側で初期化時に挿入されるテンプレートに含めるべきもの。ADDF 本体への PR 候補
- Phase 1.0 で「rust-cargo-workspace-bootstrap」ノウハウは現状プロジェクト固有 (`docs/knowhow/`) に置いたが、内容は ADDF 利用 Rust プロジェクト全般に役立つ。将来 `docs/knowhow/ADDF/` または ADDF 本体への昇格を検討
- Phase 1.3 のコントリビューション検出より: `addf-knowhow.exp.md` に記録した「統合先ファイルの冒頭メタコメントも実態に合わせて更新する」「INDEX は reindex を待たず手動同期してよい」は汎用的な教訓。ADDF 本体の `addf-knowhow.md` Phase 3 チェックリストへの追記 PR 候補
- Phase 1.1 で `EdgeLayerData` のレイヤー設計が `LayerData` (Option<XxxInfo> による Z 軸的拡張) と非対称 (フラットな data_volume / call_frequency 直値) になっている。Phase 3 でデータフロー解析が入るときに破壊的変更になりうるため、Phase 2/3 計画書を起こすタイミングで方針を明記する
- Phase 1.1 で spec §4.2 に未記載の補助型 (`OperationPayload`, `Cardinality`, `EdgeLayerData`, `ProjectMetadata`) を実装側で補完した。`project-docs/magia/spec-v0.2.md` を起こすときに正式追記する
- Phase 1.3 で `AuxRingRole` / `AuxRingKind` / `LoopKind` を spec 未記載のまま実装側で補完した (spec §5.2「分岐種別・ループ種別」の具体化)。match アームガードの扱い (通常アームと同一視) も spec-v0.2 で明記する
- Phase 1.3 の `AuxRingKind::LoopBody(LoopKind)` だけ serde 表現がオブジェクト形式 (`{"LoopBody":"For"}`) で他バリアントと不統一。Phase 2 dev-server の JSON コンシューマー実装前に `#[serde(tag)]` 等での統一を検討する
- Phase 1.3 の `AuxRingRole.anchor_operation` は親 content から導出可能な情報の直接保持。content の並び替えが起きる変更では同期が必要 (EdgeLayerData 非対称問題と同カテゴリ)。Phase 1.5 レイアウト実装時に再確認

## 改善アクション

- 計画書に「実装結果メモ」セクションを追記する形が機能した。テンプレ化候補
- 計画書記載の workspace 配置 (`magia/` 配下) と実装判断 (リポジトリルート直下) に差が出た。Phase 1.7 の `cargo install --path crates/magia-cli` 記述と整合するルート配置が正解だった。後続 Phase の計画書がルート配置を前提にしていることを再確認
- 計画書の依存ライブラリ表記 (`thiserror = "1"`) が実装時に最新 stable (`= "2"`) と乖離した。Phase 1.3 以降の計画書も同様の世代ずれが起きる可能性あり。実装時に必ず計画書の依存セクションも実値で上書きする運用を明文化済み (Phase 1.2 実装結果メモ参照)

## 完了済み
