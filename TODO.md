# TODO

`docs/plans/` の完了状態・優先度をトラックする。
`docs/plans/` と TODO が一致しなければ TODO を編集する。

## 現在のフェーズ: Phase 4 — フロントエンド充実

**Phase 3 (3.0〜3.5) は全完了 (2026-06-11)** — Spell Diff 系譜 + CI 統合 + ベルカ式。
残存する判定待ち: Phase 1.8 / 3.2 / 3.5 の意匠判定素材 (オーナー送付済み)。
2026-06-11 にオーナー要望から **Phase 4 系 (フロントエンド充実) ストリームを立ち上げ**、
4.0 (ソース連動ビュー) は**サーバ側 API まで完了** (2026-06-11、スコープ縮小はオーナー指示)。次は 4.0.5 (Vue 基盤 — UI 実装をここに集約)。
Phase 3 振り返り (二式並置ビュー・レイヤー差分分解・knowhow 一括昇格 PR) は
オーナー判定が出揃った節目に実施する。

## バックログ

| 優先度 | Phase | 計画ファイル | 状態 |
|---|---|---|---|
| 1 | 3.0 | [docs/plans/phase3.0-spec-v0.3.md](docs/plans/phase3.0-spec-v0.3.md) | 完了 |
| 2 | 3.1 | [docs/plans/phase3.1-diff-engine.md](docs/plans/phase3.1-diff-engine.md) | 完了 |
| 3 | 3.2 | [docs/plans/phase3.2-spell-diff.md](docs/plans/phase3.2-spell-diff.md) | 完了（意匠判定待ち） |
| 4 | 3.3 | [docs/plans/phase3.3-ci-integration.md](docs/plans/phase3.3-ci-integration.md) | 完了 |
| 5 | 3.4 | [docs/plans/phase3.4-dataflow-ir.md](docs/plans/phase3.4-dataflow-ir.md) | 完了 |
| 6 | 3.5 | [docs/plans/phase3.5-belka-style.md](docs/plans/phase3.5-belka-style.md) | 完了（意匠判定待ち） |
| 7 | 4.0.5 | [docs/plans/phase4.0.5-frontend-foundation.md](docs/plans/phase4.0.5-frontend-foundation.md) | 未着手（**4.0 の前段**） |
| 8 | 4.0 | [docs/plans/phase4.0-source-paired-view.md](docs/plans/phase4.0-source-paired-view.md) | 完了（サーバ側 API まで。UI は 4.0.5 に移管） |
| 9 | 4.1 | [docs/plans/phase4.1-pinned-focus-view.md](docs/plans/phase4.1-pinned-focus-view.md) | 未着手 |
| 10 | 4.2 | [docs/plans/phase4.2-proximity-model.md](docs/plans/phase4.2-proximity-model.md) | 未着手 |
| 11 | 4.3 | [docs/plans/phase4.3-composite-still-render.md](docs/plans/phase4.3-composite-still-render.md) | 未着手 |
| 12 | 4.4 | [docs/plans/phase4.4-call-jump.md](docs/plans/phase4.4-call-jump.md) | 未着手（イメージ感のみ） |
| 13 | 4.5 | [docs/plans/phase4.5-workspace-overview.md](docs/plans/phase4.5-workspace-overview.md) | 未着手（イメージ感のみ） |
| 14 | 4.6 | [docs/plans/phase4.6-theme-and-diff-overlay.md](docs/plans/phase4.6-theme-and-diff-overlay.md) | 未着手（イメージ感のみ） |

依存関係:
- 3.0 (仕様化) は全ての前提。3.1 → 3.2 → 3.3 が Spell Diff の系譜
- 3.4 → 3.5 がベルカ式の系譜 (3.1〜3.3 と独立して進められる)
- 3.4 は EdgeLayerData の破壊的再設計 (spec v0.2 §4.3 の既定方針) を含む
- **4.0.5 が Phase 4 全体の前提**。Vue 3 + Vite+ 基盤を立ち上げ、Phase 2.x の inline HTML/JS を Vue 化する。4.0 以降は本基盤の上に構築
- **4.0 → 4.1 → 4.2 が「ピン中心ビュー」のコア系譜**。4.0 はペアビュー基盤、4.1 はフォーカス + リング配置、4.2 は近接度モデルを 4.1 のスタブから本実装に差し替え
- **4.3 (静止画) は 4.1 のレイアウト関数を共有** — `magia render --focus` で動的UIと同じ構図を1枚 SVG に
- **4.4 (呼び出しジャンプ) は 4.1 + 4.2 + Phase 3.4 データフロー IR に依存**
- **4.5 (ワークスペース俯瞰) は 4.0〜4.4 完了後に詳細精緻化**
- **4.6 (テーマ + Spell Diff overlay) は Phase 3.2 / 3.5 完了済成果物を 4.x 上に重ねる**
- 4.4〜4.6 は計画書時点では **イメージ感のみ**、実装着手時に内容を精緻化する (オーナー方針 2026-06-11)
- notes の Phase 4 (多言語アダプタ) は **Phase 5 系に繰り下げ**

---

## アーカイブ

| Phase | 計画ファイル | 状態 |
|---|---|---|
| 1.0 | [docs/plans/phase1.0-workspace-bootstrap.md](docs/plans/phase1.0-workspace-bootstrap.md) | 完了 |
| 1.1 | [docs/plans/phase1.1-ir-skeleton.md](docs/plans/phase1.1-ir-skeleton.md) | 完了 |
| 1.2 | [docs/plans/phase1.2-syn-to-ir.md](docs/plans/phase1.2-syn-to-ir.md) | 完了 |
| 1.3 | [docs/plans/phase1.3-aux-rings.md](docs/plans/phase1.3-aux-rings.md) | 完了 |
| 1.4 | [docs/plans/phase1.4-summon-effects.md](docs/plans/phase1.4-summon-effects.md) | 完了 |
| 1.5 | [docs/plans/phase1.5-layout-engine.md](docs/plans/phase1.5-layout-engine.md) | 完了 |
| 1.6 | [docs/plans/phase1.6-svg-renderer-midchilda.md](docs/plans/phase1.6-svg-renderer-midchilda.md) | 完了（意匠は部分合格・1.8 で再判定） |
| 1.7 | [docs/plans/phase1.7-cli-integration.md](docs/plans/phase1.7-cli-integration.md) | 完了 |
| 1.8 | [docs/plans/phase1.8-layout-collision-avoidance.md](docs/plans/phase1.8-layout-collision-avoidance.md) | 完了（意匠再判定待ち） |
| 2.0 | [docs/plans/phase2.0-spec-v0.2.md](docs/plans/phase2.0-spec-v0.2.md) | 完了 |
| 2.1 | [docs/plans/phase2.1-dev-server.md](docs/plans/phase2.1-dev-server.md) | 完了 |
| 2.2 | [docs/plans/phase2.2-layer-toggle.md](docs/plans/phase2.2-layer-toggle.md) | 完了 |
| 2.3 | [docs/plans/phase2.3-filter-dsl.md](docs/plans/phase2.3-filter-dsl.md) | 完了 |
| 2.4 | [docs/plans/phase2.4-transcript.md](docs/plans/phase2.4-transcript.md) | 完了 |
