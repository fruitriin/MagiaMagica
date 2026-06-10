# Mystical CI Appendix E: 設計哲学カタログ v0.1

> **本文書の位置づけ**
> 本文書は『Mystical CI 仕様書 v0.1』および各 Appendix に続く補遺 E である。
>
> ソフトウェア工学において歴史的に重要な設計哲学・設計書を、本ツールの「式 (RenderStyle)」体系に翻訳して分類・分析する。Christopher Alexander の建築パターン以来の系譜にある設計書群を、視覚的記号として位置づけることを目的とする。
>
> Appendix A が「式」の体系を提供し、Appendix D が「レシピ」の枠組みを提供するのに対し、本文書は「歴史的に蓄積されたレシピ集」を体系的に整理する。本ツールが既存の設計知識資産と接続する架け橋として機能する。

---

## 1. カタログの構造

### 1.1 各エントリの記述項目

各設計書について、以下の項目で分析する:

- **書誌情報**: 著者、出版年、版
- **中核思想**: 書籍が主張する核心
- **対応する式**: Appendix A の式体系における位置づけ
- **適用ドメイン**: その哲学が最も効果を発揮する領域
- **引き寄せられる優位性**: 採用することで得られる性質
- **トレードオフ**: 失われるもの、コスト
- **裁量 vs 規範の位置**: スペクトラム上の位置
- **ズームレベル**: どの粒度を扱うか (Appendix B のズーム階層)
- **参照実装**: 具体的なプロジェクト例
- **本ツールでの活用**: Analyzer / Designer どちらで使うか

### 1.2 分類の二軸

設計書を整理する2軸を導入する:

**軸 1: 規範性 ↔ 裁量性**
- 規範性: 構造を強制し、判断を廃する側
- 裁量性: 構造を提案し、判断を尊重する側

**軸 2: 粒度 (扱うズームレベル)**
- マイクロ: 関数・モジュール内部
- ミドル: モジュール間・パッケージ
- マクロ: アーキテクチャ全体・組織

この二軸のマトリクス上に各設計書を配置すると、書籍同士の関係性が浮かび上がる。

---

## 2. 設計書ごとの分析

### 2.1 A Philosophy of Software Design (POSD)

**書誌情報**: John Ousterhout, 2018 (第2版 2021)

**中核思想**: 
複雑性こそがソフトウェア開発の根本問題である。Deep Module (狭いインターフェース、深い実装) こそが良いモジュールであり、複雑性は下層に押し込み (Pull complexity downward)、エラーは存在しないものとして定義し (Define errors out of existence)、汎用化と情報隠蔽を徹底する。

**対応する式**: **深層式 (Profundus, Aperiodic)**

**適用ドメイン**: 
- 個別モジュール設計
- ライブラリ設計
- API/SDK 設計
- 関数・クラス・パッケージレベルの設計判断

**引き寄せられる優位性**: 
- インターフェースの単純さによる誤用の少なさ
- 認知負荷の低減
- 高い汎用性
- 長期的な保守性

**トレードオフ**: 
- 実装の複雑性が集中する
- 上級者向け (達人技を要する)
- 初期実装コストが高い
- レビュー時の認知負荷 (内部が深いため)

**裁量 vs 規範**: **強く裁量性寄り** (90% 裁量)
- 「Tactical Programming vs Strategic Programming」の対比で、Ousterhout は Strategic を推奨するが、それは個人の判断を継続的に求めるアプローチ
- ガードレールではなく、自問する文化を要求する

**ズームレベル**: マイクロ〜ミドル

**参照実装**: 
- Unix システムコール (5要素程度のシンプル API + 巨大な kernel)
- Lisp の defun
- Brian Kernighan の C 標準ライブラリ
- 良くできた SDK 全般

**本ツールでの活用**: 
- Analyzer: 各関数・モジュールの DepthScore を測定
- Designer: 「Deep Module を作りたい」要求から、Profundus 式の骨格を提供

---

### 2.2 Clean Architecture

**書誌情報**: Robert C. Martin (Uncle Bob), 2017

**中核思想**: 
ソフトウェアは同心円の階層構造で組織化されるべきである。外側から内側へ Frameworks & Drivers → Interface Adapters → Use Cases → Entities の順に並び、依存は常に内向きのみ。フレームワークから独立し、テスタビリティを保ち、ビジネスルールを保護する。

