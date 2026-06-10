# Phase 2.0 — 仕様書 v0.2 への昇格

## 出典

- `project-docs/magia/phase2plus-notes-v0.1.md` 付録 A (昇格対象の指定)
- `.claude/Feedback.md` に蓄積された spec 宿題 (Phase 1.1〜1.4 の実装補完)

## 目的

Phase 2 実装に入る前に `project-docs/magia/spec-v0.2.md` を起こし、
(1) notes の Phase 2 該当セクションを正式仕様化、(2) Phase 1 実装で spec に
先行して確定した型・規約を追記して、仕様と実装の乖離を解消する。

## スコープ

### やること

- notes 付録 A の指定どおり昇格:
  - notes §1 dev-server 化 → spec §7 として正式化 (Phase 2.1 計画の実装範囲に絞って具体化)
  - notes §2 レイヤーシステム → spec §5 を拡張 (位置共有制約を必須要件として明文化)
  - notes §3 フィルター言語 → spec §8 として正式化 (Phase 2.3 で実装する最小構文を確定)
  - notes §9 アクセシビリティ → spec §15 として新規追加 (呪文書き起こしの出力形式)
- Feedback.md の spec 宿題を §4.2 / §5.2 に正式追記して該当 Feedback 行を削除:
  - Phase 1.1 補完型: `OperationPayload` / `Cardinality` / `EdgeLayerData` / `ProjectMetadata`
  - Phase 1.3 補完型: `AuxRingRole` / `AuxRingKind` / `LoopKind`、match アームガードの扱い
  - Phase 1.4 確定事項: 効果ヒューリスティックの近似仕様 (use 展開・メソッド pure 扱い・マクロ白リスト)
  - `AuxRingKind::LoopBody` の serde 表現不統一 — v0.2 で表現を統一するか現状を正式仕様とするか判断を明記
- `EdgeLayerData` 非対称問題 (Phase 1.1 持ち越し) の方針を Phase 3 設計の前提として記載
- spec v0.1 → v0.2 の差分一覧 (付録 B バージョン履歴) を書く
- `project-docs/magia/INDEX-v0.5.md` の更新

### やらないこと

- Phase 3 以降のセクション昇格 (notes §4 CI 統合・§7 描画様式は Phase 3 で)
- 実装の変更 (本計画はドキュメントのみ。serde 表現統一を「やる」と決めた場合は別計画に切り出す)

## 設計上の判断

- 仕様化は「実装が既に確定した事実の追認」と「Phase 2 実装前に決めるべき契約」の2部構成にする
- ファンタジー語彙とプログラミング語彙の相互乗り入れ (CLAUDE.repo.md の哲学) を維持する

## 受け入れ基準

- [x] `project-docs/magia/spec-v0.2.md` が存在し、付録 A 指定の4セクションを含む（レビューエージェントが4件全対応を確認）
- [x] Feedback.md の spec 宿題 4 項目が解消され、該当行が削除されている
- [x] spec v0.1 からの差分一覧がある
- [x] INDEX-v0.5.md が v0.2 を参照する

## 後続

- Phase 2.1〜2.4 の実装計画は本仕様を契約として参照する

## 実装結果メモ (2026-06-11)

- **改訂方式**: 全文書き直しではなく「v0.1 基底 + 増補差分」方式を採用（記載のない節は
  v0.1 が有効）。理由は差分のレビュー可能性と、v1.0 で全文統合する方針との整合。
  方式判断は spec v0.2 冒頭に明記
- **LoopBody serde 判断 (宿題解消)**: 現状の externally-tagged 混在表現を正式仕様とした
  （§4.4）。非 Rust コンシューマー出現を再訪条件として明示。JSON 具体例は
  `magia emit-ir` の実出力と突き合わせて検証済み
- **整合レビュー**: Critical 0 / High 2（節順序・余弦定理の記述精度）/ Medium 4 / Low 3。
  全件反映（§5.2 追補を §5 先頭へ移動、占有帯計算の説明精緻化、AuxRingRole の JSON
  完全例追加、Phase 1.7 CLI との規約整合明記、aria 属性の具体化、「同型」→「同様の
  設計パターン」等）
- contribution 検出は対象が project-docs / Feedback / INDEX のみのため自明（分離違反なし、
  全てプロジェクト固有文書）としてエージェント起動を省略
