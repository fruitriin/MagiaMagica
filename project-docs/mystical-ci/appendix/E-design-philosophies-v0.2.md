# Mystical CI Appendix E: 設計哲学カタログ v0.2

> **本文書の位置づけ**
> 歴史的に重要な設計哲学・設計書を、本ツールの「式 (RenderStyle)」体系に翻訳して分類・分析する。
>
> **v0.2 での主な変更**: 
> - §1.3 として「設計書のメタ分類」を新規追加。設計書を**パターン集型 / 単一哲学型 / 戦略+戦術型 / 態度・原則型 / 規範書型**の5タイプに分類。これにより GoF などの「複数魔法陣の集合体」と POSD などの「単一哲学」の根本的な違いが明示される。
> - §2.5 (GoF) の記述を大幅強化。GoF を「カタログ式 (Catalog Style)」として位置づけ、新しい構造的概念を導入。
> - §3 の関係マップにメタ分類の軸を追加。

---

## 1. カタログの構造

### 1.1 各エントリの記述項目

各設計書について、以下の項目で分析する:

- 書誌情報、中核思想、対応する式、適用ドメイン、引き寄せられる優位性、トレードオフ、裁量 vs 規範の位置、ズームレベル、参照実装、本ツールでの活用

### 1.2 分類の二軸 (旧軸)

設計書を整理する2軸:

- **軸 1: 規範性 ↔ 裁量性**
- **軸 2: 粒度 (扱うズームレベル)**

### 1.3 ★v0.2 新規: 設計書のメタ分類 (構造軸)

これまでの軸に加えて、設計書の**構造的タイプ**という第3の軸を導入する。これは v0.2 で発見された極めて重要な分類軸である。

設計書は以下の5つの構造タイプに分類できる:

#### タイプ 1: パターン集型 (Pattern Catalog Type)

多数の独立した「解」のコレクション。各パターンは独立して選択・適用できる。書籍全体に貫かれる単一の哲学はなく、共通の語彙と分類軸のみが提供される。

**特徴**:
- 個々のパターンが独立した小さな魔法陣
- 「採用」は選択的 (全パターンを使う必要はない)
- 統一された巨視的構造を提案しない
- カタログから「必要なもの」を取り出す

**所属書籍**:
- GoF Design Patterns (1994) — 23 のパターン
- PoEAA (Fowler, 2002) — 51 のパターン
- Refactoring (Fowler, 1999) — 約 100 のリファクタリング技法
- Working Effectively with Legacy Code (Feathers, 2004) — 約 24 の Seam 技法

**対応する魔法陣構造**: **カタログ式 (Catalog Style)** — これは新たな構造的概念で、§1.4 で詳述する。

#### タイプ 2: 単一哲学型 (Single Philosophy Type)

一つの中核思想を徹底し、それを全体に貫く。書籍全体が一つの大きな主張の展開である。

**特徴**:
- 統一された巨視的構造を提案する
- 部分採用は不可能か、価値を大きく損なう
- 「採用」は包括的 (思想全体を受け入れる)
- 個別のパターンより、全体としての一貫性が重要

**所属書籍**:
- A Philosophy of Software Design (Ousterhout, 2018) — Deep Module の徹底
- Clean Architecture (Martin, 2017) — 同心円の規律
- Domain Modeling Made Functional (Wlaschin, 2018) — 型による不正状態の排除

**対応する魔法陣構造**: 単一の大きな式 (Profundus、Midchilda × Fate ハイブリッド、Alchemy × ベルカ ハイブリッドなど)

#### タイプ 3: 戦略+戦術型 (Strategy + Tactics Type)

巨視的なビジョンと具体的なパターンの両方を提供する。戦略レベルと戦術レベルが補完的に機能する。

**特徴**:
- 全体構造の指針 (戦略) と個別の解 (戦術) の二層構成
- 戦略だけでも、戦術だけでも価値が出る
- 両方を組み合わせて初めて完全になる
- 単一哲学型とパターン集型のハイブリッド

**所属書籍**:
- Domain-Driven Design (Evans, 2003) — Strategic Design + Tactical Patterns
- Building Microservices (Newman, 2015) — システム設計 + 個別パターン
- Continuous Delivery (Humble, Farley, 2010) — パイプライン全体 + 個別プラクティス

**対応する魔法陣構造**: 大きな式 (景観レベル) + 内部の複数の式 (モジュールレベル)

