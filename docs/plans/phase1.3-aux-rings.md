# Phase 1.3 — 制御構造の AuxRing 化 (M3)

## 出典

- `project-docs/magia/spec-v0.1.md` §5.2 (`control_flow` レイヤー)
- `project-docs/magia/spec-v0.1.md` §6.1.2 (補助リング)
- `project-docs/magia/tech-selection-v0.1.md` §3 M3

## 目的

`magia-rust` のアダプタを拡張し、関数内の制御構造 (if/else, match, loop/while/for) を `SigilKind::AuxRing` として抽出する。MainRing と AuxRing は `EdgeKind::ControlFlow` の `Edge` で接続する。

## スコープ

### やること

- syn の `Visit` を拡張し、以下を AuxRing に変換:
  - `if expr { ... } else if ... else { ... }` — 分岐先ごとに AuxRing 1つ
  - `match expr { arm => ... }` — アームごとに AuxRing 1つ
  - `for / while / loop` — ループ本体を AuxRing 1つ (`OperationKind::Loop` を MainRing 側に置く)
- `LayerData.control_flow` に Phase 1 で必要な情報を格納 (分岐種別、ループ種別、入口/出口の Operation 位置)
- 早期リターン (`return` / `?`) を `OperationKind::Return` として AuxRing と MainRing の両方で正しく扱う
- 入れ子は**まず深さ制限なく**再帰的に展開する (各 AuxRing が自身の `Sigil` を持つ)
  - Phase 1.6 でレンダリングしてみて視覚破綻するようなら、深さ制限の導入を Phase 1.6 完了時の振り返りで判断する (オーナー確定: 「動かしてから調整」方針)
- AuxRing → MainRing の `Edge` を `cardinality` 付きで生成
- ユニットテスト:
  - `if a { x } else { y }` → 2つの AuxRing と 2本の Edge
  - `match x { 1 => a, 2 => b, _ => c }` → 3つの AuxRing
  - `for i in 0..10 { ... }` → 1つの Loop AuxRing
  - `if let Some(x) = ... { ... }` → 1つの AuxRing
  - 入れ子 (`if { match { ... } }`) → AuxRing が正しく親子関係を持つ

### やらないこと

- 召喚記号 (関数呼び出し) — M4
- データフロー解析 (Phase 3, ベルカ式)
- マクロ展開後の構造

## 設計上の判断

- AuxRing も独立した `SigilId` を持つフラットな配列で保持する。親子関係は `Edge` で表現 (spec §4.2 の `Module.edges`)
- レイアウトでの「親 MainRing の周囲に配置」は M5 のレイアウトエンジンで `Edge` を辿って決める
- `if-else if-else` の連鎖は左から順に AuxRing を生成し、末尾の `else` も AuxRing 化する
- `?` 演算子は内部的には `match` 相当だが、Phase 1 では構文上の `?` を専用フラグで扱い、巨大な AuxRing を作らない (spec §6.1.3 の「早期リターン記号」のため)

### Phase 1.2 レビューからの持ち越し (Info)

- **`ParseContext` パターンの導入を検討**: Phase 1.2 では `fn_is_unsafe: bool` を `statement_to_operation` に直接渡しているが、AuxRing が増えると引数が膨れる。`ParseContext { fn_is_unsafe, current_depth, .. }` のような構造体にまとめると、Phase 1.4 (call site) / Phase 1.5 で追加コンテキストが増えたときの呼び出し連鎖が崩れない。POSD「複雑性を下に押し下げる」観点でも妥当
- **`OperationKind::Await` / `Yield` / `Throw` の扱い**: Phase 1.2 ではトップレベル statement を全て `Compute` / `Return` で済ませているが、AuxRing 化に合わせて Await を独立 OperationKind に持ち上げるか、ConcurrencyInfo に統合したままにするかを判断する

## 受け入れ基準

- [x] 5種類のテストケースが全て通る
- [x] 入れ子の制御構造で `SigilId` の衝突が起きない
- [x] `Edge.kind == ControlFlow` の数 = AuxRing 数 (各 AuxRing は MainRing と1本の Edge を持つ)
- [x] JSON 出力が決定論的
- [x] `cargo clippy` 警告0

## 後続

- Phase 1.4 で AuxRing 内の関数呼び出しから召喚記号を生成

## 実装結果メモ (2026-06-11)

### 設計判断の確定

- **`LayerData.control_flow` の「分岐種別・ループ種別・入口/出口の Operation 位置」**は `ControlFlowInfo.role: Option<AuxRingRole>` として実装した。`AuxRingRole { kind: AuxRingKind, anchor_operation: u32, ordinal: u32, label: Option<String> }`。制御構造は親リングの `content` 上で常に1個の Operation を占めるため、**入口と出口は `anchor_operation` の1点で表現できる** (計画の「入口/出口」は同一値に縮退)
- **Edge の方向は 親リング → AuxRing** (source = 親)。制御フローが分岐へ流れ込む向きと一致させた。`Edge.cardinality` = AuxRing の Operation 数 (空ブロックは最低 1.0)
- **`Sigil.cardinality.weight` = Operation 数**を MainRing にも一律適用 (Phase 1.2 では default 0.0 だった。v1.0 前の破壊的変更)
- **`ParseContext` 導入** (Phase 1.2 持ち越し対応): `fn_is_unsafe` のみ。depth は YAGNI で見送り
- **`OperationKind::Await`/`Yield`/`Throw` の持ち上げは見送り** (Phase 1.2 持ち越し判断): `ConcurrencyInfo.await_points` 集約を維持し、Operation 単位への展開は Phase 1.6 のレンダリングで必要になったときに判断する
- 計画の「AuxRing 用の式単位 Operation 変換ヘルパー」は不要化: 非ブロックのアーム体 (`1 => a()`) を `Stmt::Expr(expr.clone(), None)` で statement 化し、`build_ring` の再帰経路に一本化した。これにより `_ => match ...` の入れ子も特別扱いなしで展開される

### スコープの境界 (明示)

- `let x = if ... { .. }` のような**式の内側**の制御構造は切り出さず Compute 1個に畳む (回帰テスト `let_binding_with_if_stays_compute` で固定)。Phase 1.6 の視覚検証で必要性を再評価
- match の**アームガード** (`1 if cond => ...`) は通常アームと同じ扱い (ガード式は scan されない)。spec-v0.2 を起こすときに扱いを明記する

### レビュー対応 (Stage 2)

- 修正済み: `spawn_child` の子リング探索を `sigils.last()` + `debug_assert` に変更 (O(depth²) 回避) / `anchor` の `u32::MAX` サイレントフォールバックを `expect` に変更 (Phase 1.5 が「存在しない位置」を参照する無音バグの予防) / `if let ... else`・1アーム match・`let x = if` のテスト追加
- 先送り (Info、Phase 1.5〜2 で確認):
  - `AuxRingKind::LoopBody(LoopKind)` の serde 表現だけがオブジェクト形式 (`{"LoopBody":"For"}`)。Phase 2 dev-server の JSON コンシューマー実装前に `#[serde(tag)]` 等での統一を検討
  - `anchor_operation` は Edge と親 content から導出可能な情報の直接保持。content の並び替えが起きる変更では同期に注意 (Feedback.md にも記録)
  - `source_span` の column は意図的に `None` (Phase 1.2 からの近似継続)
