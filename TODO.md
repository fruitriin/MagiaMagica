# TODO

`docs/plans/` の完了状態・優先度をトラックする。
`docs/plans/` と TODO が一致しなければ TODO を編集する。

## 現在のフェーズ: Phase 1 — 単一 Rust 関数 → ミッドチルダ式 SVG

## バックログ

| 優先度 | Phase | 計画ファイル | 状態 |
|---|---|---|---|
| 1 | 1.0 | [docs/plans/phase1.0-workspace-bootstrap.md](docs/plans/phase1.0-workspace-bootstrap.md) | 完了 |
| 2 | 1.1 | [docs/plans/phase1.1-ir-skeleton.md](docs/plans/phase1.1-ir-skeleton.md) | 完了 |
| 3 | 1.2 | [docs/plans/phase1.2-syn-to-ir.md](docs/plans/phase1.2-syn-to-ir.md) | 完了 |
| 4 | 1.3 | [docs/plans/phase1.3-aux-rings.md](docs/plans/phase1.3-aux-rings.md) | 完了 |
| 5 | 1.4 | [docs/plans/phase1.4-summon-effects.md](docs/plans/phase1.4-summon-effects.md) | 完了 |
| 6 | 1.5 | [docs/plans/phase1.5-layout-engine.md](docs/plans/phase1.5-layout-engine.md) | 完了 |
| 7 | 1.6 | [docs/plans/phase1.6-svg-renderer-midchilda.md](docs/plans/phase1.6-svg-renderer-midchilda.md) | 完了（オーナー目視判定待ち） |
| 8 | 1.7 | [docs/plans/phase1.7-cli-integration.md](docs/plans/phase1.7-cli-integration.md) | 未着手 |

優先度は実装順序 (1.0 → 1.7) に対応。各 Phase 1.x は前段の成果物に依存するため、原則として順次着手する。

オーナーリクエスト:
タスクが無くなったら以下に取り組んでください
- Phase 1 完了後、Phase 2 (dev-server, レイヤー切替, フィルター DSL) の計画書を `project-docs/magia/phase2plus-notes-v0.1.md` §1〜§3 から起こす
- プロジェクトの品質を向上させる計画を追加する

---

## アーカイブ

| Phase | 計画ファイル | 状態 |
|---|---|---|