#### タイプ 4: 態度・原則型 (Attitude/Principle Type)

特定の技術的解や構造ではなく、開発者の態度・原則・心構えを提供する。実装の詳細より、判断の基準を与える。

**特徴**:
- 特定の式に縛られない、メタ的な指針
- 「採用」は文化的・態度的
- 機械的検証が困難
- 全体に通底する開発者の姿勢

**所属書籍**:
- The Pragmatic Programmer (Hunt & Thomas, 1999) — DRY, Orthogonality など
- The Mythical Man-Month (Brooks, 1975) — Conceptual Integrity など
- Programming Pearls (Bentley) — 思考法
- The Practice of Programming (Kernighan & Pike, 1999)

**対応する魔法陣構造**: 特定の式ではなく、すべての式に適用可能なメタ原則。本ツールの設計原則そのものとも重なる。

#### タイプ 5: 規範書型 (Manual Type)

包括的なベストプラクティス集。あらゆる側面を網羅的にカバーする百科事典的書籍。

**特徴**:
- 規範を全方位に提供
- 「採用」は全面的
- 厚い、重い (1000ページ超のことも)
- 教育的価値が高い

**所属書籍**:
- Code Complete (McConnell, 1993/2004) — 構築全般
- The Art of Computer Programming (Knuth) — アルゴリズム
- Effective Java (Bloch, 2001/2008/2018) — Java の規範
- Effective Modern C++ (Meyers, 2014)

**対応する魔法陣構造**: ミッドチルダ式 (近代直交式) を基本とした規範化された構造。各章が一つの小さな規範。

### 1.4 ★v0.2 新規: カタログ式 (Catalog Style) という新構造

パターン集型の設計書に対応する新しい魔法陣の構造を**カタログ式 (Catalog Style)** として定義する。これは Appendix A の他の式とは根本的に異なる構造を持つ。

**カタログ式の図像的特徴**:
- 多数の独立した小魔法陣が並ぶ
- 各小魔法陣は固有の式 (ベルカ、うみねこ、Vesperia など) を持つ
- 小魔法陣同士は**構造的には独立**
- 統一する大きな構造はない (これが他の式との決定的な違い)
- カタログ全体の図像は「魔導書のページ」のように、複数の異なる陣が並んだ書物

**カタログ式の構造的特性**:
```
SymmetryOrder::Meta  (Aperiodic と Meta の中間: 構造ではなく集合性)
ExpressionMode::CatalogCollection  (新規)
```

**カタログ式の数学的解釈**:
カタログ式は「圏 (Category)」というより「圏の集合 (Set of Categories)」、あるいは「圏の直和 (Coproduct of Categories)」に近い。各パターンは独立した圏として存在し、ユーザーが必要に応じて選び取る。

これに対し、単一哲学型の式は「圏」そのもの、戦略+戦術型は「圏の中の圏 (Internal Category)」、態度・原則型は「圏全体に対するメタ原理 (Functorial Property)」と整理できる。

**カタログ式の技術ドメイン**:
- GoF パターンの実装 (Adapter, Observer, Factory など個別に採用)
- PoEAA パターンの実装 (Repository, Unit of Work など個別に採用)
- リファクタリング技法の適用 (個別に適用)
- ライブラリの API カタログ (各 API が独立)

**カタログ式の自動推奨条件**:
- プロジェクト内に複数の異なる式のパターンが**意図的に**混在している
- パターン名が明示的にコメントや命名で示されている (`UserFactory`, `OrderObserver` など)
- ライブラリ的な性格を持つ (各機能が独立)

**カタログ式の独自の限界**:
カタログ式は構造的に一貫した全体図を持たないため、Mystical CI の「景観可視化」の対象としては難しい。各パターンを個別に描画することはできるが、それらを統合した一枚の図にはなりにくい。

これは「カタログ式は本来全体図を持たない」という性質を反映している。GoF を採用するということは、全体構造ではなく**個別の解の引き出し**を選ぶということ。

ただし、本ツールは Appendix B の景観論で論じた「異なる式のモジュールが結合点で繋がる」という構造を、カタログ式の中の各パターンに適用できる。GoF パターンを使ったコードでも、パターン同士の結合関係は分析可能。

### 1.5 メタ分類のまとめ

設計書のメタ分類を表で整理する:

