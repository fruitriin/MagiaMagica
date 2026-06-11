# Phase 3.5 — ベルカ式レンダラ (三角力場)

## 出典

- `project-docs/magia/phase2plus-notes-v0.1.md` §7.3〜7.6 (ベルカ式 = Data Flow Graph 可視化)
- `project-docs/magia/appendix/` Appendix A (描画様式カタログ)
- Phase 3.4 のデータフロー IR

## 目的

`RenderStyle::Belka` の `unimplemented!` を解除し、データの流れを「3点と力場」で
描く第2の式を実装する。ミッドチルダ式と**同じ IR の別射影**であることが核心
(notes §7.4: 並置によって初めて見えるスメルがある)。

## スコープ

### やること

- ベルカ式レイアウト (`layout` の Belka バリアント or `layout_belka`):
  - 関数内の主要な値の流れを「生成 (def 群) / 変換 (中間 use-def) / 消費 (戻り値・副作用)」の
    3 極に集約し、三角形の頂点に配置する
  - 頂点の重み = 含まれる Operation 数。重みの偏りが三角形の歪みとして現れる
- 力場のレンダリング (notes §7.6): 各頂点を中心とした放射状グラデーションの
  アディティブ合成 (`<radialGradient>` + opacity 重ね)。場の濃淡 = データフロー量
- CLI: `magia render --style belka` (`--style` オプションの新設、既定は midchilda)
- serve: スタイル切替 (クエリ `?style=belka`、パレットにトグル)
- 自動推奨の最小形 (notes §7.5): `(A, B) -> A` 型シグネチャ (Reducer 形) を検出したら
  「構造的にはベルカ式が適合します」と CLI 出力に一行提案 (自動切替はしない)
- ゴールデンテスト + 自己ホスティング素材の生成 (オーナー意匠判定)

### やらないこと

- 逆方向のフォールバック提案 (ベルカ式指定だが3点関係が不明瞭なケース) — Phase 3 振り返りで判断 (spec v0.3 §14.3)
- 夜天の書式 (Phase 6+)
- ミッドチルダ式とベルカ式の同時並置ビュー (Phase 3 振り返り後の計画)
- 力場強度への呼び出し頻度反映 (動的解析、Phase 5)

## 設計上の判断

- 「3点」の抽出はヒューリスティック (def 密度クラスタリングの最小形) から始め、
  目視判定で調整する (Phase 1.6〜1.8 と同じ「動かしてから調整」方針)
- 決定論・乱数なしはミッドチルダ式と同じ規約

## 受け入れ基準

- [x] `--style belka` で三角力場の SVG が出る (XML valid・決定論的)
- [x] Reducer 形 fixture で式の推奨メッセージが出る
- [x] ミッドチルダ式の既存出力が不変 (回帰)
- [x] 自己ホスティング素材を生成しオーナーに意匠判定を依頼
- [x] `cargo test --workspace` / clippy 警告0

## 後続

- Phase 3 振り返りで二式並置ビュー・レイヤー差分分解の計画を判断

## 実装結果メモ (2026-06-11)

- `render/belka.rs` 新設: 射影モデル (3極分類 + フロー集計) と描画をセクション分離。
  極の色は palette に BELKA_* 3色 (空色=生成 / 琥珀=変換 / 臙脂=消費)
- **フロー集計は anchor に沿った実行順走査**: 当初の深さ優先 (SigilId 順) では
  ループ内再定義 → 末尾戻り値の「変換 → 消費」還流を取り逃がした。**目視確認で発見**
  (loop_accumulate の矢印欠落) — 実物レンダリング駆動の開発ループが再び機能
- MainRing 末尾の Operation (暗黙の戻り値) は is_tail フラグで消費極に分類
  (Return kind ではないため)
- ドット散布は phyllotaxis (個数によらず決定論的)。力場は radialGradient
  (gradientUnits 既定の objectBoundingBox で円中心基準 — 確認済みの旨コメント)
- serve は両式を毎回レンダリングして state に同梱、クライアントは表示切替のみ
  (?style= URL 同期、preview E2E でトグル・リロード復元を確認)
- ベルカ式は FilterSpec/--layers 未対応 — CLI で明示エラー (黙って無視しない)
- 意匠判定素材3点送付済み (loop_accumulate / 自己ホスティング measure / Reducer 形)。
  **意匠判定待ち**。既知の調整候補: 引数は Operation を持たないため Reducer 形で
  生成極が空円になる (「値は外から来る」表現とも読める — 判定次第で調整)
- レビュー (Stage 2): Critical 0 / Warning 2 / Suggestion 4 → 全件対応。
  W-2 は u32::MAX センチネルの**4度目の再発** (新規モジュール執筆時) — expect に修正
- 計画スコープ外で見送り: ベルカ式の diff 強調 (render_diff は midchilda のみ)、
  逆方向フォールバック提案 (計画どおり Phase 3 振り返りで判断)