**対応する式**: **ミッドチルダ式 × Fate式 のハイブリッド (Eight, Convergent + core)**

**適用ドメイン**:
- エンタープライズアプリケーション
- 長期保守が必要なシステム
- 多人数チームでの開発
- 業務システム

**引き寄せられる優位性**:
- テスタビリティの極大化
- フレームワークへの独立性
- ビジネスロジックの保護
- 規律ある一貫性
- 新人参画の容易性 (構造が明確)

**トレードオフ**:
- 初期実装の overhead
- スモールスタートには過剰
- ドメイン変化への硬直性 (一度決めた構造の変更が高コスト)
- 単純な CRUD アプリには重い

**裁量 vs 規範**: **強く規範性寄り** (90% 規範)
- 構造を一度決めれば、後の判断は最小化される
- 「Dependency Rule」のような単純なルールで違反を機械的に検出可能
- 個人の判断より構造への忠誠

**ズームレベル**: マクロ

**参照実装**:
- 各種 Clean Architecture テンプレート
- Onion Architecture (Jeffrey Palermo)
- Hexagonal Architecture (Alistair Cockburn)
- Domain-Driven Design 寄りのエンタープライズ Java

**本ツールでの活用**:
- Analyzer: 同心円構造の検出、Dependency Rule 違反の警告
- Designer: 新規プロジェクトの骨格生成 (Fate × ミッドチルダ ハイブリッド)
- Enforcer Mode との親和性が高い

---

### 2.3 Domain-Driven Design (DDD)

**書誌情報**: Eric Evans, 2003

**中核思想**:
ソフトウェアの中核はドメインモデルである。Ubiquitous Language (ユビキタス言語) を共有し、Bounded Context (境界づけられたコンテキスト) で意味の境界を区切り、Context Map で複数の境界を結ぶ。Aggregate, Entity, Value Object, Repository, Domain Service, Domain Event などの戦術的パターンを用いる。

**対応する式**: 
- 戦略的設計 (Strategic Design): **結合点 (Junction Glyph) と複数式の景観**
  - 各 Bounded Context が独自の式を持つモジュール
  - Context Map = 複数モジュールの景観 (Appendix B の景観論と直結)
- 戦術的設計 (Tactical Design): **ベルカ式 + Alchemy式 のハイブリッド**
  - Entity/Value Object の三項関係 (ID/属性/振る舞い) はベルカ式
  - 不変条件と契約は Alchemy式
- Ubiquitous Language: **外周のルーン文字に対応** (各魔法陣の外周にドメイン語彙のラベル)

**適用ドメイン**:
- 複雑なビジネスドメイン
- 長期的に育てるプロダクト
- ドメインエキスパートとの協業が必要なシステム
- 大規模システムの境界設計

**引き寄せられる優位性**:
- ドメインモデルとコードの一致
- ビジネスロジックの明確化
- チーム間のコミュニケーション容易化 (Ubiquitous Language)
- 境界が明確な大規模システム

**トレードオフ**:
- 学習コストが高い
- 単純な CRUD には過剰
- ドメインエキスパートの巻き込みが必要
- 全パターンを適用すると複雑度が増す

**裁量 vs 規範**: **中間** (50% 裁量 / 50% 規範)
- Strategic Design は裁量を要求 (境界をどこに引くか)
- Tactical Patterns は規範を提供 (Entity/VO の使い方)

**ズームレベル**: ミドル〜マクロ

**参照実装**:
- "Implementing Domain-Driven Design" (Vaughn Vernon)
- Axon Framework
- Java + Spring の DDD 実装
- F# + Domain Modeling Made Functional

**本ツールでの活用**:
- Analyzer: Bounded Context の境界検出、Context Map の自動生成
- Designer: 「ドメイン駆動の設計を始める」要求から、Aggregate, Entity, VO の骨格を提供
- 景観可視化と最も親和性が高い

---

### 2.4 Domain Modeling Made Functional

**書誌情報**: Scott Wlaschin, 2018

