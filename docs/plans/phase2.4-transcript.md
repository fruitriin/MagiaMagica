# Phase 2.4 — 呪文書き起こし (アクセシビリティ基礎)

## 出典

- `project-docs/magia/phase2plus-notes-v0.1.md` §9 (アクセシビリティ)

## 目的

図形ベース表現の視覚障害者対応として、同じ IR から構造化テキストの
「呪文書き起こし (Incantation Transcript)」を生成する。notes §9.1 の基本機能。

## スコープ

### やること

- `magia transcribe <FILE> --fn <NAME>` サブコマンド
- IR (`MagiaGraph`) からの書き起こし生成 (`magia-core::transcript`):
  notes §9.1 の例の形式に従う —
  メインリングの規模 (Operation 数)、補助リングの数と種別 (分岐/ループ・入れ子)、
  外部呼び出し (呼び出し先・効果カテゴリ・回数集計)、早期リターン経路数、
  戻り値型 (Result/Option)、async/await
- 同一呼び出し先の集計 (「db::query (DB副作用、1回)」形式) — IR では glyph が重複している前提
- 出力はプレーンテキスト (スクリーンリーダー前提、装飾なし)。決定論的
- dev-server の HTML に `aria-label` / 不可視テキストとして同内容を埋め込む
- テスト: fixtures 各種からの書き起こしのスナップショット (insta)

### やらないこと

- レイヤー切替に対応した書き起こし (Phase 3)
- 動的解析情報を含む包括版 (Phase 5)
- 多言語化 (日本語のみ。英語は README 英語化と同じタイミング)

## 設計上の判断

- 書き起こしはレンダラと同格の「IR の射影」として `magia-core` に置く
  (SVG と書き起こしが同じ IR から出ることが、内容の一致を保証する)
- 文面テンプレートは関数化して分離し、Phase 3 のレイヤー対応で文を差し込める構造にする

## 受け入れ基準

- [ ] 全 fixtures で transcribe が動き、notes §9.1 の例と同等の情報を含む
- [ ] 同じ IR から2回生成して完全一致 (決定論)
- [ ] dev-server の HTML に書き起こしが埋め込まれている
- [ ] `cargo test --workspace` / clippy 警告0

## 後続

- Phase 3 でレイヤー対応書き起こしへ拡張
