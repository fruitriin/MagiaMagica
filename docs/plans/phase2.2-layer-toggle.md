# Phase 2.2 — 対話的レイヤー切替 (レイヤーパレット)

## 出典

- `project-docs/magia/phase2plus-notes-v0.1.md` §2 (レイヤーシステム)
- `project-docs/magia/spec-v0.1.md` §6.1.5 (`<g>` 分離は Phase 1 で確保済み)

## 目的

Phase 2.1 の dev-server 画面に Photoshop 風のレイヤーパレットを追加し、
SVG を再生成せず **CSS のクラス操作だけ**でレイヤーの ON/OFF・透明度調整を行えるようにする。

## スコープ

### やること

- レイヤーパレット UI (素の HTML/CSS/JS、フレームワークなし):
  - 各レイヤー (`control_flow` / `effects` / `type_info`) の表示チェックボックス
  - 透明度スライダー (CSS `opacity`)
  - 全表示 / 全非表示ボタン
- 切替状態は URL クエリ (`?layers=effects,type_info`) に反映し、リロード・共有で再現可能にする
- dev-server の自動更新 (Phase 2.1) 後もパレット状態が保持されること
- CLI `--layers` (Phase 1.7) と同じレイヤー名語彙を使う

### やらないこと

- 色相の変更・AND/OR 合成・プリセット保存 (notes §2.3 の残り) — Phase 2.3 の DSL と合流して実装
- 新レイヤーの追加 (データフロー等は Phase 3)
- SVG の再生成を伴う切替

## 設計上の判断

- **位置共有制約 (notes §2.2) はテストで明文化する**: レイヤーの組み合わせを変えても
  `LayoutResult` が不変であることを確認するテストを追加 (現実装は単一レイアウトなので自明に通るが、
  将来の退行防止として制約を機械化する)
- JS は WebSocket 受信 + クラス切替のみの最小実装。ビルドツールは導入しない

## 受け入れ基準

- [x] ブラウザ上でレイヤーを ON/OFF でき、図形の位置が変わらない (preview_eval で display:none を実機検証)
- [x] 透明度スライダーが効く (opacity 0.3 を実機検証)
- [x] URL クエリで切替状態を共有できる (?layers=...&op=... の往復、リロード復元、パレット UI 同期まで実機検証)
- [x] 位置共有制約のテストが存在する (layout_ignores_display_layers — control_flow 以外の全 LayerData フィールドを剥がして LayoutResult 完全一致)
- [x] `cargo test --workspace` (124本) / clippy 警告0

## 後続

- Phase 2.3 のフィルター DSL がパレット状態の保存形式 (プリセット) を引き受ける

## 実装結果メモ (2026-06-11)

### 実機検証で発見・修正したバグ

- **クエリ付き URL が 404**: ルート照合が `url == "/"` の完全一致だったため、
  URL クエリに状態を保存する機能自体がリロードで自壊していた。
  `url.split('?').next()` でパスを切り出して照合するよう修正 + 回帰テスト化。
  **単体テストでは検出できず、preview のリロード動線検証で発見** (knowhow に記録)

### 設計判断の確定

- パレットは素の HTML/CSS/JS (フレームワークなし)。状態は visible (Set) + opacity (map) を
  URLSearchParams と双方向同期、SSE の SVG 差し替え後に毎回 apply
- SVG 挿入は innerHTML でなく **DOMParser + replaceChildren** (スクリプト実行を遮断する
  多層防御。レンダラは XML エスケープ済みだが、レビュー M-1 対応)
- JS の cssClass() とレンダラの `<g>` クラス名の一致は統合テストで契約化 (レビュー H-2 対応)

### レビュー対応 (Stage 2)

- 修正: op パースの最初のコロン分割 (H-1) / クラス名一致の契約テスト (H-2) /
  DOMParser 化 (M-1) / opacity の浮動小数比較を許容誤差つきに (M-2) /
  位置共有テストの剥がし対象を control_flow 以外の全フィールドに拡大 (M-3。
  effects は LayerData でなく Operation 側にある旨をコメント化) /
  パレット input の早期 return (L-1) / Content-Type 検証 (L-2)
- 受理 (コスメティック): URLSearchParams がカンマを %2C にエンコードする (M-4。
  機能影響なし、手動構築への変更は見送り)
