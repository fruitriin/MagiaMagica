# Phase 3.4 — データフロー解析の導入 (Use-Def chains)

## 出典

- `project-docs/magia/phase2plus-notes-v0.1.md` §7.3 (ベルカ式に必要な解析)
- `project-docs/magia/spec-v0.1.md` §5.1 (`data_flow` レイヤー)
- spec v0.2 §4.3 (EdgeLayerData 非対称の Phase 3 再設計方針 — 本計画で実行する)

## 目的

ベルカ式 (3.5) の前提となるデータフロー情報を IR に充填する。
関数内の「値がどこで生まれ、どこで使われるか」を syn ベースの近似で抽出する。

## スコープ

### やること

- `magia-rust` に変数スコープ追跡を追加 (意味解決なしの近似):
  - `let` 束縛 = 値の誕生 (def)、識別子の出現 = 使用 (use)、再代入 = 再定義
  - Operation 単位で「どの変数を def / use するか」を記録
  - シャドーイングは新しい def として扱う。クロージャ内は Phase 3 では追わない
- IR 拡張 (**v1.0 前の破壊的変更**):
  - `DataFlowInfo` を具体化: リング単位の use-def チェーン数・最長チェーン長
  - `EdgeKind::DataFlow` の Edge を生成: 同一関数内で「def したリング → use するリング」
  - **EdgeLayerData の再設計** (spec v0.2 §4.3 の既定方針): `Option<XxxInfo>` 積層構造へ
    破壊的に変更し、`data_volume` を DataFlow 情報として正式化
- transcript への文の差し込み (「データフロー: 変数N個、最長チェーンM」)
- spec v0.3 への追記 (実装結果の追認、Phase 3.0 の文書に増補)

### やらないこと

- 関数間データフロー (Phase 4+ の検討)
- 借用・寿命の解析 (LifetimeInfo は Phase 5+)
- Taint Analysis / Information Flow (notes §7.1 の将来系)

## 設計上の判断

- 近似の精度より**決定論と説明可能性**を優先する (どの構文から def/use を取ったかを
  payload に残し、誤検出をデバッグ可能にする)
- ミッドチルダ式の描画には影響させない (DataFlow Edge は既定で非表示レイヤー。
  描画はベルカ式 3.5 の仕事)

## 受け入れ基準

- [x] fixture 群で def/use 抽出が決定論的に動く (golden)
- [x] DataFlow Edge が生成され、既存レイヤー・レイアウトの出力が不変 (回帰)
- [x] EdgeLayerData の再設計が完了し、round-trip テストが更新されている
- [x] `cargo test --workspace` / clippy 警告0

## 後続

- 3.5 (ベルカ式) がこの DataFlow 情報を力場として描く

## 実装結果メモ (2026-06-11)

- `magia-rust/src/dataflow.rs` 新設: 「候補抽出 (純粋構文) とスコープ解決 (状態機械) の
  分離」方式。関数名・unit variant・定数は**スコープに無いため自然に落ちる** —
  大文字小文字ヒューリスティクスはパターン束縛の unit variant 曖昧性のみに限定できた
- スコープ追跡は RingBuilder の再帰と並走 (別パス二重走査は Operation 添字との
  対応付けが分岐するため不採用)。引数・for パターン・match アーム・if let の束縛は
  `seeds` 引数でリング冒頭に def
- **再代入 = 新 def (再代入が起きたリング由来)** とする設計が想定以上に効いた:
  ループ内 `total += ...` の値が親へ還流する**上り方向の DataFlow Edge** が自然に出る
  (ベルカ式 §14.2「変換 → 消費」の構造がそのまま IR に現れる)
- EdgeLayerData は Option 積層へ破壊的再設計 ([break])。data_volume (f64) 廃止で
  Eq も導出可能になった。spec v0.3 に §4.3 / §5.1 追補として追認済み
- 回帰防御: edges 全走査の4箇所 (layout / midchilda 線描画 / diff 木構築 /
  outward_direction) を ControlFlow フィルタ。SVG ゴールデン・レイアウト
  ベースラインは**全て無変更で通過**。transcript ゴールデンのみ +1 行 (計画どおり)
- レビュー (Stage 2): Critical 0 / Warning 1 (outward_direction のフィルタ漏れ —
  ソート順の暗黙前提を排除) / Suggestion 4 → 全件対応 (doc 補足2件、AuxRing payload
  の直接検証テスト、SigilId 生値依存の明記)
- 既知の近似限界 (spec §5.1 追補に明記): クロージャ・マクロ内・let chains・
  非ローカル代入は追わない。transcript の「変数N個」は再代入分が別チェーンとして
  数えられる (説明可能性を優先した仕様)
