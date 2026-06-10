# Mystical CI Appendix A: 描画様式カタログ v0.1

> **本文書の位置づけ**
> 本文書は『Mystical CI 仕様書 v0.1』および『Mystical CI Phase 2 以降の議論ノート v0.1』の補遺 A として、描画様式 (RenderStyle) の体系を詳細にカタログ化したものである。本ツールが扱う「式 (Style)」の網羅的な分類と、各式の技術的対応、自動推奨ヒューリスティクスを記述する。
>
> 本文書は仕様書本体ではなく**カタログ**として位置づけられ、新しい式の発見に伴って継続的に拡張される。

---

## 1. 二軸分類モデル

### 1.1 一軸目: 対称性のオーダー

魔法陣の図像が持つ回転対称性のオーダー (n-fold symmetry) を一軸目とする。

```rust
enum SymmetryOrder {
    Two,     // 2-fold: 二元対立、入出力対
    Three,   // 3-fold: 三項関係
    Four,    // 4-fold: 四象限、四方位
    Five,    // 5-fold: 五要素契約
    Six,     // 6-fold: 雪片、ヘキサゴン
    Eight,   // 8-fold: 八方位、全周
    Higher,  // それ以上の対称性 (パイプライン化)
}
```

### 1.2 二軸目: 表現様式

同じ対称性のオーダーでも、複数の表現様式が存在する。これを二軸目とする。

```rust
enum ExpressionMode {
    Convergent,   // 中心に向かう力場
    Divergent,    // 中心から外へ放出する力場
    Orthogonal,   // 軸で区切る離散分割
    Organic,      // 連続的に絡みつく伝播
    Balanced,     // 全方位等価な配置
}
```

### 1.3 分類の例示

各式は (Order, Mode) の組み合わせとして表現される:

| 式 | Order | Mode | 由来 |
|---|---|---|---|
| ベルカ式 | Three | Convergent | リリカルなのは |
| うみねこ式 | Four | Orthogonal | うみねこのなくころに |
| Vesperia式 | Four | Organic | テイルズオブヴェスペリア |
| ミッドチルダ式 (簡略) | Four | Balanced | リリカルなのは |
| ミッドチルダ式 (八芒星) | Eight | Balanced | リリカルなのは |
| Alchemy式 | Five | Balanced | 鋼の錬金術師 |
| 黒執事式 | Five | Divergent | 黒執事 |
| RWBY式 | Six | Divergent (fractal) | RWBY |
| Fate式 | Eight | Convergent (+ core) | Fate シリーズ |
| 夜天の書式 | Meta | Meta-recursive | リリカルなのは StrikerS |

### 1.4 偶奇による性格差

偶数オーダー (2, 4, 6, 8) は反転対称性 (mirror symmetry) を持ち、「対」を作りやすい構造であるため、入出力、対称分割、座標系、並列性を扱うのに向く。

奇数オーダー (3, 5) は反転対称性を持たず、循環的・非対称な変換に向く。関係性駆動、契約、変換チェーンを扱うのに向く。

これは群論的に C_n 群の構造的性質がそのまま「扱えるドメイン」を決めている現象として整理できる。

---

## 2. 各式の詳細カタログ

### 2.1 ベルカ式 (Three, Convergent)

**図像的特徴**: 三つの円または三角形を三点に配置し、結んで大きな三角形を作る。中央に小さな収束点 (中心円) を持ち、そこに変換器が降ろされる。

**構造**: 3点+1中心 (合計4点)。3点が等格で、中心が変換を担う。

**表現する関係性**: 三項変換。3つの入力点が中央で統合され、新しい結果が生成される。

**技術ドメイン**:
- State + Action + Reducer (Redux, Elm Architecture)
- Model + View + Controller (MVC, MVVM)
- Port + Adapter + Domain (ヘキサゴナルアーキテクチャ)
- Command + Event + State (CQRS, Event Sourcing)
- Producer + Channel + Consumer (Actor model)
- Test + Spec + Implementation (TDD/BDD)

