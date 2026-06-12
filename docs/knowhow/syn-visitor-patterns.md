# syn::Visit で AST から情報を集めるパターン

> Phase 1.2 (syn → IR) で確立、Phase 1.3 (AuxRing 再帰展開)・
> Phase 1.4 (call site 抽出と効果判定)・Phase 3.4 (近似データフロー解析)・
> Phase 4.0 (impl メソッド対応の FunctionIndex) で拡張。

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

## 再帰構造の展開: RingBuilder パターン (Phase 1.3)

制御構造 (`if` / `match` / ループ) をネスト構造ごと別ノード (AuxRing) に切り出すときの定型。

```rust
struct RingBuilder<'a> {
    allocator: &'a mut SigilIdAllocator,
    ctx: ParseContext,        // fn_is_unsafe 等を1構造体に集約 (引数膨張の防止)
    sigils: Vec<Sigil>,       // 再帰中に子から push される
    edges: Vec<Edge>,
}

fn build_ring(&mut self, kind, stmts, role, span) -> SigilId {
    let id = self.allocator.allocate();  // 親 ID を先に採番 (子の Edge.source に必要)
    for stmt in stmts { /* 制御構造なら spawn_child で再帰 */ }
    self.sigils.push(/* 親自身は最後に push */);
    id
}
```

ポイント:

- **親 ID 先採番 → 子 push → 親 push → 最後に ID 順 sort**。再帰中の push 順は
  「子が先」になるため、`sort_by_key(|s| s.id)` で「ソース出現順の深さ優先」を回復する。
  ID 採番が決定論的ならソート結果も決定論的
- **非ブロックの式体は statement 化して経路を一本化する**: match のアーム体 `1 => a()` は
  `Stmt::Expr(expr.clone(), None)` に包んで同じ `build_ring` に流す。
  これで `_ => match ...` のような入れ子も特別扱いなしで再帰展開される (clone は Phase 1 では許容)
- **二重計上の防止**: 制御構造を親リング側の Operation (Branch/Match/Loop) にするとき、
  scan するのは条件式・被検査式・イテレータ式のみ。本体ブロックは AuxRing 側が処理するため、
  本体まで scan すると `unsafe` / `?` のフラグが親子で二重計上される
- `clippy::too_many_arguments` 対策: `AuxRingRole` のような役割構造体を呼び出し側で組み立てて
  渡す (anchor/ordinal/label をバラで渡さない)
- **インデックス系の `u32::try_from(..).unwrap_or(u32::MAX)` は禁じ手** (Phase 1.3 レビュー指摘):
  後続フェーズが「存在しない位置」を有効値として参照する無音バグになる。実用上起こらない
  超過なら `expect` で明示的に落とす。センチネルで誤魔化さない (POSD「エラーを存在しないものとして定義」)

## call site 抽出と近似パス解決 (Phase 1.4)

意味解決なし (Phase 1a) で call site を拾うときの定型と落とし穴。

- **`visit_macro` は `Stmt::Macro` と `Expr::Macro` の両方を1フックで捕捉する**。
  `visit_expr_macro` だけだと statement 位置のマクロ (`println!("x");`) を取り逃がす。
  マクロ呼び出しを拾うなら `visit_macro` 一択
- **マクロのトークン列内部は走査されない**: `println!("{}", foo())` の `foo()` は
  syn の visitor が式として降りないため call 抽出から漏れる。マクロは名前ベースの
  白リスト判定 (展開しない) が Phase 1 の割り切り
- **パス前方一致はセグメント境界つきにする**: `path.strip_prefix(prefix)` 後に
  `rest.is_empty() || rest.starts_with("::")` を要求。素の `starts_with` だと
  `std::io` が `std::iox::fake` に誤一致する
- **use 文の機械的展開 (UseMap)**: `UseTree` の Path/Name/Rename/Group を再帰 walk、
  Glob は無視。先頭セグメント名 → フルパスの HashMap を作り、call パスの先頭だけ
  置換する。モジュール境界は無視 (同名 use は後勝ち) で Phase 1a には十分
- **メソッド呼び出しはレシーバ型が分からない**ため解決不能。`.method` 形式で保持して
  pure 扱いに倒す (Phase 1b の ra_ap_hir 導入で再訪)

