# Phase 4.3.7 — Spell Diff を Web (serve) に載せる

## 出典

- オーナー要望 (2026-06-12、4.3 M4 判定時): 「diff を web 上に載せられる?」
- **Phase 4.6 (テーマ + Spell Diff overlay) からの分離**: 4.6 は「4.1〜4.5 完了後の
  仕上げバスケット」だが、diff overlay の前提は 4.3 M4 (diff_spell_ir +
  MagicCircle の overlay prop) で揃った — 4.5 を待つ理由がないため先行切り出し。
  4.6 にはテーマ切替・ベルカ同時表示・URL 軸統合が残る

## 目的

serve の動的 UI で、フォーカス関数の Spell Diff (金ハロー = 追加 / シアン = 変更 /
灰破線ゴースト = 削除) を**ピン位置のまま**重ね合わせ表示する。
CLI (`magia diff --svg`) でしか見られなかった差分が、ブラウザの探索フロー
(ピン遷移・ホバー・インスペクタ) の中で見える。

## スコープ

### やること

- **serve**: `/spell/<fn>?diff=<REV>` — gitio (Phase 3.3) で REV 時点の同ファイルを
  読み、現行を after として `diff_spell_ir` を実行。応答に `diff_overlay`
  (DiffMarkIr 列) と `diff_report` (メトリクス変化の要約テキスト) を付与
  - REV 時点に関数が無い (新規関数) は「全部 added」でなく diff_overlay 省略 +
    案内文 (差分の概念が成立しないため)
  - REV が解決できない (git 外・不正な rev) はエラーでなく案内文 (会話を切らない)
- **Vue**:
  - URL `?diff=<rev>` 同期 (replace — 4.0.5 のクエリ軸規約)
  - MagicCircle に overlay を渡す (M4 実装済みの prop をピンビュー経路に配線)
  - パレットに diff 入力 (rev テキストボックス + クリア)。プリセット候補
    (HEAD~1 / main) をワンクリックで
  - diff_report (「+2 操作 / 早期リターン +1」級の要約) をパレット内に表示
- **viewBox**: ゴースト拡張済み viewBox (diff_spell_ir が返す) をピンビューの
  focus_layout とどう合成するか — 実装時に決める (拡張分だけ focus viewBox を
  広げるのが素直)
- **テスト**: serve 統合 (?diff= 応答契約 / 不正 rev の案内) + e2e (rev 入力 →
  ハロー表示 → クリア)

### やらないこと (4.6 に残す)

- テーマ・パレット切替 (`?theme=` / `?palette=`)
- ベルカ式の diff / 同時表示 (`?style=both`)
- before/after の切替トグル表示 (overlay は after 基準のみ — 3者比較もしない)
- パーマリンク生成

## 設計上の判断

- **diff の基準は「git rev → 現行ファイル」のみ**: CLI の「ファイル2つ比較」は
  web の文脈 (1ファイル監視) に合わない。gitio の `git show REV:path` 経路を再利用
- **SSE 更新との整合**: ファイル保存で after 側だけ動く — diff は保存のたびに
  再計算され「いま書いている変更がリアルタイムで金ハローになる」。これが本計画の
  一番の体験価値 (CLI ではできない)
- **ピンビューとの合成**: overlay はフォーカス魔法陣にだけ重ねる (周辺チップには
  出さない — チップの diff は 4.5 ワークスペース俯瞰の領分)

## 受け入れ基準

- [ ] `?diff=HEAD~1` でフォーカス魔法陣にハロー/ゴーストが重なる
- [ ] ファイル保存 (SSE) で diff が追従する (live diff)
- [ ] 不正 rev / git 外で UI が壊れず案内が出る
- [ ] `?diff=` なしの応答・表示は従来と完全互換
- [ ] cargo / vp check / vitest / playwright 通過

## 依存

- Phase 4.3 M4 (diff_spell_ir + MagicCircle overlay prop) — **完了済 (2026-06-12)**
- Phase 3.3 gitio — 完了済
- 4.3 M5 (Rust SVG レンダラ削除) とは独立 — どちらが先でもよい

## 実装ステップ (粗粒度)

1. serve: gitio 配線 + `?diff=` パース + diff_spell_ir + 応答拡張
2. Vue: URL 同期 + overlay 配線 + パレット UI + diff_report 表示
3. テスト (serve 統合 + e2e) + 素材 (自己ホスティングの実 diff) → 判定
4. Stage 1/2 + 完了処理
