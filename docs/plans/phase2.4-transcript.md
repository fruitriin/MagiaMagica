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

- [x] 全 fixtures で transcribe が動き、notes §9.1 の例と同等の情報を含む (種別内訳など §9.1 を上回る情報量、レビューで確認)
- [x] 同じ IR から2回生成して完全一致 (決定論、5回反復テスト)
- [x] dev-server の HTML に書き起こしが埋め込まれている (visually-hidden 完全形 + role="region")
- [x] `cargo test --workspace` (149本) / clippy 警告0

## 後続

- Phase 3 でレイヤー対応書き起こしへ拡張

## 実装結果メモ (2026-06-11)

### 設計判断の確定

- EffectSet → カテゴリの分類ロジックを `filter::EffectCategory::of` に昇格
  (レビュー M-2: render 内部の palette に transcript が依存しない。色相・絞り込み・
  ラベルの三者が同じ分類を共有)
- 同一呼び出し先でカテゴリが揺れた場合は `danger_rank` で**危険側を採用**して集計
  (H-1: 聞き手に安全側の誤解を与えない)
- serve は `Rendered { svg, transcript }` で SVG と書き起こしを**対で生成**
  (同一 IR の射影の一致保証)。エラー中は両方とも直前の正常値を保持
  (視覚側と同じ扱い、エラー自体は #status で伝わる — M-3 の判断をコメント化)
- visually-hidden は clip (旧) / clip-path (新) 併記の完全形、role="region" (H-2)

### レビュー対応 (Stage 2)

- 修正: H-1 (危険度優先 + テスト) / H-2 (CSS 完全形・role) / M-1 (Result 優先の根拠コメント) /
  M-2 (分類の filter 昇格) / M-3 (保持方針のコメント化) / M-4 (fixture をワークスペース
  ルートに統一、async_await へ変更) / L-1 (閾値根拠コメント) / I-2 (テスト名)
- 受理: L-2 (カテゴリ集計の Vec::contains — 最大5要素のため) / I-1 (must_use 文言)
- 教訓: fixture が「ワークスペースルート」と「magia-rust テスト用」の2系統あり
  同名・類似名で取り違えた (async_io vs async_await)。テストの fixture は原則
  ワークスペースルートを参照する規約とした
