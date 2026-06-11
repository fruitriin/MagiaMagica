# 第2のレンダリング様式 (同じ IR の別射影) を足すパターン

> Phase 3.5 (ベルカ式 = データフロー三角力場) で確立。既存様式 (ミッドチルダ式) と
> 同じ IR を別の軸で描く様式を追加するときの定型。

## 発見した知見

### 1. 様式の語彙は enum + FromStr + SELECTABLE で一元化

`RenderStyle` に `as_str` / `FromStr` / `SELECTABLE` 定数を生やし、
CLI (`--style`) / serve (`?style=`) / テストで共有する
(`LayerName`・`EffectCategory` と同じ mini-dsl-pattern の適用)。
**未実装バリアント (Yagami) は `SELECTABLE` に含めず候補一覧にも出さない** —
選べないものを提示しない。

### 2. 旧様式のインフラを使わないことを「黙殺」にしない

新様式が `LayoutResult` や `FilterSpec` を使わない場合、dispatch で引数を
無視するだけだと利用者には「効かないオプション」に見える。
使わない理由を doc に書き、**CLI 側で併用を明示エラー**にする
(「黙って無視しない」原則の様式版):
`ベルカ式は --layers / --filter に未対応です (レイヤー語彙はミッドチルダ式専用)`。

### 3. 射影モデルと描画をセクション分離する

「IR → 極分類 + フロー集計 (モデル)」と「モデル → SVG (描画)」を同一ファイル内でも
セクションで分け、単体テストはモデルだけを直接叩く (分類の優先順位・三角の歪み)。

### 4. フロー集計の走査順は実行順 (anchor インターリーブ)

「深さ優先 (SigilId 順)」は実行順と食い違う: ループ本体での再定義を末尾の戻り値より
**後に**見てしまい、「変換 → 消費」の還流フローを取り逃がす。
anchor に沿ってリングの Operation と子リングをインターリーブする実行順走査が要る
(ループ本体は1回だけ辿る静的近似)。
この欠陥は**目視確認で発見した** (loop_accumulate で還流の矢印が出ない) —
視覚アルゴリズムは「実物レンダリング → 自分の目 → 修正」のループが最速
(Phase 1.8 の教訓の再確認)。

### 5. 暗黙の戻り値は「末尾フラグ」で扱う

MainRing 末尾の Operation は Rust の暗黙の戻り値だが `OperationKind::Return` では
ない (semicolon の有無は IR に残らない)。分類関数に `is_tail` フラグを渡して
消費極に倒す。

### 6. 円内の均一散布は phyllotaxis

N 個の点を円内へ均一に置くのは `r = R * sqrt((i+0.5)/n)`, `θ = i × 黄金角` の
ひまわり配置が個数によらず決定論的で見栄えが安定する (黄金角は無理数だが定数)。

### 7. serve の多様式対応は「全様式を毎回レンダリング + クライアント表示切替」

サーバが全様式の SVG を state に同梱し、クライアントは表示する SVG を選ぶだけに
すると最小になる: SSE の再取得・サーバ側の様式状態・様式パラメータ付き
再レンダリング要求が全部不要。様式は URL クエリ (`?style=belka`) に同期して
リロード・共有で再現できる。

## プロジェクトへの適用

- `crates/magia-core/src/render/belka.rs` — 極モデル + 三角力場 SVG
- `crates/magia-core/src/render/mod.rs` — RenderStyle の語彙と dispatch
- `crates/magia-cli/src/main.rs` の `run_render` — --style、併用エラー、Reducer 形ヒント
- `crates/magia-cli/src/serve.rs` — 両式同時レンダリング + トグル
- `crates/magia-rust/tests/belka_golden.rs` — golden + 実行順フローの検証

## 注意点・制約

- 引数 (関数パラメータ) は Operation を持たないため、生成極にドットとして
  現れない (Reducer 形では生成極が空円になる)。「値は外から来る」の表現とも
  読めるが、意匠判定の結果次第で調整余地あり
- 数値・XML エスケープの規約 (`num` / `escape_xml`) は midchilda から
  `pub(crate)` で共有 — 出力数値の規約は様式をまたいで統一する

## 参照

- `project-docs/magia/spec-v0.3.md` §14 (二大可視化軸・ベルカ式の契約)
- `docs/knowhow/svg-deterministic-rendering.md` (SVG 決定論の一般則)
- `docs/knowhow/mini-dsl-pattern.md` (語彙一元化の一般形)
- `docs/knowhow/deterministic-layout-pattern.md` (目視判定駆動の調整方針)
