# Phase 1.4 — 召喚記号と効果判定 (M4)

## 出典

- `project-docs/magia/spec-v0.1.md` §5.2 (`effects` レイヤー)
- `project-docs/magia/spec-v0.1.md` §6.1.2 (召喚記号)
- `project-docs/magia/spec-v0.1.md` §6.1.3 (効果カテゴリの色相)
- `project-docs/magia/tech-selection-v0.1.md` §2.1 (Phase 1a の近似解決), §3 M4

## 目的

関数本体および AuxRing 内の他関数呼び出し (`call site`) を `SigilKind::SummonGlyph` として抽出し、crate 名先頭セグメントのヒューリスティックで `EffectSet` を付与する。

## スコープ

### やること

- syn の `ExprCall` / `ExprMethodCall` を走査して call site を抽出
- 呼び出し先のパス解決 (Phase 1a 近似):
  - 同ファイル内の `use` 文を機械的に展開してフルパスを近似
  - `use std::collections::HashMap` があれば `HashMap::new()` → `std::collections::HashMap::new`
  - 解決不能なものは元の構文のまま保持し、`EffectSet` は `pure: true` 扱いとする
- 効果カテゴリ判定 (spec §6.1.3 の色相規約に対応):
  - `std::io::*` / `print!*` / `eprint!*` → `io: true`
  - `std::net::*` / `reqwest::*` / `hyper::*` / `tokio::net::*` → `network: true`
  - `sqlx::*` / `diesel::*` / `rusqlite::*` → `db: true`
  - `std::fs::*` / `tokio::fs::*` → `filesystem: true`
  - 既知の crate に該当しない場合 → `pure: true`
- ヒューリスティック表は `magia-rust/src/effects.rs` に分離し、後で拡張可能にする
- 各 SummonGlyph に対し、呼び出し元 Sigil (MainRing または AuxRing) からの `Edge { kind: ControlFlow }` を作る
- ユニットテスト:
  - `println!("x")` → IO 効果 (※ println! はマクロだが Phase 1 は `print` 系マクロ名を白リストで io 判定)
  - `std::fs::read_to_string(...)` → filesystem
  - `reqwest::get(...)` → network
  - `my_helper()` → pure (近似)
  - 同一関数を複数回呼び出した場合の SummonGlyph 重複扱い (現状は呼び出しごとに1つ生成)

### やらないこと

- `ra_ap_hir` 等の意味解決 (tech-selection §2.1 の Phase 1b)
- マクロ展開 (`println!` 以外の任意マクロは無視 or 不明扱い)
- 動的ディスパッチ (`dyn Trait`) の解決
- 呼び出し頻度や引数の型推論

## 設計上の判断

- ヒューリスティック表は **小さく始めて段階拡張** する。最初は spec §6.1.3 の色相 (io / network / db / filesystem / unsafe / pure) に対応する最低限の crate のみ
- マクロは syn の `ExprMacro` から**名前ベースで白リスト判定** (展開しない)。オーナー確定 (2026-06-10): 「作ってみてめんどくさくない範囲で動作確認 (人間目視) しつつ」進める方針。最小白リストから開始:
  - `println!` / `eprintln!` / `print!` / `eprint!` / `dbg!` → io
  - `format!` / `write!` / `writeln!` (引数1つの `format!` は pure と見るのが妥当だが Phase 1 は io 側に倒して様子見) → io
  - 上記以外のマクロは pure 扱い (= 黒記号として表示) で OK。実コードを通したときに違和感が強いものから随時追加する
- `EffectSet.custom` フィールドは Phase 1 では未使用
- 呼び出し先のフルパス解決失敗は**警告ではなくサイレントに pure 扱い**とする (UI 上は黒い記号として表示される)
- SummonGlyph 重複は M5 のレイアウトで merge する余地を残すため、IR 段階では重複させる

## 受け入れ基準

- [x] 5種類のテストケースが全て通る
- [x] AuxRing 内の呼び出しも正しく SummonGlyph 化される
- [x] 効果カテゴリの判定が決定論的 (同じ入力で同じ結果)
- [x] `magia-rust/src/effects.rs` がヒューリスティック表を分離した形で実装されている
- [x] `cargo clippy` 警告0

## 後続

- Phase 1.5 で MainRing / AuxRing / SummonGlyph を平面に配置するレイアウトエンジンを実装

## 実装結果メモ (2026-06-11)

### 設計判断の確定

- **SummonGlyph の内部表現**: `content` に `OperationKind::Call` の Operation を1個持たせ、
  `payload.call_target` に近似解決後のパス、`effects` にヒューリスティック判定結果を載せる
  (Sigil 直下に呼び出し情報用フィールドを増やさない)。glyph の `layers.control_flow` は `None`
- **係留規約**: 制御構造のガード式・被検査式・イテレータ式・アームガード中の call は親リング側、
  本体中の call は AuxRing 側に係留する (二重計上なし)。Edge は所属リング → glyph の
  `ControlFlow`、cardinality 1.0
- **メソッド呼び出し** (`x.read_to_string()`) はレシーバ型が分からないため `.method` 形式で
  保持し pure 扱い (Phase 1b の ra_ap_hir 導入で再訪)
- **マクロ**は `visit_macro` で `Stmt::Macro` / `Expr::Macro` の両方を捕捉。`call_target` は
  `println!` 形式。トークン列内部の call (`println!("{}", foo())` の `foo()`) は走査されず
  抽出漏れする (Phase 1 の既知の限界、summon.rs に明記)
- **ネスト呼び出し** (`outer(inner())`) はそれぞれ独立 glyph として同一リングに平坦化され、
  引数位置の関係は IR 上では失われる
- **ターボフィッシュ** (`HashMap::<K,V>::new`) の型引数は `path_to_string` で意図的に落とす

### レビュー対応 (Stage 2)

- 修正済み: Phase 1.3 由来の `unwrap_or(u32::MAX)` センチネル残留4箇所を `expect` 化
  (branch_count / early_return_count / weight / ordinal)。`lib.rs` の `sigils[0]` 参照に
  MainRing 先頭規約の `debug_assert` 追加。match アームガード中の call の取りこぼしを修正
  (親リング係留 + テスト)。`use Bar as B;` (空 prefix rename) が `::Bar` になるバグを修正
- 先送りなし (Info はコードコメント・本メモへの文書化で対応済み)
- `format!` を io 側に倒す判断 (オーナー確定) は、false positive 報告が出たときの
  再判断材料として Feedback.md に記録

### 効果テーブルの現状 (要拡張ウォッチ)

- `tokio::io` は未登録 (tokio::net / tokio::fs と非対称)。effects.rs に TODO コメントあり。
  実コードで false negative が目立ったら追加する
