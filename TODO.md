# TODO

`docs/plans/` の完了状態・優先度をトラックする。
`docs/plans/` と TODO が一致しなければ TODO を編集する。

## 現在のフェーズ: Phase 1.8 (品質向上) → Phase 2 — dev-server とレイヤーシステム

Phase 1 (M1〜M7) は全マイルストーン完了。2026-06-11 にオーナーリクエストに基づき
Phase 1.8 (品質向上) + Phase 2.0〜2.4 の計画を立案した。**優先度はオーナー確認待ち**
(暫定: 意匠フィードバック対応の 1.8 を先頭、以降は依存順)。

## バックログ

| 優先度 | Phase | 計画ファイル | 状態 |
|---|---|---|---|
| 1 | 1.8 | [docs/plans/phase1.8-layout-collision-avoidance.md](docs/plans/phase1.8-layout-collision-avoidance.md) | 完了（再判定素材送付済み） |
| 2 | 2.0 | [docs/plans/phase2.0-spec-v0.2.md](docs/plans/phase2.0-spec-v0.2.md) | 完了 |
| 3 | 2.1 | [docs/plans/phase2.1-dev-server.md](docs/plans/phase2.1-dev-server.md) | 完了 |
| 4 | 2.2 | [docs/plans/phase2.2-layer-toggle.md](docs/plans/phase2.2-layer-toggle.md) | 完了 |
| 5 | 2.3 | [docs/plans/phase2.3-filter-dsl.md](docs/plans/phase2.3-filter-dsl.md) | 未着手 |
| 6 | 2.4 | [docs/plans/phase2.4-transcript.md](docs/plans/phase2.4-transcript.md) | 未着手 |

依存関係:
- 1.8 は独立 (意匠フィードバック対応。dev-server 前に済ませると体験品質が上がる)
- 2.0 (仕様化) は 2.1〜2.4 の前提
- 2.2 は 2.1 の HTML 土台に載る。2.3 は 2.2 のパレットと連携。2.4 は独立 (いつでも可)

オーナーリクエスト (消化済み 2026-06-11):
- ~~Phase 2 計画書を notes §1〜§3 から起こす~~ → 2.1 / 2.2 / 2.3 として作成
- ~~品質向上計画を追加する~~ → 1.8 として作成 (+ 2.0 仕様整合, 2.4 アクセシビリティ)

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