## クレート選定メモ

- `syn = { version = "2", features = ["full", "visit", "extra-traits"] }`: `full` を入れないと `ItemFn::block` などが取れない
- `proc-macro2 = { version = "1", features = ["span-locations"] }`: feature を入れないと `span.start().line` が取れない
- `quote = "1"`: `ToTokens::to_token_stream().to_string()` でシグネチャを文字列化 (`syn::Signature` には `to_string()` が無い)
- `thiserror = "2"`: 現行 stable に追従。`#[from]` / `#[error(..)]` の挙動は v1 と互換

## 近似データフロー解析 (Phase 3.4)

意味解決なしの syn ベース def/use 抽出の定型 (`crates/magia-rust/src/dataflow.rs`):

- **候補抽出 (純粋構文) とスコープ解決 (状態機械) を分離する**: 識別子の出現は
  単一セグメント・型引数なしのパスを全部候補として集め、let / 引数 / パターン束縛で
  積んだスコープ (`Vec<BTreeMap>` フレームの push/pop) で解決できたものだけ採用する。
  関数名・unit variant・定数はスコープに無いため**自然に落ち**、大文字小文字
  ヒューリスティクスを最小化できる (パターン側の unit variant 曖昧性のみ
  先頭大文字で除外)
- **スコープ追跡は既存の再帰構築 (RingBuilder) と並走させる**: 別パスで AST を
  二重走査すると Operation 添字との対応付けが分岐しやすい。フレーム push は
  build_ring 冒頭、pop は seal (Sigil push) 直後。リング本体に Operation を持たない
  束縛 (引数・for パターン・match アーム・if let) は `seeds` 引数で冒頭にまとめて def
- **解決順序が要**: `let x = x + 1` は uses 解決 → reassign/define の順に処理しないと
  シャドーイングが壊れる
- **再代入 = 再定義の実装**: 変数が見えるフレームの位置はそのまま、レコードだけ
  新リング由来に差し替える。これで「ループ内で変換された値が親へ還流する」上り方向の
  フローが自然に出る (ベルカ式の「変換 → 消費」の根拠)。syn 2 では複合代入は
  `ExprBinary` + `*Assign` 系 `BinOp` (ExprAssignOp は無い)
- クロージャは `visit_expr_closure` を空実装にして再帰を止める (追わない宣言)

## impl メソッド対応の関数索引 (Phase 4.0)

- **ImplItemFn → ItemFn の正規化**: attrs / vis / sig / block は形が共通なので、
  メソッドを `ItemFn` に詰め替えれば後段 (リング構築) を一本化できる
- **qualified 名 (`Foo::bar`) を一意キーの正**とし、素の名前はソース出現順の最初に
  解決するフォールバックにする (同名メソッドの impl 違いを区別する一意キー)。
  impl 文脈は self 型の最後のパスセグメント (`impl fmt::Display for Foo` → `Foo`)
- **「索引と本体は同じ走査範囲」が規約なら walker を1本にする** (1関心1visitor の
  意図的な例外として明記する)。list / parse / serve の3者が同じ walker を共有すれば
  「列挙された名前は必ず再発見できる」規約が一点で守れる (`magia-rust/src/index.rs`)

## 関数間 call graph の近似解決 (Phase 4.2 近接度)

ファイル1本から関数間の呼び出しエッジを取る定型 (`index.rs::function_index_with_calls`):

- **索引とエッジは1パースで取る**: `function_index` と別 API に分けると同一ソースを
  syn で2回パースする。walker (entries + bodies) を1回走らせ、bodies から call site を
  収集して entries と照合する形に統合する
- **呼び出し先の同ファイル解決は3段フォールバック**: qualified 完全一致 →
  **呼び出し元と同じ impl の名前一致** → 定義順先頭の名前一致。`self.method()` は
  レシーバ型が取れないため、同名メソッドが複数 impl にあるときの最有力は
  呼び出し元の impl (レビューで指摘された偽陽性の典型)
- **`Self::x` は呼び出し元の impl_context で置換**してから照合する
- **マクロ (`name!`) はエッジにしない**: UI の定義ジャンプ (resolveCall) は寛容で
  よいが、自動計算 (近接度) では偽陽性を避ける — 同じ「名前照合」でも用途で
  厳しさを変える