**必要な解析**: Data Flow Analysis (Use-Def chains, 関数間解析)

**推奨条件 (自動判定)**:
- 関数シグネチャが `(A, B) -> C` の形で、A と C が同じ型
- 構造体に3つの責務フィールド
- ディレクトリに `commands/`, `events/`, `states/` が並存
- 3つのトレイトが相互参照
- 3者のチャネル形成

**実装 Phase**: Phase 3

### 2.2 うみねこ式 (Four, Orthogonal)

**図像的特徴**: 二重円の内側に十字を引いて4象限に分割。各象限にカテゴリ記号を配置。中央に小さな結節点。

**構造**: 4つの離散セクション + 中心。

**表現する関係性**: 排他的なカテゴリ分類。「世界は4つの離散カテゴリに分かれている」という宣言。

**技術ドメイン**:
- 4-valued logic (真/偽/不明/矛盾)
- HTTP ステータス分類 (2xx/3xx/4xx/5xx)
- テスト結果 (pass/fail/skip/error)
- Eisenhower Matrix (重要度 × 緊急度)
- enum / sum type / tagged union (4バリアント)
- 監査ログのトリアージ

**必要な解析**: AST 解析、enum/match の網羅性チェック

**推奨条件**:
- `enum` が4バリアントを持つ
- `match` 文が4つの腕を持つ
- 4つの並列な `if/else if/else if/else` 分岐
- 状態を4つに分類する構造体

**実装 Phase**: Phase 4

### 2.3 Vesperia式 (Four, Organic)

**図像的特徴**: 4方向に円が配置され、内側に向かって有機的な装飾 (葉、ツル、羽根) が伸びる。中心に小さな円。

**構造**: 4つの起点 + 連続的な影響伝播。

**表現する関係性**: 横断的関心事 (Cross-Cutting Concerns)。4つの起点から発される影響が、装飾的に全体に絡みつく。

**技術ドメイン**:
- AOP (Aspect-Oriented Programming)
- React の Context, useEffect 連鎖
- Pub/Sub の多チャネル放送
- RxJS の Subject ネットワーク
- Middleware chain (Express, Koa)
- ログ、認証、トランザクション、エラーハンドリング

**必要な解析**: 注釈 (アノテーション/デコレータ) 解析、middleware 解決

**推奨条件**:
- `@Aspect`, `@Around` 等のアノテーション多用
- React で `useContext` + `useEffect` の連鎖
- middleware の長いチェーン
- 4つの「横断的関心事」モジュールの存在

**実装 Phase**: Phase 4

### 2.4 ミッドチルダ式 (Four/Eight, Balanced)

**図像的特徴**: 均等な円の多重連環、または内接する正四角形 (簡略版)。または、正方形を45度ずらして重ねた八芒星 (本格版)。各頂点にギリシャ文字や数学記号。

**構造**: 4頂点+中心、または8頂点+中心。直交軸が支配的。

**表現する関係性**: 直交基底分解、座標系。

**技術ドメイン**:
- 関数空間の基底分解
- 構造化された OOP の階層
- Trait + Impl の多重実装
- レイヤードアーキテクチャ (Presentation/Application/Domain/Infrastructure)
- 関数型のコンビネータ合成

**バリエーション**:
- `MidchildaConcentric`: 均等な円の多重連環
- `MidchildaSquare`: 内接正四角形 (簡略)
- `MidchildaOctagram`: 八芒星 (本格)

**必要な解析**: AST + Symbol Resolution

**推奨条件**:
- 整然としたモジュール階層
- trait/impl のクリーンな分離
- 4分割または8分割の対称的な責務

**実装 Phase**: Phase 1 (ConcentricRings バリアント), Phase 2+ (他バリアント)

### 2.5 Alchemy式 (Five, Balanced)

**図像的特徴**: 五芒星 (正五芒星) + 中心の三角形 + 錬金術記号 (魂・肉体・精神を表す3記号)。

**構造**: 5頂点 + 中心三角形 (合計5+3点)。

**表現する関係性**: 等価交換の原則、双方向の契約、形式的に保証された変換。