**中核思想**:
ドメインモデルを型 (Type) で表現する。「不正な状態を表現不能にする (Make illegal states unrepresentable)」を目指し、F# のような関数型言語の代数的データ型を活用する。DDD と関数型プログラミングの統合。

**対応する式**: **Alchemy式 + ベルカ式のハイブリッド (型による契約 + Reducer パターン)**

**適用ドメイン**:
- 関数型言語によるビジネスシステム
- 型安全性を最大化したいドメインロジック
- 不変なデータモデル中心のシステム

**引き寄せられる優位性**:
- 型レベルで不正な状態を排除
- 副作用の隔離
- リファクタリングの安全性
- 仕様 = 型 = ドキュメント

**トレードオフ**:
- 関数型言語の習熟が必要
- パフォーマンスチューニングの難しさ
- 命令型に慣れたチームでの学習コスト

**裁量 vs 規範**: **規範寄り** (70% 規範)
- 「不正な状態を表現不能に」というルールが強い規範として作用
- 型システムが規範を機械的に強制する

**ズームレベル**: マイクロ〜ミドル

**参照実装**:
- F# によるドメインモデリング
- Haskell の Refinement Types
- Rust の `Result<T, E>` + Newtype Pattern
- TypeScript の Discriminated Union を活用したドメインモデル

**本ツールでの活用**:
- Analyzer: 型による契約の検出、不正状態の可能性の警告
- Designer: 「型安全なドメインモデルを設計する」要求から、Alchemy 式の骨格を提供

---

### 2.5 GoF Design Patterns

**書誌情報**: Gamma, Helm, Johnson, Vlissides (Gang of Four), 1994

**中核思想**:
オブジェクト指向設計で繰り返し現れる23の設計パターンをカタログ化する。Creational, Structural, Behavioral の3カテゴリに分類。各パターンは特定の問題に対する再利用可能な解決策。

**対応する式**: **個々のパターンが固有の式に対応**

代表的な対応:
- **Factory Method / Abstract Factory**: 召喚記号としての Fate 式の要素
- **Builder**: ベルカ式 (Builder/Director/Product の三項)
- **Singleton**: 中央核としての Fate 式の縮退形
- **Adapter**: 結合点 (Junction Glyph) の典型
- **Facade**: Profundus 式 (狭い入口 + 広い内部)
- **Observer**: Vesperia 式 (横断的伝播)
- **Strategy**: 補助リング差し替え (ミッドチルダ式)
- **Command**: ベルカ式 (Command/Receiver/Invoker)
- **Iterator**: RWBY 式の腕としての並列イテレーション
- **State**: うみねこ式 (状態の離散分類) + ベルカ式 (遷移)
- **Visitor**: 夜天の書式 (構造を辿る別レイヤーの処理)

**適用ドメイン**:
- オブジェクト指向プログラミング全般
- 特に Java, C#, Python のエンタープライズ系
- フレームワーク設計

**引き寄せられる優位性**:
- 既知の問題に対する標準解
- チーム内のコミュニケーション (パターン名で意図が伝わる)
- 拡張性
- 教育的価値

**トレードオフ**:
- 過剰適用 (パターン病) のリスク
- 言語パラダイムによっては不要 (関数型では多くが第一級言語要素)
- 学習が記憶ベース (23 パターンの暗記)
- パターンランゲージとしてはやや弱い (Alexander の系譜から離れている)

**裁量 vs 規範**: **中間** (50% / 50%)
- パターンの選択は裁量
- 選択後の実装は規範

**ズームレベル**: マイクロ

**参照実装**:
- ほぼ全ての OO 言語の標準ライブラリ
- Spring Framework
- .NET Framework

**本ツールでの活用**:
- Analyzer: GoF パターンの自動検出 (既存ツールに似た機能)
- Designer: 「Observer パターンを使いたい」要求から、Vesperia 式の骨格を提供
- 23パターンそれぞれがレシピとなる

---

### 2.6 Patterns of Enterprise Application Architecture (PoEAA)

**書誌情報**: Martin Fowler, 2002

**中核思想**:
エンタープライズアプリケーション (業務システム) に固有の設計パターンを体系化。レイヤー化、ドメインロジックパターン、データソースパターン、O/R マッピング、Web プレゼンテーション、分散、並行性、セッション状態、ベースパターン。