| メタタイプ | 構造 | 採用の単位 | 例 |
|---|---|---|---|
| パターン集型 | カタログ式 | 個別 | GoF, PoEAA, Refactoring |
| 単一哲学型 | 単一の大きな式 | 全面 | POSD, Clean Architecture |
| 戦略+戦術型 | 景観 + 内部式 | 二層 | DDD, Microservices |
| 態度・原則型 | メタ原則 | 文化的 | Pragmatic Programmer, MMM |
| 規範書型 | 規範化された構造 | 全方位 | Code Complete |

これは「設計書をどう読むか」「どう採用するか」の指針にもなる。パターン集型は「カタログから選ぶ」、単一哲学型は「全面採用するか否か」、戦略+戦術型は「両層を意識する」、態度・原則型は「文化として浸透させる」、規範書型は「教科書として読む」、というふうに、タイプによって読み方・使い方が変わる。

---

## 2. 設計書ごとの分析

### 2.1 A Philosophy of Software Design (POSD)

**書誌情報**: John Ousterhout, 2018

**メタタイプ**: 単一哲学型

**中核思想**: Deep Module、複雑性の極小化、情報隠蔽、エラーの存在否定

**対応する式**: 深層式 (Profundus)

**裁量 vs 規範**: 強く裁量性寄り (90%)

**ズームレベル**: マイクロ〜ミドル

**本ツールでの活用**: Analyzer で深さスコア測定、Designer で Profundus 骨格を提供

(詳細は v0.1 と同様)

---

### 2.2 Clean Architecture

**書誌情報**: Robert C. Martin, 2017

**メタタイプ**: 単一哲学型

**中核思想**: 同心円の階層、Dependency Rule (内向きのみ)、ビジネスロジックの保護

**対応する式**: ミッドチルダ式 × Fate 式 ハイブリッド (Eight, Convergent + core)

**裁量 vs 規範**: 強く規範性寄り (90%)

**ズームレベル**: マクロ

**本ツールでの活用**: Analyzer で Dependency Rule 違反検出、Designer で骨格生成、Enforcer Mode との親和性高

(詳細は v0.1 と同様)

---

### 2.3 Domain-Driven Design (DDD)

**書誌情報**: Eric Evans, 2003

**メタタイプ**: 戦略+戦術型

**中核思想**: ドメインモデル中心、Ubiquitous Language、Bounded Context

**対応する式**: 
- 戦略的設計: 結合点と複数式の景観 (Appendix B)
- 戦術的設計: ベルカ式 + Alchemy 式 ハイブリッド

**裁量 vs 規範**: 中間 (50/50)

**ズームレベル**: ミドル〜マクロ

**本ツールでの活用**: 景観可視化との最高の親和性、Context Map の自動生成

(詳細は v0.1 と同様)

---

### 2.4 Domain Modeling Made Functional

**書誌情報**: Scott Wlaschin, 2018

**メタタイプ**: 単一哲学型 (DDD と FP の統合哲学)

**中核思想**: 型で不正状態を排除、F# で DDD を実装

**対応する式**: Alchemy 式 + ベルカ式 ハイブリッド

**裁量 vs 規範**: 規範寄り (70%) — 型システムが規範を強制

**ズームレベル**: マイクロ〜ミドル

(詳細は v0.1 と同様)

---

### 2.5 GoF Design Patterns ★v0.2 大幅強化

**書誌情報**: Gamma, Helm, Johnson, Vlissides (Gang of Four), 1994

**メタタイプ**: **パターン集型 (Pattern Catalog Type) ★v0.2 重要**

**中核思想**: 
オブジェクト指向設計で繰り返し現れる**23の独立した解**をカタログ化する。書籍全体に貫かれる単一の哲学はなく、Creational / Structural / Behavioral という分類軸と、共通の語彙 (Context, Problem, Solution, Consequences) のみが提供される。

**v0.2 での重要な発見**: 
GoF は他の主要設計書 (POSD, Clean Architecture, DDD) と根本的に異なる構造を持つ。POSD や Clean Architecture が**一つの大きな魔法陣**を提案するのに対し、**GoF は 23 個の独立した小魔法陣の集合体**である。

これは表面的な違いではなく、設計書としての存在論的な違いである。POSD を「採用する」とは哲学を受け入れることだが、GoF を「使う」とは個別パターンをカタログから取り出すことである。