**技術ドメイン**:
- Design by Contract (前後条件、不変条件)
- Refinement Types
- Hoare Logic
- Rust の所有権システム (静的契約)
- Haskell の型クラス制約
- TLA+ の形式仕様
- Property-based test

**必要な解析**: 型制約解析、前後条件抽出、契約注釈の収集

**推奨条件**:
- 関数前後条件のアサーション (`debug_assert!`, `pre`, `post`)
- Refinement type の使用
- `Result<T, E>` を中心とした変換チェーン
- TLA+ などの形式仕様と紐付くコード

**実装 Phase**: Phase 5

### 2.6 黒執事式 (Five, Divergent)

**図像的特徴**: 五芒星 (しばしば逆五芒星) + 外側に放射状のトゲ・槍。中心に円。

**構造**: 5頂点 + 外向き放射状の力場。

**表現する関係性**: 不可逆的な契約、永続的な縛り、片方向の取引。

**技術ドメイン**:
- Rust の Move semantics、Drop trait
- Affine type, Linear type
- 所有権の移譲 (ownership transfer)
- Async の cancellation token
- データベース transaction の commit (post no-rollback)
- ファイナライザ、リソース解放保証
- Append-only ログ、ブロックチェーン

**必要な解析**: 所有権解析、リソースフロー解析、destructor 追跡

**推奨条件**:
- `Drop` トレイトの実装が多い
- 関数が引数を move で受け取る (借用ではない)
- `std::mem::forget`, `ManuallyDrop` の使用
- transaction の `commit()` 呼び出し
- ファイル/ソケット/プロセスのライフサイクル管理

**実装 Phase**: Phase 5

### 2.7 RWBY式 (Six, Divergent fractal)

**図像的特徴**: 雪片状の6方向放射構造。各腕が同形 (フラクタル)。中央にヘキサゴン。各腕の先端に武器のような特徴的形状。

**構造**: 6点放射 + 自己相似的な内部構造。

**表現する関係性**: 並列処理、自己相似な再帰、Scatter-Gather パターン。

**技術ドメイン**:
- Rust の `rayon::par_iter`
- MapReduce
- CUDA の SIMD
- Spark の RDD
- Actor モデル (Erlang/Elixir)
- 関数型のコンビネータ (`map`, `fold`, `filter` の連鎖)
- 並列フューチャー (`tokio::join!`, `futures::join_all`)

**必要な解析**: 並列構造の検出、ループの並列化可能性

**推奨条件**:
- 並列イテレータの使用
- `join_all`, `try_join_all` の呼び出し
- 同一処理の複数並列実行
- データ並列性が明示されたコード

**実装 Phase**: Phase 5

### 2.8 Fate式 (Eight, Convergent + core)

**図像的特徴**: 八芒星 (正方形2つを45度ずらして重ねる) + 二重円 + 中央三点 (勾玉/巴/三位)。外周にルーン文字。

**構造**: 8頂点 + 中央三核。「外部から呼び出される構造」。

**表現する関係性**: 依存性注入 (DI)、ポート/アダプタ、召喚 (外部リソースの呼び出し)。

**技術ドメイン**:
- ヘキサゴナルアーキテクチャの拡張 (8 ports)
- Clean Architecture の外側 (8 adapters)
- マイクロカーネル + 周辺サービス
- OS の基本サブシステム (FS/Net/Mem/Proc/IPC/Device/Time/Input)
- NestJS, Spring の DI コンテナ
- マイクロサービスのコア + 周辺

**必要な解析**: トレイト境界解析、DI コンテナ設定の解析

**推奨条件**:
- 複数の trait による抽象 + 中央構造体への注入
- DI コンテナ (`@Injectable`, `@Provider`) の使用
- マイクロカーネル的な構造 (中央 + 多数のアダプタ)
- 8ポート程度の外部インターフェース

**実装 Phase**: Phase 5

### 2.9 夜天の書式 (Meta-recursive)