**対応する式**: **Fate 式 (8-fold + core)** を基本に、各パターンが特定の式に対応

代表的な対応:
- **Layered Architecture**: ミッドチルダ式 (整然とした階層)
- **Domain Model**: ベルカ式 + Alchemy式 (DDD と同様)
- **Transaction Script**: 直列結合のパイプライン
- **Active Record**: Profundus 式 (簡潔なインターフェース + 内部の魔術)
- **Data Mapper**: 結合点 (Junction Glyph)
- **Unit of Work**: 黒執事式 (不可逆的なコミット)
- **Lazy Load**: Profundus 式 (表面は静、内部で動)
- **Identity Map**: Fate 式の中央核
- **Service Layer**: ベルカ式 + 結合点
- **MVC, MVP, Model-View-Presenter**: ベルカ式の各種バリエーション

**適用ドメイン**:
- 業務システム
- データベース駆動のアプリケーション
- Web アプリケーション

**引き寄せられる優位性**:
- 既存の解決策の活用
- エンタープライズ特有の問題への対処
- フレームワーク設計の指針

**トレードオフ**:
- 2002 年の Java 中心の世界観 (一部は古い)
- マイクロサービス時代には更新が必要
- 重厚な実装になりがち

**裁量 vs 規範**: **規範寄り** (65% 規範)

**ズームレベル**: ミドル〜マクロ

**参照実装**:
- Java EE / Spring Framework
- .NET Framework
- Ruby on Rails (Active Record)
- Django (簡素化されたパターン)

**本ツールでの活用**:
- Analyzer: PoEAA パターンの検出
- Designer: 業務システム特化のレシピ提供
- 各パターンが個別のレシピになる

---

### 2.7 Refactoring

**書誌情報**: Martin Fowler, 1999 (第2版 2018, 例が Java から JavaScript に)

**中核思想**:
動作を変えずに内部構造を改善する技法のカタログ。「コードのスメル」と、それに対応するリファクタリング手法の対応表。Extract Method, Inline Method, Move Function, Extract Class などの低レベル変換から、Replace Conditional with Polymorphism のような高レベル変換まで。

**対応する式**: 
- 個々のリファクタリングは式変換 (Style Refactoring) の例
- スメルは「式と現状のミスマッチ」として理解できる

代表的な対応:
- **Long Method**: メインリングが大きすぎる → Extract Method で補助リングに分解
- **Large Class**: 神オブジェクト → Profundus 式の悪い適用
- **Long Parameter List**: 狭いインターフェースに反する → Profundus 式の理想から離脱
- **Divergent Change**: モジュールが複数の理由で変更される → 式の混在
- **Shotgun Surgery**: 散弾銃手術 → 結合点が散乱
- **Feature Envy**: 自モジュール内より他モジュールへの呼び出しが多い → 重心オフセット
- **Data Clumps**: 同じパラメータパターンの繰り返し → 型抽出の機会
- **Primitive Obsession**: プリミティブ型の濫用 → Alchemy 式から離脱

**適用ドメイン**: 
- すべてのコードベース (refactoring は普遍的に有効)

**引き寄せられる優位性**:
- 段階的な改善
- 動作の保証 (テストとセットで運用)
- スメルの言語化と検出

**トレードオフ**:
- テスト整備が前提
- 大規模リファクタリングは時間を要する
- ツール支援なしには労力が大きい

**裁量 vs 規範**: **裁量寄り** (60% 裁量)

**ズームレベル**: マイクロ〜ミドル

**参照実装**: 
- IDE のリファクタリング機能 (IntelliJ IDEA, VS Code 等)
- 各種のリファクタリングツール

**本ツールでの活用**:
- **本ツールの中核機能の一つ**: 式変換 (Style Refactoring) は Refactoring の発展形
- Analyzer: スメル検出 (式とコードのミスマッチ)
- Designer: リファクタリング後の目標式を提案
- Refactoring のカタログ全体が本ツールのレシピ集の基礎となる

---

### 2.8 The Pragmatic Programmer

**書誌情報**: Andy Hunt, Dave Thomas, 1999 (20周年記念版 2019)