**対応する魔法陣構造**: **カタログ式 (§1.4)**

各パターンに対応する個別の式:
- **Factory Method / Abstract Factory**: 召喚記号 (Fate 式の一部)
- **Builder**: ベルカ式 (Builder / Director / Product の三項)
- **Singleton**: Fate 式の中央核の縮退形
- **Prototype**: 夜天の書式の縮退形 (自己コピー)
- **Adapter**: 結合点 (Junction Glyph) の典型
- **Bridge**: 階層分離 + 結合点
- **Composite**: ミッドチルダ式 (階層構造)
- **Decorator**: Vesperia 式 (装飾の積層)
- **Facade**: Profundus 式 (狭い入口 + 広い内部)
- **Flyweight**: Profundus 式 + 共有プール
- **Proxy**: 結合点 (Junction Glyph) の特殊形
- **Chain of Responsibility**: 直列結合トポロジー
- **Command**: ベルカ式 (Command / Receiver / Invoker)
- **Interpreter**: 夜天の書式の典型 (DSL 解釈器)
- **Iterator**: RWBY 式の一腕 (要素を巡回)
- **Mediator**: Fate 式の中央核 (集約)
- **Memento**: 黒執事式 + 復元可能 (Undo)
- **Observer**: Vesperia 式 (横断的伝播)
- **State**: うみねこ式 (離散分類) + ベルカ式 (遷移)
- **Strategy**: 補助リング差し替え (ミッドチルダ式)
- **Template Method**: ミッドチルダ式 + フック点
- **Visitor**: 夜天の書式 (構造を辿る別レイヤーの処理)

つまり、GoF の 23 パターンは、Appendix A の式体系を**ほぼ網羅的にカバー**している。これは偶然ではなく、GoF が「OO 設計で繰り返し現れるパターン」を経験的に収集した結果、自然に**式の体系の現れ方をカタログ化**していたからである。

**カタログ式としての特性**:
- 23パターンそれぞれが独立した「小魔法陣」
- パターン同士は構造的に独立 (Adapter と Observer は別物)
- ただし、複数パターンの組み合わせ (例: MVC は Observer + Composite + Strategy) は頻繁
- 統一する大きな構造はない