**図像的特徴**: 自己を内包する魔法陣。他の式の魔法陣を内部に含み、書き換えながら駆動する。固定の図像を持たず、動的に変化する。

**構造**: 自己参照ループ、メタレベルの構造。

**表現する関係性**: コードがデータであるシステム、メタプログラミング、自己書き換え。

**技術ドメイン**:
- マクロ展開 (Rust の `macro_rules!`, `proc_macro`)
- コード生成 (build.rs, codegen)
- リフレクション (Java の Reflection API)
- eval (Lisp 系)
- JIT コンパイル
- DSL 解釈器
- 自己ホスティングコンパイラ

**必要な解析**: マクロ展開トレース、コード生成トレース、リフレクション呼び出しの追跡

**推奨条件**:
- マクロ展開が多重に行われる
- build.rs でコード生成
- 動的な型情報取得
- 解釈器・コンパイラの実装

**実装 Phase**: Phase 6+

---

## 3. 自動推奨ロジックの実装

### 3.1 スコアリング方式

ツールは、AST 解析結果から各式への適合度をスコア化する。スコアが最も高い式を**推奨式**として提示する。

```rust
fn score_style(ir: &MysticalGraph) -> HashMap<RenderStyle, f64> {
    let mut scores = HashMap::new();
    
    // Belka スコアリング例
    let belka_signals = [
        ir.has_reducer_pattern(),
        ir.has_three_part_record(),
        ir.has_command_event_state(),
        ir.has_mvc_directory_structure(),
    ];
    scores.insert(RenderStyle::Belka, weighted_sum(&belka_signals));
    
    // 他の式についても同様
    scores
}
```

### 3.2 複数候補の提示

スコアが拮抗する場合は複数候補を提示する。例えば、トップ3を:

```
このコードに適合する描画様式:
1. ベルカ式 (スコア: 0.82) — Reducer パターンを検出
2. Alchemy式 (スコア: 0.71) — 前後条件アサーションを検出
3. ミッドチルダ式 (スコア: 0.45) — 構造化された階層

`--style belka` で描画、または `--style auto` で全候補を生成
```

### 3.3 ミスマッチの検出

ユーザーが明示的に指定した式と、自動推奨が大きく食い違う場合は警告を出す。

```
警告: 指定された Vesperia式は、このコードには適合しません。
このコードは Reducer パターン (ベルカ式) として描けます。
無理に Vesperia式で描画すると、図像が歪みます。
```

このミスマッチ警告は、**設計判断のフィードバック**として機能する。ユーザーが「Vesperia式で描こうとした」という意図と、「コードは Reducer パターンになっている」という現実のギャップが、設計の方向性を再考する機会を提供する。

---

## 4. RenderStyle の IR 統合

### 4.1 スキーマ

```rust
struct MysticalGraph {
    modules: Vec<Module>,
    cross_module_edges: Vec<Edge>,
    metadata: ProjectMetadata,
    inferred_style: Option<RenderStyle>,    // プロジェクト全体の主たる式
}

struct Module {
    id: ModuleId,
    name: String,
    sigils: Vec<Sigil>,
    edges: Vec<Edge>,
    inferred_style: Option<RenderStyle>,    // モジュールの主たる式
    style_scores: HashMap<RenderStyle, f64>, // 全式のスコア
}

struct Sigil {
    // 既存フィールド
    preferred_style: Option<RenderStyle>,   // この関数に最適な式
}

struct RenderStyle {
    order: SymmetryOrder,
    mode: ExpressionMode,
    variant: Option<StyleVariant>,
}
```

### 4.2 Phase ごとの埋まり方

**Phase 1**: 全フィールド未推定 (常に `None`)。レンダラはミッドチルダ式 (ConcentricRings) で固定描画。

**Phase 2**: `Sigil::preferred_style` を関数ごとに推定。ユーザーは `--style auto` で推定式を使うか、`--style midchilda` などで明示指定できる。

**Phase 3**: ベルカ式の描画を有効化 (データフロー解析の導入により)。

**Phase 4**: うみねこ式、Vesperia式、ミッドチルダ式他バリアントを有効化。