- 自己再帰・重複呼び出しはエッジにしない (近接度は回数を見ない)

## span から原文を切り出す (Phase 4.1 呼び出し式表示)

`node.span()` の範囲で元ソースから式を抜き出してユーザーに見せる定型
(`serve.rs` の `call_excerpt`):

- **proc_macro2 の `LineColumn` は 0-based・文字単位**。`start().column` は文字位置
  (inclusive)、**`end().column` は既に最後の文字の直後 (exclusive)**。1-based 規約へは
  どちらも +1 だけで変換が成立する — end にさらに +1 すると1文字余る
- **列はバイトでなく文字で数える**: 行を `chars().collect::<Vec<_>>()` してから
  スライスする。文字列リテラル内の日本語などで `&line[a..b]` は char boundary panic
- **`ExprMethodCall::span()` はレシーバ込みの式全体** — `.map` の召喚印から
  `sigil.layers...map(|role| role.kind)` のチェーン全体が取れる。`OperationPayload.
  source_excerpt` (ToTokens 文字列化) はスペース区切りの正規化形で改行が失われる
  ため、**人に見せる断片は span + 原文切り出しの方が良い**
- **継続行の dedent**: 式は行の途中から始まるため 1 行目は列頭を落とし、2 行目以降は
  共通の先頭空白 (空行は除外して min) を削ると左端が揃う
- **壊れた span は丸めて欠落扱い**: 列の clamp で panic を避け、空になった断片は
  応答に載せない (クライアント側は「無い」として隠す — 半端な式を見せない)

## 関連文書

- `docs/plans/phase1.2-syn-to-ir.md`
- `docs/knowhow/rust-ir-skeleton-pattern.md`
- `project-docs/magia/spec-v0.1.md` §10.3 / `spec-v0.3.md` §5.1 追補

## ファイル横断の call 解決 (Phase 4.5 M2 前段)

- **横断解決は「ローカルで解けなかった残り」にだけかける**: scan を共通化して
  (entries + local_edges + unresolved) を1パースで取り、unresolved のみ
  ワークスペース照合に回す。ローカル勝ちが自然に成立し、二重エッジも出ない
- **段階照合 + 一意性ガード**: ①完全一致 ②末尾2セグメント (Type::method)
  ③末尾1セグメント (トップレベル fn のみ)。**各段で候補がちょうど1つのときだけ**
  採用し、複数候補は次の段に倒さず捨てる (自動計算の偽陽性回避 — 4.2 の
  「マクロをエッジにしない」と同じ思想)。`.method` は横断では解決しない
- **段3をトップレベル関数に限定する理由**: `module::render()` が無関係な
  `Type::render` メソッドに化ける誤接続が高頻度で起きる (render/new/get 等の
  汎用名)。トップレベル限定で精度が大きく上がる
- 検証は**自己ホスティング**が早い: 実リポジトリの /workspace を叩いて
  「自分が知っている呼び出し関係」と突き合わせる (serve.rs → proximity.rs 等)

## メソッドチェーンの抽出 (Phase 4.8)

- **チェーンは最外殻の visit が1回で処理し、default 再帰しない**: visit_expr_method_call
  の default 再帰はレシーバ内の ExprMethodCall を再訪して二重計上する。レシーバ入れ子は
  手動 walk (chain_members) で集め、**引数と基底式だけ** syn::visit::visit_expr で
  手動再帰する — closure 内・match ガード内のチェーンも引数経由で正しく拾える
- **透過リスト**: Try (`?`) / Await / Paren / Reference はチェーンを切らない。
  Cast (`as`) は切る (鎖が分かれるだけで二重計上にはならない — 防御の非対称を許す)
- **実行順 (最奥→外) で CallSite 化**して「後続の参照先は登録済み」を生成側の規約に
  すると、spawn 側は chain_tail (BTreeMap) の lookup だけで Chain edge を張れる
- **新 EdgeKind 追加時の kind フィルタ全掃き** (rust-ir-skeleton の規約) の実例:
  Adjacency (layout) / diff.rs Tree::build / ir_export / 不変条件テスト /
  edge_kind_rank の5箇所。**diff の木は見落としやすい** (レビュー Critical で検出 —
  係留 Edge を増やしたら diff の親子復元も必ず追従)