**中核思想**:
特定の設計哲学ではなく、実用的な開発者の態度・習慣・原則を体系化。DRY (Don't Repeat Yourself), Orthogonality, Tracer Bullets, Prototypes, Plain Text, Source Code Control, など。

**対応する式**: **特定の式に縛られない、メタ的な実践**

- DRY → 式の合成や継承による重複排除
- Orthogonality → 各式の独立性の確保
- Tracer Bullets → 早期に End-to-End な細い実装 (景観を先に作る)
- Plain Text → 視覚言語より柔軟性を重視

**適用ドメイン**: すべて

**引き寄せられる優位性**:
- バランスの取れた開発文化
- 特定パラダイムへの偏向なし
- 実用的な習慣の獲得

**トレードオフ**:
- 体系性に欠ける (チェックリスト的)
- 各原則の深い掘り下げは別書籍に譲る

**裁量 vs 規範**: **強く裁量寄り** (80% 裁量)

**ズームレベル**: 全レベル

**参照実装**: なし (態度の問題)

**本ツールでの活用**:
- 直接的なレシピ化は難しい
- ツールの設計哲学の参考として位置づける
- 「Orthogonality」は本ツールの式の独立性の根拠

---

### 2.9 Code Complete

**書誌情報**: Steve McConnell, 1993 (第2版 2004)

**中核思想**:
ソフトウェア構築 (construction) の包括的なベストプラクティス集。コードの書き方、命名、コメント、フロー制御、エラー処理、テスト、品質向上、レイアウト、心理的考察まで。

**対応する式**: **ミッドチルダ式 (近代直交式) + 規範重視**

**適用ドメイン**: 
- 全ての構築フェーズ
- 新人教育
- コードレビュー基準

**引き寄せられる優位性**:
- 包括的なベストプラクティス
- 規範化された品質
- 教科書的な明快さ

**トレードオフ**:
- 1990年代の世界観が残る
- 関数型・並行性などへの言及が薄い
- 1000ページ超の重厚さ

**裁量 vs 規範**: **強く規範寄り** (75% 規範)

**ズームレベル**: マイクロ〜ミドル

**参照実装**: Microsoft の内部開発ガイドライン

**本ツールでの活用**:
- 命名規約、レイアウトなどはツールの低レベル設定に反映
- 直接的な式マッピングは難しい

---

### 2.10 Working Effectively with Legacy Code

**書誌情報**: Michael Feathers, 2004

**中核思想**:
テストがないコード (= レガシーコード) に対し、テストを追加しながら段階的に改善する技法。Seam (継ぎ目) を見つけて挿入し、テスト可能な単位に分解していく。

**対応する式**: 
- レガシーコード = 式の混乱、結合点の不在
- 改善過程 = 式変換 (Style Refactoring) の段階的適用

**適用ドメイン**:
- レガシーコードベース
- テストがないシステム
- 大規模リファクタリングの対象

**引き寄せられる優位性**:
- レガシーコードからの脱出経路
- リスクを抑えた段階的改善
- テスト先行の習慣化

**トレードオフ**:
- 短期的には進捗が見えにくい
- 高い忍耐力が必要

**裁量 vs 規範**: **中間** (50% / 50%)

**ズームレベル**: ミクロ〜ミドル

**参照実装**: レガシーコード改善の各種事例

**本ツールでの活用**:
- 「式の混乱した状態」の検出と、それを整理する段階的なリファクタリング提案
- 「Seam」は本ツールの結合点 (Junction Glyph) の概念と直結

---

### 2.11 Functional Programming in Scala / Haskell

**書誌情報**: 
- Functional Programming in Scala (Paul Chiusano, Rúnar Bjarnason, 2014)
- Haskell Programming from First Principles (Allen, Moronuki, 2016)
- Learn You a Haskell (Miran Lipovača, 2011)

**中核思想**:
関数型プログラミングの原則 (純粋性、不変性、第一級関数、代数的データ型、Monad など) を徹底的に学ぶ。副作用の隔離、合成可能性、参照透過性。

**対応する式**:
- 基本: **ベルカ式 + Alchemy式 のハイブリッド**
- Monad の連鎖: **直列結合のパイプライン**
- Type Class: **Profundus 式** (狭い trait インターフェース + 深い実装)
- 関数合成: **RWBY 式** (並列適用の合成)

**適用ドメイン**:
- 関数型言語によるシステム
- 不変なドメインモデル
- 並行性が重要なシステム
- 数学的厳密性が必要な領域

**引き寄せられる優位性**:
- 合成可能性
- 副作用の隔離
- 並行処理の安全性
- 数学的に証明可能なコード

**トレードオフ**:
- 学習曲線が急峻
- 命令型に慣れたチームの抵抗
- パフォーマンスチューニングの複雑さ
- 人材確保の困難

**裁量 vs 規範**: **規範寄り** (70% 規範) — 型システムが規範を強制する

**ズームレベル**: マイクロ〜ミドル

**参照実装**:
- Haskell の標準ライブラリ
- Scala の cats / scalaz
- F# のドメインモデリング
- PureScript

**本ツールでの活用**:
- Analyzer: 関数型コードの特徴的な式の検出
- Designer: 関数型スタイルのレシピ提供
- Type Class や Monad の可視化

---

### 2.12 Building Microservices

**書誌情報**: Sam Newman, 2015 (第2版 2021)

**中核思想**:
マイクロサービスアーキテクチャの設計原則と実践。サービスの境界、通信、データ管理、デプロイ、テスト、監視、セキュリティ。

**対応する式**:
- **Fate 式 (8-fold + core)** が複数のサービスとして散在
- **網状結合トポロジー** (Mesh) としてのサービス間関係
- 各サービス内部は別の式 (POSD 的な Deep Module も可)
- **結合点 (Junction Glyph)** が API Gateway, Service Mesh として実装される

**適用ドメイン**:
- 大規模分散システム
- 独立したチームが並行開発する組織
- スケーラビリティ重視のシステム

**引き寄せられる優位性**:
- 独立したデプロイ
- 障害の局所化
- チームの自律性
- 技術スタックの多様性

**トレードオフ**:
- 分散システムの複雑性
- ネットワーク依存
- 監視・トレーシングのコスト
- データ整合性の難しさ
- 小規模システムには過剰

**裁量 vs 規範**: **中間** (50% / 50%)

**ズームレベル**: マクロ (組織レベル)

**参照実装**: Netflix, Amazon, Spotify のマイクロサービス事例

**本ツールでの活用**:
- 景観可視化の主たる対象
- サービス間の結合点を Junction Glyph として可視化
- 異なるサービスが異なる式を持つことを許容

---

### 2.13 The Mythical Man-Month

**書誌情報**: Fred Brooks, 1975 (記念版 1995)

**中核思想**:
ソフトウェア開発の本質的困難 (Essential Complexity) を論じた古典。「人月の神話」「銀の弾丸はない」「Conceptual Integrity」。後の Conway's Law にも影響。

**対応する式**: **特定の式というより、メタ的な指針**

- Conceptual Integrity → 景観の式の一貫性 (Appendix B の品質指標)
- Conway's Law → 組織構造と式の対応
- Second System Effect → 過剰設計の警告 (式の過剰適用)

**適用ドメイン**: 
- プロジェクト管理
- 組織設計
- 大規模開発の理解

**引き寄せられる優位性**:
- 長期的な視点
- 神話 (誤解) の解消
- 本質的困難への謙虚さ

**トレードオフ**:
- 具体的な技法は限定的
- 現代の文脈との差異 (アジャイル以前)

**裁量 vs 規範**: **裁量寄り** (60% 裁量)

**ズームレベル**: マクロ (組織)

**本ツールでの活用**:
- 景観スコアの理論的根拠
- Conway's Law を視覚化する装置として本ツールが機能

---

## 3. 書籍間の関係マップ

### 3.1 補完関係 (互いを補強する)

```
POSD (マイクロ・裁量) ←→ Clean Architecture (マクロ・規範)
   個別モジュールの深さ        システム全体の構造
   両者は補完的、組み合わせ可能

DDD (ミドル・中間) ←→ Functional DDD (マイクロ・規範)
   戦略的設計                  戦術的設計の型による表現

Refactoring (マイクロ・裁量) ←→ Working with Legacy Code (ミドル・中間)
   技法のカタログ                適用の戦略

GoF (マイクロ・中間) ←→ PoEAA (ミドル・規範)
   汎用パターン                  業務システム特化パターン
```

### 3.2 対立関係 (互いに異なる原則)

```
POSD                     Clean Architecture
   狭いインターフェース        広いインターフェース許容
   階層を作らない              階層を強制
   裁量                        規範

Pragmatic Programmer       Code Complete
   原則と態度                  詳細なベストプラクティス
   裁量                        規範

Functional Programming     OO Patterns (GoF)
   不変・純粋                  可変状態と振る舞い
   合成                        継承
```

### 3.3 補完軸のマトリクス

```
                マイクロ          ミドル          マクロ
              ┌────────────────┬────────────────┬────────────────┐
   裁量寄り   │ POSD          │ Pragmatic      │ Mythical M-M   │
              │ Pragmatic     │ Programmer     │ Brooks         │
              │ Refactoring   │                │                │
              ├────────────────┼────────────────┼────────────────┤
   中間       │ GoF           │ DDD            │ Microservices  │
              │ Legacy Code   │ Refactoring    │ Microservices  │
              ├────────────────┼────────────────┼────────────────┤
   規範寄り   │ Code Complete │ PoEAA          │ Clean Arch     │
              │ Functional DDD│ Functional DDD │ Clean Arch     │
              └────────────────┴────────────────┴────────────────┘
```

各セルに対応する設計書を配置することで、読者は自分の状況 (ズームレベル × 文化スタンス) に応じた書籍を選択できる。

---

## 4. 推奨される読書順序

### 4.1 新人開発者向け

1. **Code Complete** (or 言語固有の入門書) - コードの基礎
2. **Pragmatic Programmer** - 開発者の態度
3. **Refactoring** - 改善の語彙
4. **GoF Design Patterns** - パターンの語彙

### 4.2 中級開発者向け (個別モジュール設計の達人を目指す)

1. **Refactoring** (再読)
2. **POSD** - Deep Module の哲学
3. **Functional Programming** 系 - 型と合成の力
4. **Working with Legacy Code** - 既存コードへの対処

### 4.3 シニア開発者向け (アーキテクト志望)

1. **Clean Architecture** - 同心円の規律
2. **DDD** - ドメインの言語化
3. **Domain Modeling Made Functional** - DDD + FP
4. **PoEAA** - 業務システムの実践知
5. **Building Microservices** - 分散システム
6. **Mythical Man-Month** - 本質的困難への謙虚さ

### 4.4 異質式適用に挑む開発者向け

1. **POSD** と **Clean Architecture** の対比を読む
2. **Functional Programming** で関数型の発想を取り込む
3. **DDD** で境界の発想を取り込む
4. その上で、本ツールの Designer Mode を活用して**異質式適用**を試す

---

## 5. ツールへの統合

### 5.1 レシピとしての書籍

各設計書は本ツールの**レシピ集** (Appendix D §4) に登録される。

例:
```yaml
recipe_id: clean-architecture-fullstack
based_on: "Clean Architecture (Robert C. Martin, 2017)"
desired_property:
  - テスタビリティ最大化
  - フレームワーク独立性
  - 長期保守性
recommended_style: Fate × Midchilda (Eight, Convergent + core)
stance: regular  # 規範寄り
applicable_domains:
  - エンタープライズ業務システム
  - 多人数チームの長期プロジェクト
expected_benefits:
  - 新人参画の容易性
  - ビジネスロジックの保護
  - テストの書きやすさ
trade_offs:
  - 初期コスト
  - スモールスタートには重い
reference_implementations:
  - "Clean Architecture template (各種言語)"
  - "Onion Architecture (Palermo)"
code_skeleton: "src/skeletons/clean-arch/{lang}.template"
related_books:
  - "Patterns of Enterprise Application Architecture (Fowler)"
  - "Domain-Driven Design (Evans)"
```

### 5.2 検索インターフェース

ユーザーは設計書名で検索できる:

```bash
$ mystical design --book "POSD"
$ mystical design --book "Clean Architecture" --language rust
$ mystical design --recipe deep-module
```

### 5.3 比較モード

複数の設計書のアプローチを比較できる:

```bash
$ mystical compare --books "POSD,Clean Architecture" --domain "state management"

POSD のアプローチ:
  - 深層式 (Profundus) で状態管理を deep module 化
  - 公開 API は get/set/subscribe の3つだけ
  - 内部実装は自由 (Reactive、Immutable、Mutable 何でも可)
  
Clean Architecture のアプローチ:
  - Use Case 層で状態管理ロジックを定義
  - Repository 経由でデータ層と分離
  - Entity が中央の核
  
両者の補完案:
  - 全体構造は Clean Architecture
  - Use Case 層内の各 Use Case は Deep Module (POSD)
```

これは Phase 4+ の野心的な機能だが、本ツールの設計支援としての真価を示す。

---

## 6. 設計哲学の時代的変遷

設計書の出版年から、設計哲学の時代的変遷が読み取れる:

```
1970s: Brooks (Mythical Man-Month) - 本質的困難の認識
       Parnas - Information Hiding の提唱
       
1980s: OOP の台頭、Smalltalk
       Design Patterns の萌芽

1990s: GoF (1994) - OO パターンの体系化
       Refactoring (1999) - 改善の体系化
       Pragmatic Programmer (1999) - 実用主義
       Code Complete (1993) - 規範の総合
       
2000s: DDD (Evans, 2003) - ドメイン中心
       PoEAA (Fowler, 2002) - エンタープライズ
       Working with Legacy Code (Feathers, 2004)
       Agile Manifesto (2001) - プロセス革命
       
2010s: Microservices (Newman, 2015) - 分散システム
       Clean Architecture (Martin, 2017) - 規律の再強調
       POSD (Ousterhout, 2018) - 裁量の再評価
       Domain Modeling Made Functional (Wlaschin, 2018)
       
2020s: ?
```

2020年代の代表的な設計哲学はまだ確立していないが、以下の方向性が見える:

- **AI-Augmented Programming**: AI が設計判断に参加する世界
- **Visual Programming**: コード可視化の本格的活用 (本ツール含む)
- **Substrate-Agnostic Design**: WebAssembly, Edge Computing, Browser, Server, Mobile を横断する設計
- **Sustainability**: 長期保守を超えて、エネルギー効率も含む設計
- **Distributed Cognition**: チームの認知を分散させる設計 (組織と AI の協業)

本ツール (Mystical CI) はこれらの 2020年代の流れの中に位置づけられる試みである。

---

## 7. 残された設計哲学

本カタログでカバーされていない、しかし価値のある設計書:

- **Smalltalk Best Practice Patterns** (Beck) - OO の語彙の根源
- **Test-Driven Development** (Beck) - TDD の体系化
- **Continuous Delivery** (Humble, Farley) - リリースの自動化
- **Site Reliability Engineering** (Google) - 運用と設計の融合
- **Accelerate** (Forsgren et al.) - 開発組織のメトリクス
- **Team Topologies** (Skelton, Pais) - チーム構造とコンウェイの法則
- **Programming Pearls** (Bentley) - 古典的なアルゴリズム的設計
- **Structure and Interpretation of Computer Programs (SICP)** - 計算プロセスの本質
- **The Art of Computer Programming** (Knuth) - アルゴリズムの百科事典

これらは Phase 2 以降に Appendix E v0.2 で追加される候補となる。

---

## 8. 本カタログの活用方針

### 8.1 静的なカタログとしての価値

各設計書を読まなくても、その思想と本ツールの式体系における位置を把握できる。これは技術選定における**羅針盤**として機能する。

### 8.2 動的なレシピソースとしての価値

各書籍はレシピとして本ツールに統合され、Designer Mode で活用される。「Clean Architecture スタイルで新規プロジェクトを始める」が `mystical design --book "Clean Architecture"` の一行で開始できる。

### 8.3 教育的価値

新人開発者は、本カタログを通じて設計書の存在と全体像を把握できる。読むべき書籍の優先順位を、自分の経験レベルと目的に応じて判断できる。

### 8.4 議論の出発点としての価値

チーム内で「うちのプロジェクトはどの設計哲学を採用しているか」「他の哲学を試す価値はあるか」という議論を、本カタログが媒介する。

---

## バージョン履歴

v0.1 — 初版。13の主要設計書を分析し、本ツールの式体系における位置づけを行った。書籍間の補完・対立関係をマップ化。読書順序とツール統合方針を提示。