**Phase 5**: Alchemy式、黒執事式、RWBY式、Fate式を有効化。

**Phase 6**: 夜天の書式、立体ビュー、複数式の融合表示を有効化。

---

## 5. 未収集の組み合わせ

二軸分類モデルにおいて、(Order, Mode) の全ての組み合わせが既存ファンタジー作品の魔法陣として見つかっているわけではない。以下は現時点で未対応 (図像的参照が手元にない) の組み合わせ:

- (Two, *) — 2-fold は単純すぎて魔法陣として描かれにくいが、技術的には if/else や true/false の二項対立を表現できる
- (Three, Divergent) — 3点放出。トリプレットからの放散
- (Six, Convergent) — 6方向から中心への集約
- (Six, Orthogonal) — 6つの離散カテゴリ
- (Eight, Organic) — 8方位の有機的伝播
- (Higher, *) — 12-fold, 16-fold などの高次対称性

ツールは「未知の組み合わせを描けるアルゴリズム」として実装される。群論的に正しく描けば、対応する作品例がなくても新しい魔法陣が自動生成される。これは Phase 6 以降の自動生成エンジンの基礎となる。

---

## 6. 命名と中立訳語

配布時の現実的配慮として、各式には作品固有の名称と中立的訳語を併記する。

| 内部 enum 名 | 作品由来名 | 中立訳語 |
|---|---|---|
| Belka | ベルカ式 (リリカルなのは) | 古典三項式 |
| Umineko | うみねこ式 | 四象限分類式 |
| Vesperia | Vesperia式 (テイルズ) | 有機伝播式 |
| Midchilda | ミッドチルダ式 (リリカルなのは) | 近代直交式 |
| Alchemy | Alchemy式 (鋼の錬金術師) | 等価交換式 |
| Kuroshitsuji | 黒執事式 | 不可逆契約式 |
| RWBY | RWBY式 | 雪片並列式 |
| Fate | Fate式 | 召喚式 / 八方依存式 |
| Yagami | 夜天の書式 (リリカルなのは StrikerS) | 自己参照式 / メタ式 |

CLI では `--style classical-tripartite` のような中立訳語が標準で受け付けられ、`--style belka` のような作品由来名はエイリアスとして動作する。これは配布時の権利関係への配慮と、技術ドキュメントとしての中立性の両立である。

ただし、内部開発・カジュアルな議論では作品由来名のままで通すのが自然である。本ツールの命名は「真面目さと遊び心の両立」を貫く。

---

## 7. カタログの今後の拡張

新しい式の発見・追加は、以下のプロセスで行う:

1. 新しいファンタジー作品の魔法陣を観察
2. (Order, Mode) の組み合わせとして分類
3. 既存式との重複でないことを確認
4. 対応する技術ドメインを言語化
5. 推奨ヒューリスティクスを定義
6. 本カタログに追記

このカタログは生きているドキュメントであり、Mystical コミュニティ (将来的に存在するなら) で共同編集される想定。

---

## 付録: 参照すべき作品リスト (今後の収集対象)

未収集の図像で、収集価値が高いと思われる作品:

- ロード・オブ・ザ・リング (ガンダルフの魔法陣、One Ring の刻印)
- ハリー・ポッター (各種の防御円)
- ベルセルク (使徒召喚、烙印)
- 葬送のフリーレン (各種魔法陣)
- 魔法少女まどか☆マギカ (キュゥべえの結界)
- ソードアート・オンライン (System Call の魔法陣)
- ダンジョン飯 (シスル使用の魔法陣)
- メイドインアビス (祭祀場の図像)
- 七つの大罪 (各団員の紋章)
- BLEACH (鬼道、虚化、卍解の図像)

これらの収集により、未知の (Order, Mode) 組み合わせが埋まり、二軸分類が完成形に近づく。

---

## バージョン履歴

v0.1 — 初版。Phase 1 仕様書および Phase 2 以降ノートの補遺 A として、描画様式の体系を初出。9式 + 中立訳語対応表を含む。
