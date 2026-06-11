# Phase 4.3 — 複合静止画レンダ (ピン中心構図を1枚の SVG に)

## 出典

- オーナー方針 (2026-06-11): 「静止画出力 (CLI render) と動的UI (serve) は別断面として両方厚く育てたい」
- 親計画: Phase 4.1 のピン中心レイアウトを静止画に転用

## 目的

`magia render` を「単一関数 → 単一 SVG」から「**フォーカス関数 + 周辺関数の縮小盾を含む1枚の SVG**」に拡張する。
README・ドキュメント・PR 添付などの静的な成果物として「いま見ているコードの世界感」を1枚で共有できる形にする。

動的 UI (4.1) と静止画 (4.3) は同じレイアウト関数を共有し、出力先と対話性だけが違う設計に揃える。

## スコープ

### やること

- **CLI 拡張**: `magia render <FILE> --focus <fn> [--neighbors all|inner|none] [--out PATH]`
  - `--focus` 指定で 4.1 と同じピン中心構図を生成
  - `--neighbors all` (既定) / `inner` (内リングのみ) / `none` (フォーカスのみ = 旧 render 互換)
- **共有レイアウト**: Phase 4.1 で実装した `render_focus_layout` を `magia-core` から呼び出す。serve / render で同一コード
- **静止画固有の調整**:
  - ピン遷移アニメ・ホバー・キーボードナビは無効 (静止画は対話なし)
  - 関数名チップは scale に応じてフォントサイズを大きめに (印刷品質)
  - `<title>` 要素で各盾に関数名を埋め込み、SVG ビューアやブラウザ hover で名前を出せる
- **既定値の変更 (破壊的)**:
  - `--focus` 未指定時はファイル先頭関数 + neighbors=all (`magia render <FILE>` だけでピン中心 SVG が出る)
  - 旧 `--fn <NAME>` フラグは `--focus <NAME>` にリネーム (Phase 4.0 と整合)
- 静止画ゴールデンテスト: fixture × neighbors オプションの組み合わせで SVG 一致

### やらないこと

- 動的なピン遷移 (4.1)
- PNG/PDF 出力 (本計画は SVG のみ。ラスタライズは外部ツール経由)
- 印刷用ページ分割 (1ファイルに収まらない規模は 4.5 ワークスペース俯瞰の責務)

## 設計上の判断

### `magia render` を CLI の主役に据える

「ぱっと SVG 1枚」が `magia render <FILE>` で得られるのが、ドキュメント・README 文脈で最も価値がある。
- v1.0 前なので旧 `--fn` は廃止、新形式に統一
- README の「Try it: `magia render src/lib.rs > spell.svg`」が成立する

### 動的UI とレイアウト共有

POSD の「Pull Complexity Downwards」: serve も render も同じ `render_focus_layout` を呼ぶ。
- 引数で出力モードを切替 (`LayoutMode::Static` / `Interactive`)
- Interactive 時のみインタラクションフックを差し込む

## 受け入れ基準

- [ ] `magia render <FILE>` でフォーカス + 周辺リング構造の SVG が stdout に出る
- [ ] `--focus <fn>` で焦点を指定できる
- [ ] `--neighbors none` で旧形式 (フォーカスのみ) 相当の SVG が出る
- [ ] 既存 fixtures でゴールデン SVG が決定論 (3 回走らせて diff なし)
- [ ] `--out <PATH>` でファイル書き出し
- [ ] `cargo test --workspace` / clippy 通過

## 後続候補

- 4.4 で「呼び出し関係を矢印として overlay」する静止画オプション
- 4.5 でワークスペース全体の俯瞰 SVG
- 4.6 で Spell Diff + ピン中心の重ね表示

## 実装ステップ

1. **magia-core**: `LayoutMode` enum (`Static` / `Interactive`) を 4.1 のレイアウト関数に追加
2. **magia-cli/render**: `--focus` / `--neighbors` / `--out` フラグ追加。旧 `--fn` 廃止
3. **テスト**: fixture × neighbors オプション × focus の組み合わせで golden SVG
4. **目視素材**: README 用に推し fixture でピン中心 SVG を生成
5. **Stage 1 品質ゲート**
6. レビュー + 指摘対応
7. 完了処理

## 想定リスク

- **golden SVG の差分頻度**: 4.1 / 4.2 の調整があるたびに golden が動く。`UPDATE_GOLDENS=1` 環境変数で一括更新する仕組みを 4.1 のテスト基盤に入れておく
- **印刷品質との両立**: scale 0.25 の盾でフォントが潰れる場合、`LayoutMode::Static` のときだけフォントスケールを底上げする補正を入れる
