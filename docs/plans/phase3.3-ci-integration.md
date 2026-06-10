# Phase 3.3 — CI 統合 (GitHub Actions + PR コメント)

## 出典

- `project-docs/magia/phase2plus-notes-v0.1.md` §4.1 / §4.3
- spec v0.3 §9 (Phase 3.0 で確定)

## 目的

PR ごとに「Spell changed」を自動投稿する GitHub Actions 統合を作る。
本リポジトリ自身をドッグフーディングの場とする。

## スコープ

### やること

- `magia diff --git <BASE_REV> <FILE> --fn <NAME>`: git show でベース版を取得して比較
  (3.1 の糖衣。リポジトリ内で完結し、CI からそのまま使える)
- 変更関数の検出: `git diff --name-only` + `magia list` の突き合わせで
  「変更された .rs ファイル内の関数」を列挙する補助コマンド (`magia changed --git <REV>`)
- GitHub Actions ワークフロー (`.github/workflows/spell-diff.yml`):
  1. PR で変更された関数を検出
  2. 各関数の SpellDiff (テキスト + SVG) を生成
  3. PR コメントに投稿 (gh api / sticky comment 方式で更新)
- しきい値判定 (spec v0.3 §9 の最小主義): fail 対象は
  「unsafe ブロックの新規追加」のみから開始 (status check)。他は警告表示に留める
- 本リポジトリでの実地確認 (実 PR を1本作って動作を見る)

### やらないこと

- GitLab / Bitbucket 対応 (GitHub のみ)
- しきい値の拡充 (複雑度急増等) — 運用してから判断
- 画像ホスティングの一般解 (PR コメントには SVG を artifact リンク + テキスト要約で。
  画像インライン表示が必要なら実装時に判断)

## 設計上の判断

- CI ロジックは極力 `magia` コマンド側に寄せ、ワークフロー YAML は薄く保つ
  (ローカルで同じことが再現できる = デバッグ可能)
- 「判決を下さず注意を誘導する」原則: fail は明確に有害な変化のみ (spec v0.3 §9)

## 受け入れ基準

- [ ] `magia diff --git` と `magia changed --git` がローカルで動く
- [ ] 本リポジトリの実 PR で Spell Diff コメントが付く (実地確認)
- [ ] unsafe 新規追加で status check が fail する (テスト PR で確認)
- [ ] `cargo test --workspace` / clippy 警告0

## 後続

- 運用フィードバックを Phase 3 振り返りで収集し、しきい値とコメント形式を調整
