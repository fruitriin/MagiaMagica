# TODO

`docs/plans/` の完了状態・優先度をトラックする。
`docs/plans/` と TODO が一致しなければ TODO を編集する。

## 現在のフェーズ: Phase 3 — Spell Diff と第2の式 (ベルカ)

Phase 1〜2 の計画群は全完了 (2026-06-11)。2026-06-11 に notes §12 のロードマップから
Phase 3 計画 6 本を立案した。**優先度はオーナー確認待ち** (暫定: 依存順)。
残存する判定待ち: Phase 1.8 の意匠再判定素材 (オーナー送付済み)。

## バックログ

| 優先度 | Phase | 計画ファイル | 状態 |
|---|---|---|---|
| 1 | 3.0 | [docs/plans/phase3.0-spec-v0.3.md](docs/plans/phase3.0-spec-v0.3.md) | 完了 |
| 2 | 3.1 | [docs/plans/phase3.1-diff-engine.md](docs/plans/phase3.1-diff-engine.md) | 完了 |
| 3 | 3.2 | [docs/plans/phase3.2-spell-diff.md](docs/plans/phase3.2-spell-diff.md) | 完了（意匠判定待ち） |
| 4 | 3.3 | [docs/plans/phase3.3-ci-integration.md](docs/plans/phase3.3-ci-integration.md) | 未着手 |
| 5 | 3.4 | [docs/plans/phase3.4-dataflow-ir.md](docs/plans/phase3.4-dataflow-ir.md) | 未着手 |
| 6 | 3.5 | [docs/plans/phase3.5-belka-style.md](docs/plans/phase3.5-belka-style.md) | 未着手 |

依存関係:
- 3.0 (仕様化) は全ての前提。3.1 → 3.2 → 3.3 が Spell Diff の系譜
- 3.4 → 3.5 がベルカ式の系譜 (3.1〜3.3 と独立して進められる)
- 3.4 は EdgeLayerData の破壊的再設計 (spec v0.2 §4.3 の既定方針) を含む

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
