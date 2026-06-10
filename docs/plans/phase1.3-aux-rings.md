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

## 受け入れ基準

- [ ] 5種類のテストケースが全て通る
- [ ] 入れ子の制御構造で `SigilId` の衝突が起きない
- [ ] `Edge.kind == ControlFlow` の数 = AuxRing 数 (各 AuxRing は MainRing と1本の Edge を持つ)
- [ ] JSON 出力が決定論的
- [ ] `cargo clippy` 警告0

## 後続

- Phase 1.4 で AuxRing 内の関数呼び出しから召喚記号を生成