**適用ドメイン**: 
オブジェクト指向プログラミング全般 (Java, C#, Python, C++, Smalltalk など)

**引き寄せられる優位性**:
- 既知の問題に対する標準解
- 23 個の独立した語彙 (チーム内コミュニケーション容易)
- 学習しやすい (個別パターンを順次学習可能)
- パターン名で意図が伝わる (`UserFactory`, `OrderObserver` のような命名習慣)

**トレードオフ**:
- 過剰適用 (パターン病) のリスク
- 関数型言語では多くが第一級言語要素として組み込まれており、明示的パターンは不要 (Strategy = 関数、Iterator = generator、Observer = signal/event)
- カタログから「適切に」選ぶ判断力が必要
- パターンランゲージとしては Alexander の系譜から離れている (Alexander は統一的なパターンの**言語**を目指したが、GoF は**カタログ**)

**裁量 vs 規範**: 
- パターンの選択は裁量 (どのパターンを使うか)
- 選択後の実装は規範 (パターンの構造に従う)
- 結果として中間 (50/50)

**ズームレベル**: マイクロ

**参照実装**: 
- Java の標準ライブラリ (`Iterator`, `Observer`, `Comparator`, ...)
- .NET Framework
- ほぼ全ての OO フレームワーク

**本ツールでの活用**:
- Analyzer: 23 パターンの自動検出 (既存ツールに似た機能)
- Designer: 「Observer パターンを使いたい」要求から、Vesperia 式の骨格を提供
- 23 パターンそれぞれが個別のレシピとなる
- ★v0.2: GoF 採用プロジェクトは「カタログ式」として景観可視化される。各パターンが小魔法陣として並ぶ、魔導書のようなビューが提供される。

**v0.2 における追加的観察**:
GoF を「カタログ式」と認識することで、以下が見える:

1. **GoF の限界**: 統一的なアーキテクチャを提案しない。GoF だけでシステム全体を設計しようとすると、各パターンの寄せ集めになり、景観としての一貫性を失う。これが「パターン病」と呼ばれる現象の正体。

2. **GoF と他の哲学の組み合わせ**: GoF は他の単一哲学型 (Clean Architecture, POSD) や戦略+戦術型 (DDD) と**自然に組み合わせ可能**。GoF は道具箱を提供し、他の哲学が構造を提供する。「Clean Architecture のフレーム上で、各層内部に GoF パターンを使う」が典型例。

3. **言語パラダイムによる変動**: GoF は OO 中心。関数型言語では多くのパターンが消える (Strategy → 高階関数、Iterator → モナド、Observer → FRP)。これは「パターンがカタログである」ことの裏返しで、言語が変わればカタログが書き換えられる。

4. **本ツールにおける表現**: カタログ式は単一の景観を持たないため、本ツールの可視化は「魔導書のページのような表示」になる。複数の独立した小魔法陣が並ぶ、図書館的なビュー。これは Appendix B の景観論と異なる、新しい表現様式の必要性を示唆する。

---

### 2.6 Patterns of Enterprise Application Architecture (PoEAA)

**書誌情報**: Martin Fowler, 2002

**メタタイプ**: **パターン集型** (GoF と同じカテゴリ)

エンタープライズアプリケーション特化のパターンカタログ。51 のパターン。GoF と同じくカタログ式に対応する。

**対応する式**: カタログ式 + 各パターンが固有の式
- Layered Architecture: ミッドチルダ式
- Domain Model: ベルカ + Alchemy
- Active Record: Profundus 式
- Data Mapper: 結合点
- Unit of Work: 黒執事式
- Lazy Load: Profundus 式
- Identity Map: Fate 式の中央核
- Service Layer: ベルカ式 + 結合点

**裁量 vs 規範**: 規範寄り (65%)

**ズームレベル**: ミドル〜マクロ

(詳細は v0.1 と同様)

---

### 2.7 Refactoring

**書誌情報**: Martin Fowler, 1999 (第2版 2018)

**メタタイプ**: **パターン集型** (リファクタリング技法のカタログ)

約 100 のリファクタリング技法のカタログ。各技法は独立して適用可能。

**対応する式**: 
- カタログ式 + 各技法が**式変換 (Style Refactoring)** の例
- スメルは「式と現状のミスマッチ」として理解できる

**本ツールでの活用**: 
- 本ツールの**式変換機能**の理論的基礎
- Fowler のスメル一覧は、本ツールの自動検出ルールの参考
- リファクタリングカタログ全体がレシピ集の基礎

(詳細は v0.1 と同様)

---

### 2.8 The Pragmatic Programmer

**書誌情報**: Andy Hunt, Dave Thomas, 1999 (20周年記念版 2019)

**メタタイプ**: 態度・原則型

特定の設計哲学ではなく、実用的な開発者の態度・習慣・原則を体系化。

**対応する式**: 特定の式に縛られない、メタ的な実践

**裁量 vs 規範**: 強く裁量寄り (80%)

**本ツールでの活用**: ツールの設計哲学の参考。「Orthogonality」は本ツールの式の独立性の根拠。

(詳細は v0.1 と同様)

---

### 2.9 Code Complete

**書誌情報**: Steve McConnell, 1993 (第2版 2004)

**メタタイプ**: 規範書型

ソフトウェア構築の包括的なベストプラクティス集。

**対応する式**: ミッドチルダ式 + 規範重視

**裁量 vs 規範**: 強く規範寄り (75%)

(詳細は v0.1 と同様)

---

### 2.10 Working Effectively with Legacy Code

**書誌情報**: Michael Feathers, 2004

**メタタイプ**: **パターン集型** (Seam の技法カタログ)

レガシーコードに対する Seam (継ぎ目) を見つけて挿入する技法のカタログ。

**対応する式**: 
- カタログ式 + 各 Seam が結合点 (Junction Glyph) の典型
- 改善過程 = 式変換の段階的適用

(詳細は v0.1 と同様)

---

### 2.11 Functional Programming in Scala / Haskell

**書誌情報**: 各種 (Chiusano & Bjarnason 2014, Lipovača 2011 など)

**メタタイプ**: 単一哲学型 (関数型プログラミングの哲学)

**対応する式**: 
- 基本: ベルカ式 + Alchemy 式 ハイブリッド
- Monad 連鎖: 直列結合パイプライン
- Type Class: Profundus 式
- 関数合成: RWBY 式

**裁量 vs 規範**: 規範寄り (70%)

(詳細は v0.1 と同様)

---

### 2.12 Building Microservices

**書誌情報**: Sam Newman, 2015 (第2版 2021)

**メタタイプ**: 戦略+戦術型

マイクロサービス特化の戦略と戦術。

**対応する式**: 
- Fate 式が複数のサービスとして散在
- 網状結合トポロジー
- 各サービス内部は別の式
- 結合点が API Gateway, Service Mesh

(詳細は v0.1 と同様)

---

### 2.13 The Mythical Man-Month

**書誌情報**: Fred Brooks, 1975

**メタタイプ**: 態度・原則型

ソフトウェア開発の本質的困難を論じた古典。

**対応する式**: 特定の式というより、メタ的な指針

**本ツールでの活用**: 景観スコアの理論的根拠、Conway's Law を視覚化する装置として本ツールが機能。

(詳細は v0.1 と同様)

---

## 3. 書籍間の関係マップ (v0.2 拡張)

### 3.1 メタ分類による分類 ★v0.2 新規

```
パターン集型 (Catalog Style)
├── GoF Design Patterns
├── PoEAA
├── Refactoring
└── Working Effectively with Legacy Code

単一哲学型 (Single Philosophy)
├── POSD
├── Clean Architecture
├── Domain Modeling Made Functional
└── Functional Programming 各書

戦略+戦術型 (Strategy + Tactics)
├── DDD
├── Building Microservices
└── Continuous Delivery (将来追加予定)

態度・原則型 (Attitude / Principles)
├── The Pragmatic Programmer
├── The Mythical Man-Month
└── Programming Pearls (将来追加予定)

規範書型 (Manual)
├── Code Complete
├── Effective Java
└── The Art of Computer Programming
```

### 3.2 補完関係

```
POSD (単一哲学・マイクロ・裁量) ←→ Clean Architecture (単一哲学・マクロ・規範)
   個別モジュールの深さ                システム全体の構造

DDD (戦略+戦術・ミドル・中間) ←→ Functional DDD (単一哲学・マイクロ・規範)
   戦略的設計                          戦術的設計の型による表現

GoF (パターン集・マイクロ・中間) ←→ PoEAA (パターン集・ミドル・規範)
   汎用パターン                        業務システム特化パターン

Refactoring (パターン集・マイクロ・裁量) ←→ Working with Legacy Code (パターン集・ミドル・中間)
   技法のカタログ                      適用の戦略
```

### 3.3 構造的補完 ★v0.2 新規

メタタイプは**互いに補完的**であることが多い:

```
単一哲学型 (Clean Architecture)
    + パターン集型 (GoF)
    = 全体構造は Clean Architecture、内部実装は GoF パターン
    
戦略+戦術型 (DDD)
    + 単一哲学型 (Functional DDD)
    = DDD の戦略 + 関数型の戦術
    
規範書型 (Code Complete)
    + 態度・原則型 (Pragmatic Programmer)
    = 規範を守りつつ、態度として柔軟性も持つ
```

メタタイプの混合は、現実の優れたプロジェクトの典型的な構造。「単一哲学型だけ」「パターン集型だけ」では限界があり、メタタイプを組み合わせることで強い設計が生まれる。

### 3.4 対立関係

```
パターン集型 (GoF)         単一哲学型 (POSD)
   ↓                            ↓
   個別の解の集合              統一的な哲学
   選択的採用                  包括的採用
   構造の不在                  構造の徹底

態度・原則型 (Pragmatic)    規範書型 (Code Complete)
   ↓                            ↓
   原則と態度                  詳細なベストプラクティス
   裁量                        規範
```

### 3.5 三軸マトリクス (v0.2 拡張)

```
                マイクロ          ミドル          マクロ
              ┌────────────────┬────────────────┬────────────────┐
   裁量寄り   │ POSD           │ Pragmatic      │ Mythical M-M   │
              │ Pragmatic      │                │                │
   (態度+哲学)│ Refactoring    │                │                │
              ├────────────────┼────────────────┼────────────────┤
   中間       │ GoF (★カタログ)│ DDD (★戦略+戦術)│ Microservices  │
              │ Legacy Code    │ Refactoring    │ (★戦略+戦術)   │
   (パ集+戦戦)│ (★パターン集)  │                │                │
              ├────────────────┼────────────────┼────────────────┤
   規範寄り   │ Code Complete  │ PoEAA          │ Clean Arch     │
              │ Functional DDD │ (★パターン集)  │ (★単一哲学)    │
   (規範書+哲)│ Effective Java │ Functional DDD │                │
              └────────────────┴────────────────┴────────────────┘
```

各セルにメタタイプ ★ も表示することで、設計書を**裁量 × 粒度 × メタタイプ**の三軸で位置づけられる。

---

## 4. 推奨される読書順序

### 4.1 新人開発者向け

1. Code Complete (規範書型 → 基礎の規範)
2. Pragmatic Programmer (態度・原則型 → 開発者の態度)
3. Refactoring (パターン集型 → 改善の語彙)
4. GoF Design Patterns (パターン集型 → パターンの語彙)

メタタイプ的に: 規範書型 → 態度型 → パターン集型 の順。これは「基礎を守る」「態度を学ぶ」「道具を増やす」というステップ。

### 4.2 中級開発者向け

1. Refactoring (再読)
2. POSD (単一哲学型 → Deep Module)
3. Functional Programming 系 (単一哲学型 → 型と合成)
4. Working with Legacy Code (パターン集型 → 既存対処)

メタタイプ的に: パターン集型から単一哲学型へ。「カタログから選ぶ」段階から「哲学を選ぶ」段階へ。

### 4.3 シニア開発者向け (アーキテクト志望)

1. Clean Architecture (単一哲学型 → 規律)
2. DDD (戦略+戦術型 → ドメイン)
3. Domain Modeling Made Functional (単一哲学型 → DDD + FP)
4. PoEAA (パターン集型 → 業務実践)
5. Building Microservices (戦略+戦術型 → 分散)
6. Mythical Man-Month (態度型 → 本質的困難)

メタタイプ的に: すべてのタイプを経験する。アーキテクトは複数のメタタイプを使い分ける必要がある。

### 4.4 ★v0.2 新規: メタタイプ別の読書順

特定のメタタイプを深掘りしたい場合:

**パターン集型コース**:
GoF → PoEAA → Refactoring → Working with Legacy Code

**単一哲学型コース**:
POSD → Clean Architecture → Functional DDD → Functional Programming

**戦略+戦術型コース**:
DDD → Building Microservices → Continuous Delivery → Team Topologies

**態度・原則型コース**:
Pragmatic Programmer → Mythical Man-Month → Programming Pearls

---

## 5. ツールへの統合 (v0.2 拡張)

### 5.1 メタタイプ別の Designer Mode 振る舞い ★v0.2 新規

ユーザーが書籍を指定するとき、メタタイプによって Designer Mode の振る舞いが変わる:

**パターン集型を指定** (`--book "GoF" --pattern observer`):
- 個別パターンの骨格を提供
- 統一的な構造は強制しない
- 必要に応じて複数パターンを組み合わせる

**単一哲学型を指定** (`--book "POSD"`):
- 哲学全体の骨格を提供
- 統一的な構造を強制
- スタンスは哲学に従う (POSD なら Advisor)

**戦略+戦術型を指定** (`--book "DDD"`):
- 戦略レベル (Context Map) と戦術レベル (Aggregate, Entity, VO) の両方を提供
- ズームレベルごとに異なる骨格

**態度・原則型を指定** (`--book "Pragmatic Programmer"`):
- 特定の構造は提供しない
- 開発習慣のチェックリストを提供 (CI ルールへの変換)

**規範書型を指定** (`--book "Code Complete"`):
- 全方位の規範を CI ルールとして提供
- 命名規約、フォーマット規約など

### 5.2 メタタイプとスタンスの組み合わせ

| メタタイプ | 推奨スタンス | 理由 |
|---|---|---|
| パターン集型 | Advisor | カタログから選択するため裁量重視 |
| 単一哲学型 | 哲学次第 | POSD なら Advisor、Clean Arch なら Enforcer |
| 戦略+戦術型 | Hybrid | 戦略は規範、戦術は裁量 |
| 態度・原則型 | Advisor | 文化的浸透が中心 |
| 規範書型 | Enforcer | 規範の徹底が目的 |

### 5.3 比較モード ★v0.2 拡張

```bash
$ mystical compare --books "POSD,GoF" --domain "library design"

POSD のアプローチ (単一哲学型):
  - 深層式 (Profundus) で各機能を deep module 化
  - 統一的な「狭い API + 深い実装」の哲学を適用
  - スタンス: Advisor
  
GoF のアプローチ (パターン集型):
  - 必要に応じて Facade, Adapter, Strategy などを採用
  - 統一的な哲学はなく、個別パターンを組み合わせる
  - スタンス: Advisor
  - 注意: パターン病に陥らないよう、本当に必要なパターンのみを使う
  
両者の組み合わせ (推奨):
  - 全体は POSD の哲学 (Deep Module)
  - 内部実装で必要な箇所のみ GoF パターン (Facade で deep module の入口を構成、Strategy で内部の柔軟性、など)
  - これにより、哲学の徹底とパターンの実用性が両立
```

メタタイプを考慮した比較により、「哲学 vs カタログ」「規範 vs 裁量」が立体的に見える。

---

## 6. 設計哲学の時代的変遷

(v0.1 と同様、ただしメタタイプの観点を追加)

```
1970s: Brooks (態度・原則型) - 本質的困難の認識
       Parnas - Information Hiding (単一哲学型の萌芽)
       
1980s: OOP の台頭、Smalltalk
       Design Patterns の萌芽

1990s: GoF (パターン集型) - OO パターンの体系化
       Refactoring (パターン集型) - 改善の体系化
       Pragmatic Programmer (態度・原則型) - 実用主義
       Code Complete (規範書型) - 規範の総合
       
2000s: DDD (戦略+戦術型) - ドメイン中心
       PoEAA (パターン集型) - エンタープライズ
       Working with Legacy Code (パターン集型)
       Agile Manifesto (態度・原則型) - プロセス革命
       
2010s: Microservices (戦略+戦術型) - 分散システム
       Clean Architecture (単一哲学型) - 規律の再強調
       POSD (単一哲学型) - 裁量の再評価
       Domain Modeling Made Functional (単一哲学型)
       
2020s: ★ Mystical CI (記号体系) - 視覚的設計言語の提案
       ★ AI-augmented design - メタタイプの境界が曖昧化?
```

**v0.2 の観察**: 1990年代はパターン集型が支配的だったが、2010年代以降は単一哲学型 (Clean Architecture, POSD) が増えている。これは「カタログでは飽き足らず、統一的な哲学を求める」時代の流れと読める。

ただし、現実の優れたプロジェクトは**複数のメタタイプを組み合わせる**ため、特定のメタタイプの「勝利」とは見ない方が正確。

---

## 7. 残された設計哲学

(v0.1 と同様)

未収集の設計書: TDD (Beck), Smalltalk Best Practice Patterns (Beck), Continuous Delivery, SRE, Accelerate, Team Topologies, SICP, Knuth など。

Phase 2 以降に Appendix E v0.3 で追加される候補。

---

## 8. 本カタログの活用方針

### 8.1 静的なカタログとしての価値

各設計書を読まなくても、その思想と本ツールの式体系における位置を把握できる。**v0.2 ではメタタイプも明示される**ため、設計書の選び方が立体的に分かる。

### 8.2 動的なレシピソースとしての価値

各書籍はレシピとして本ツールに統合される。メタタイプによって Designer Mode の振る舞いが変わるため、「カタログから選ぶ」体験と「哲学を採用する」体験が区別される。

### 8.3 教育的価値

新人開発者は本カタログを通じて、設計書の存在、全体像、そして**メタタイプの違い**を把握できる。「全部読む必要はない、メタタイプを意識して選べばよい」という方針が立つ。

### 8.4 議論の出発点としての価値

チーム内で「うちのプロジェクトはどのメタタイプを採用しているか」「他のメタタイプを試す価値はあるか」という議論を、本カタログが媒介する。これは Conway's Law の作用を**意識的に活用する**議論である。

---

## バージョン履歴

v0.1 — 初版。13の主要設計書を分析し、本ツールの式体系における位置づけを行った。書籍間の補完・対立関係をマップ化。読書順序とツール統合方針を提示。

v0.2 — **メタ分類軸の新規導入**。設計書を「パターン集型 / 単一哲学型 / 戦略+戦術型 / 態度・原則型 / 規範書型」の5タイプに分類。GoF を「カタログ式 (Catalog Style)」として位置づけ、§1.4 で新しい構造的概念を導入。各書籍の記述にメタタイプを追加。関係マップを三軸 (裁量 × 粒度 × メタタイプ) に拡張。Designer Mode の振る舞いをメタタイプ別に整理。
