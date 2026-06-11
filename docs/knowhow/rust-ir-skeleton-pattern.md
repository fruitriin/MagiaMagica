# Rust IR スケルトンを一気に立てるパターン

> spec §付き設計書を持つプロジェクトで、Phase 1 から全 Phase 分のフィールドを揃えた IR を
> 一括定義するときの定型。Phase 1.1 (MagiaIR) で確立。

## いつ使うか

- 設計書 (spec) に「全関心軸を最初から保持する」原則がある
- Phase 1 で値を埋めるフィールドは少ないが、後方互換のためスキーマは固める必要がある
- 段階的にレイヤーが増えていく可視化・分析ツール、コンパイラ IR、設計ドキュメント駆動の中間表現

## 構成パターン

### 1. サブモジュール分割

単一の `ir.rs` に全型をフラットに置くと 400 行を超え見通しが悪い。spec §4.2 の論理ブロックに合わせて分割:

```
ir/
├── mod.rs          # pub mod 宣言 + pub use re-export
├── graph.rs        # トップレベル (Graph, Module, ID, メタデータ)
├── sigil.rs        # 中核ノード型
├── operation.rs    # 内部処理単位
├── edge.rs         # 接続
├── layers.rs       # 多次元レイヤー情報
└── source.rs       # ソース位置
```

`mod.rs` は薄く保ち、すべての公開型を `pub use ...;` で1点に集約すると呼び出し側が `magia_core::ir::{...}` だけで完結する。

### 2. 全 struct に `#[serde(default)]`

```rust
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayerData { /* ... */ }
```

- 新フィールド追加で既存の JSON が壊れない
- 空 JSON `{}` から `MagiaGraph::default()` 等価のオブジェクトが復元できる
- `deserialize_minimal_json_uses_defaults` 相当のテストで自動化

### 3. `deny_unknown_fields` は意図的に付けない

新しい Phase のフィールドを古いバイナリで無視して読めるようにするため。**ドキュメントコメントに意図を明記**しないと、レビュアーから「セキュリティ上の懸念」と指摘されやすい。

### 4. enum の `Default` は `#[derive(Default)] + #[default]`

```rust
#[derive(Debug, Default, Clone, Copy, ...)]
pub enum SigilKind {
    #[default]
    MainRing,
    AuxRing,
    /* ... */
}
```

手書きの `impl Default for SigilKind { fn default() -> Self { Self::MainRing } }` は clippy `derivable_impls` で警告される。

### 5. ID は newtype + `#[serde(transparent)]`

```rust
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, ..., Serialize, Deserialize)]
#[serde(transparent)]
pub struct SigilId(pub u32);
```

- JSON では数値リテラルとしてシリアライズされ、IR が読みやすい
- `pub u32` を直接アクセスできるのは便利だが、採番ロジックを書く段階 (Phase 1.2 以降) で `Allocator` パターンへ移行する前提を残しておく

### 6. `f64` を含む struct は `Eq` を派生しない

`Option<f64>` を含むと `derive(Eq)` がコンパイルエラー (E0277)。`PartialEq` のみで派生。
追加で、コメントに `f64 のため Eq は派生しない` と添えると後続者の手戻りを防げる。

## 品質ゲートで起きやすい clippy 警告と対処

### `derivable_impls`

`impl Default for Enum` を手書きすると指摘される。`#[derive(Default)] + #[default]` でリプレース。

### `struct_excessive_bools` (4 個以上の bool)

spec が直接定義する直交フラグ集合 (例: `EffectSet { pure, io, network, db, filesystem, mutation, unsafe_block }`) は**意味的に正しい設計**なので、局所的に allow:

```rust
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default, ...)]
pub struct EffectSet { /* ... */ }
```

理由コメント (spec 参照) を必ず添える。

### `doc_nested_refdefs`

`mod.rs` で `- [`graph`]: 説明` のような markdown は intra-doc link 扱いされて警告。`[`graph`][graph]` 形式にする (rust-1.94 で発生)。

### `doc_lazy_continuation` (doc list item without indentation)

doc コメントの継続行を `+` や `-` で始めると箇条書きの続きと誤認されて警告。
「角度 = A + B」のような数式をコメントで改行するときは、行頭に演算子を置かない
言い回し (「A」「B」の和、等) に変える (Phase 1.5 で発生)。

### `doc_markdown`

カタカナ・固有名詞 (`MagiaMagica`, `MainRing`, `Mystical`) を多用するプロジェクトでは workspace 全体で `doc_markdown = "allow"` (cargo-workspace-bootstrap 既収録)。

## 必須テスト 3 種

1. **空インスタンス生成**: `Type::default()` が `Default` 派生で破壊なく生成できる
2. **空 JSON round-trip**: `serde_json::to_string(&Type::default()) → from_str` が同値
3. **Phase 1 のレイヤーのみ充填した実例の round-trip**: 実装の意図を表す fixture を組み立てて round-trip
4. (任意) **`{}` からのデシリアライズ**: 後方互換性の最小テスト
5. (任意) **同一値の連続シリアライズが等しい**: HashMap を追加した時に壊れるので、テスト名は `same_value_serializes_twice_identically` のように現実に即した名前にする

## レビュアーから刺さりやすい指摘

- センチネル値 (`u32 = 0` を「未取得」扱い) → `Option<u32>` を推奨される。**最初から Option にする**
- 集計単位の曖昧さ (`branch_count: u32` だけだと if 1 個 = 1 か match arm 1 個 = 1 か不明) → コメントで集計ルールを明示
- `pub` フィールドが直書きできる ID → 採番のアロケータ導入の判断を Phase 1.2 計画にメモする
- `deny_unknown_fields` を**付けない**意図のコメント不在

## 後続 Phase で IR に新しい Edge 種別を足すとき (Phase 3.4)

- **既存の edges 全走査箇所を先に grep して kind フィルタを入れる**。MagiaMagica では
  layout の隣接構築・レンダラの線描画・diff の木構築の3箇所が「edges = ControlFlow 木」
  を暗黙に前提していた。フィルタを入れ忘れると多重親・余計な線・diff 子ノード混入が
  無言で起きる
- テストの不変条件 (「各子は親と1本の Edge を持つ」等) も kind 限定に更新が要る
- 複数 kind の Edge ソートは `(kind 序列, target, source)` のタプルキーにし、
  **既存 kind を序列 0** にすると既存出力 (スナップショット) が不変のまま決定論を保てる

## 関連文書

- `docs/plans/phase1.1-ir-skeleton.md`
- `docs/knowhow/rust-cargo-workspace-bootstrap.md`
- `project-docs/magia/spec-v0.1.md` §4 (中間表現スキーマ) / `spec-v0.3.md` §4.3 追補
