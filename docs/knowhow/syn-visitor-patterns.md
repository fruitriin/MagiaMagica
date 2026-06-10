# syn::Visit で AST から情報を集めるパターン

> Phase 1.2 (syn → IR) で確立。Phase 1.3 以降の `if` / `match` / `loop` 解析、
> Phase 1.4 の call site 抽出でも同じパターンを再利用する。

## 基本形: 単一関心の Visitor を関数スコープに閉じ込める

```rust
fn count_await_points(item_fn: &ItemFn) -> u32 {
    struct Counter { count: u32 }
    impl<'ast> Visit<'ast> for Counter {
        fn visit_expr_await(&mut self, node: &'ast syn::ExprAwait) {
            self.count += 1;
            syn::visit::visit_expr_await(self, node);  // 必ず再帰
        }
    }
    let mut counter = Counter { count: 0 };
    counter.visit_block(&item_fn.block);
    counter.count
}
```

ポイント:

- 1関心1 visitor。`Counter`, `TryFinder`, `UnsafeFinder` のように単機能に絞る
- 関数内に閉じ込めれば `pub` 露出ゼロ (POSD「情報隠蔽」)
- **再帰呼び出しを忘れない**: `syn::visit::visit_*` を呼ばないとサブツリーが走査されない。バグの定番

## 同じ AST を2回走査しそうになったら集約せよ

NG パターン:
```rust
let kind = classify_statement(stmt);   // visitor 1
let early_return = is_early_return(stmt);  // visitor 2 (同じツリーをまた走る)
let has_unsafe = statement_contains_unsafe(stmt);  // visitor 3
```

GOOD パターン:
```rust
struct StatementScan { kind: OperationKind, early_return: bool, has_unsafe: bool }

fn scan_statement(stmt: &Stmt) -> StatementScan {
    let is_return_stmt = matches!(stmt, Stmt::Expr(Expr::Return(_), _));
    let mut visitor = StatementVisitor::default();
    visitor.visit_stmt(stmt);
    StatementScan { /* 1回のスキャンで全部出す */ }
}
```

Phase 1.2 では3〜4機能でも分かれていたが、Phase 1.3 で `if` / `match` が増えると visitor の本数が指数的に増えうる。**N 個の関心事を 1 visitor に統合する**設計に最初から寄せる。

## list/parse の API 規約: 探索範囲を揃える

`list_functions` (名前列挙) と `parse_function` (実体取得) で別々の visitor を使う場合、両者の再帰方針 (`mod` 内に降りるか / `impl` 内に降りるか) を**明示的に揃える**。揃わないと「list には載るが parse できない名前」が出て API として矛盾する。

規約をコードコメントとテストで残す:
```rust
/// 規約: list_functions が返す任意の名前は parse_function で必ず発見できる
```
```rust
#[test]
fn listed_names_are_all_parseable() {
    for name in list_functions(src).unwrap() {
        parse_function(src, name).unwrap();
    }
}
```

これは POSD「複雑性を下に押し下げる」の好例。利用者が「あ、これは mod 内だから別 API を使わないと」と分岐する形は避ける。

## ID 採番は `Allocator` パターンに閉じ込める

```rust
pub(crate) struct SigilIdAllocator { next: u32 }
impl SigilIdAllocator {
    pub(crate) fn allocate(&mut self) -> SigilId {
        let id = SigilId(self.next);
        self.next += 1;
        id
    }
}
```

- IR 側で `pub struct SigilId(pub u32)` と公開していても、採番はアロケータでのみ行う規約にすると単調増加・一意性が守れる
- 乱数を使わないので決定論的 (spec §6.1.4 への布石)

## lifetime の落とし穴: `'src` と `'ast` を混ぜない

```rust
struct FunctionRefCollector<'src> {
    target: String,
    found: Option<&'src ItemFn>,
}
impl<'ast> Visit<'ast> for FunctionRefCollector<'ast> { /* ... */ }
```

`'src` と `'ast` は事実上同じ lifetime に単一化されるが、命名が分かれていると読者が「別物」と誤読する。**Visit の対象になる struct の lifetime は `'ast` で統一する** ことで読みやすくなる。

## thiserror + 候補提示

```rust
#[derive(Debug, Error)]
pub enum Error {
    #[error("Rust 構文エラー: {0}")]
    Syntax(#[from] syn::Error),

    #[error(
        "関数 `{name}` が見つかりません (候補: {})",
        if candidates.is_empty() { "なし".to_string() } else { candidates.join(", ") }
    )]
    FunctionNotFound { name: String, candidates: Vec<String> },
}
```

- `#[error(..., expr)]` で条件分岐を埋め込める
- `#[from] syn::Error` で `?` 演算子経由の伝播が無痛
- ユーザー向けのエラーは**候補を載せる**だけで体感が大きく変わる (POSD: 「エラーから学べる」)

## クレート選定メモ

- `syn = { version = "2", features = ["full", "visit", "extra-traits"] }`: `full` を入れないと `ItemFn::block` などが取れない
- `proc-macro2 = { version = "1", features = ["span-locations"] }`: feature を入れないと `span.start().line` が取れない
- `quote = "1"`: `ToTokens::to_token_stream().to_string()` でシグネチャを文字列化 (`syn::Signature` には `to_string()` が無い)
- `thiserror = "2"`: 現行 stable に追従。`#[from]` / `#[error(..)]` の挙動は v1 と互換

## 関連文書

- `docs/plans/phase1.2-syn-to-ir.md`
- `docs/knowhow/rust-ir-skeleton-pattern.md`
- `project-docs/magia/spec-v0.1.md` §10.3
