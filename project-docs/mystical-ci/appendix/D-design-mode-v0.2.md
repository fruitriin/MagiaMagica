# Mystical CI Appendix D: 設計支援モードと式駆動設計 v0.2

> **本文書の位置づけ**
> 本ツールの双方向性 (Analyzer / Designer の両モード) を確立する文書。
>
> **v0.2 での主な変更**: 
> - §4 (式のレシピ集) に Clean Architecture, POSD, Clean-POSD Fusion の3レシピを正式追加 (Appendix E と連携)
> - 新セクション §11 (ツールのスタンス選択) を追加し、Advisor/Enforcer/Hybrid モードを正式仕様化
> - 既存の §11 (歴史的位置づけ) と §10 (最終的な位置づけ) を §12, §13 にスライド

---

## 1. ツールの双方向性

### 1.1 二つのモード

- **Analyzer Mode**: Code → Analysis → Magic Circle (現状診断)
- **Designer Mode**: Desired Property → Magic Circle → Code Skeleton (設計支援)

両モードは同じ視覚言語 (式の体系) を共有し、双方向に行き来できる。Analyzer で診断した結果から、Designer で改善案を構築する、というワークフローが自然に成立する。

### 1.2 モードの切替

```bash
# Analyzer Mode
$ mystical render src/state/reducer.rs::update

# Designer Mode
$ mystical design --property fault-tolerance --domain api-handler
$ mystical design --style alchemy --output-skeleton src/skeleton/
$ mystical design --recipe "state machine with refinement types"
$ mystical design --book "Clean Architecture" --language rust  # ★v0.2: Appendix E 連携
```

### 1.3 価値の乗算

双方向ツールになることで、ツールの価値は乗数的に増える。Analyzer の出力と Designer の入力を比較できることで、本ツールは「**式の変換器 (Style Transformer)**」としても機能できる。

---

## 2. 異質式適用 (Cross-Paradigm Application)

### 2.1 概念

各ドメインには「当たり前」とされる式が存在する。**異質式適用**とは、この当たり前を意図的に破り、別の式を当てることで新しいパラダイムと優位性を引き寄せる設計手法である。

技術史上、多くのブレイクスルーは「当たり前を疑った異質適用」から生まれている。

### 2.2 具体例

(v0.1 と同様、6つの具体例を保持)

- 状態管理に Alchemy 式: Refinement Types ベース、Elm のような型安全な状態管理
- API ハンドラに RWBY 式: Actor モデル、Erlang OTP 的な fault-tolerance
- リソース管理に Vesperia 式: GC + Weak refs、Reactive な解放
- DI コンテナにベルカ式: 最小限の3層 DI
- 並列処理に Alchemy 式: STM、合成可能な並列性
- ビジネスロジックに夜天の書式: ルールエンジン、業務 DSL

### 2.3 異質式適用の方法論

7ステップで進める: 当たり前の式の特定 → 限界の言語化 → 求める優位性の言語化 → 候補式の選定 → パイロット実装 → 評価 → 採用判断。

---

## 3. 式駆動設計 (Style-Driven Design)

### 3.1 概念

設計を式の選択から開始する手法。求める性質に対応する式を最初に選び、骨格を描いてからコードを実装する。

### 3.2 SDD の利点

設計意図の視覚的明示、パラダイムの一貫性、異質適用の制度化、設計議論の構造化。

### 3.3 SDD の限界

式が固定される弊害、創造性の限定、コミュニケーションコスト。これらは「ツールは式を強制しない、提案する」という設計原則で緩和される。

---

## 4. 式のレシピ集

### 4.1 レシピの形式

```yaml
recipe_id: <unique-id>
based_on: "<book or paper reference>"  # ★v0.2: Appendix E との連携
desired_property: [...]
recommended_style: <style or composition>
stance: <advisor|enforcer|hybrid>  # ★v0.2: スタンス選択
applicable_domains: [...]
expected_benefits: [...]
trade_offs: [...]
reference_implementations: [...]
code_skeleton: <path-to-template>
related_books: [...]
```

### 4.2 既存の主要レシピ

(v0.1 で列挙したもの)

- 型安全性最大化レシピ → Alchemy式 (Elm, Haskell)
- fault-tolerance レシピ → RWBY式 (Erlang OTP)
- 横断的関心事分離レシピ → Vesperia式 (Spring AOP)
- 契約による設計レシピ → Alchemy式 (Eiffel, F*)
- マイクロカーネルレシピ → Fate式 (NestJS, Spring)
- リアクティブストリームレシピ → Vesperia式 + RWBY式 ハイブリッド (RxJS)
- メタプログラミング DSL レシピ → 夜天の書式 (Racket, Rebol)
- Functional Core レシピ → ベルカ式 + Alchemy式 (Elm, Bernhardt スタイル)
- Event Sourcing レシピ → ベルカ式
- Saga パターンレシピ → ベルカ式 + 黒執事式 ハイブリッド
- Pipes and Filters レシピ → 直列結合トポロジー
- Pub/Sub レシピ → Vesperia式 + ハブ&スポーク

### 4.3 ★v0.2 新規: Clean Architecture レシピ

```yaml
recipe_id: clean-architecture-fullstack
based_on: "Clean Architecture (Robert C. Martin, 2017)"
desired_property:
  - テスタビリティ最大化
  - フレームワーク独立性
  - 長期保守性
  - 規律ある構造
  - 新人参画の容易性
recommended_style:
  composition: hybrid
  outer: Midchilda  # 八芒星バリアント
  inner: Fate       # 中央核
  symmetry: (Eight, Convergent + core)
  dependency_rule: inward-only
stance: enforcer  # 規範重視
applicable_domains:
  - エンタープライズアプリケーション
  - 業務システム
  - 多人数チームでの長期プロジェクト
  - ドメインが比較的安定したシステム
expected_benefits:
  - 同心円層による依存制約 (Dependency Rule)
  - 内側ほど安定的・抽象的、外側ほど不安定・具象的
  - フレームワーク変更の影響範囲を外周に限定
  - ビジネスロジック (Entities) の保護
  - 各層が独立にテスト可能
  - 構造に従えば誰でも一定品質
trade_offs:
  - 初期実装の overhead
  - スモールスタートには過剰
  - ドメイン変化への硬直性 (一度決めた構造の変更が高コスト)
  - 簡単な CRUD アプリには重い
  - 「Second System Effect」を引き起こしやすい
reference_implementations:
  - "Clean Architecture template (各種言語)"
  - "Onion Architecture (Jeffrey Palermo)"
  - "Hexagonal Architecture (Alistair Cockburn)"
  - "DDD + Clean Architecture (Java/Spring)"
code_skeleton: "src/skeletons/clean-arch/{language}.template"
related_books:
  - "Patterns of Enterprise Application Architecture (Fowler)"
  - "Domain-Driven Design (Evans)"
  - "Code Complete (McConnell)"

violations_to_warn:
  - dependency-direction-outward      # 内→外への依存禁止
  - layer-skip                         # 層を飛び越える依存
  - business-logic-in-adapters         # ビジネスロジックがアダプタ層に漏れる
  - framework-coupling-in-core         # コア層がフレームワークに依存

automatic_checks:
  - dependency_rule: required (CI で違反は fail)
  - layer_isolation: required
  - core_purity: warning (Phase 5+ で AI 検証)
```

このレシピは Enforcer Mode との親和性が高く、CI で違反を自動検出してビルドを fail させる運用が想定される。

### 4.4 ★v0.2 新規: POSD レシピ (Deep Module)

```yaml
recipe_id: posd-deep-module
based_on: "A Philosophy of Software Design (Ousterhout, 2018)"
desired_property:
  - 認知負荷の低減
  - インターフェースの単純さ
  - 誤用の少なさ
  - 個別最適化
  - ドメイン追従性
recommended_style:
  primary: Profundus  # 深層式 (Aperiodic, Profundus)
  composition_with: any  # 他のすべての式と合成可能
stance: advisor  # 裁量重視
applicable_domains:
  - 個別モジュール設計
  - ライブラリ設計
  - API/SDK 設計
  - 少数精鋭チームでの開発
  - ドメインが流動的なシステム
expected_benefits:
  - 公開インターフェースが極小 (誤用が起きにくい)
  - 内部実装の自由 (各モジュールが最適形を選択)
  - 認知負荷の局所化
  - 高い汎用性
  - 長期保守容易 (隠蔽の徹底)
trade_offs:
  - 実装の複雑性が内部に集中
  - 上級者向け (達人技を要する)
  - レビュー時の認知負荷 (内部が深い)
  - 「自問する文化」が必要
  - 一貫した品質の維持が個人に依存
reference_implementations:
  - "Unix システムコール"
  - "Lisp の defun と関数群"
  - "Brian Kernighan の C 標準ライブラリ"
  - "良くできた SDK 全般"
  - "React の useState (極小 API + 巨大な実装)"
code_skeleton: "src/skeletons/posd-deep-module/{language}.template"
related_books:
  - "The Pragmatic Programmer (Hunt & Thomas)"
  - "Refactoring (Fowler)"
  - "Functional Programming 系の書籍"

depth_metrics:
  ideal_depth_ratio: > 5.0  # 実装/インターフェース比
  warning_depth_ratio: < 2.0  # 浅すぎる
  interface_complexity_max: 5  # 公開シンボル数の上限 (目安)

automatic_checks:
  - depth_score: warning (自動測定、警告のみ)
  - interface_size: warning (公開 API が肥大化したら警告)
  - information_hiding: warning (private 詳細が漏れたら警告)
```

POSD レシピは Advisor Mode との親和性が高く、深さスコアを警告として表示するが、強制はしない。「自問する文化」を支援するツールとして機能する。

### 4.5 ★v0.2 新規: Clean-POSD Fusion レシピ

```yaml
recipe_id: clean-posd-fusion
based_on: 
  - "Clean Architecture (Martin, 2017)"
  - "A Philosophy of Software Design (Ousterhout, 2018)"
desired_property:
  - システム全体の規律 (Clean Architecture)
  - 個別モジュールの最適化 (POSD)
  - スケーラブルな品質
  - 長期的な進化への対応
recommended_style:
  composition: zoom-level-multi-style
  zoom_level_1: Midchilda × Fate  # プロジェクト全体景観
  zoom_level_2: Midchilda × Fate  # モジュール間構造
  zoom_level_3: Profundus         # 個別モジュール内部
stance: hybrid  # 構造は規範、内部実装は裁量
applicable_domains:
  - エンタープライズアプリケーション
  - 長期保守が必要な大規模システム
  - 多人数 × 達人混在のチーム
  - 品質と柔軟性を両立したいプロジェクト
expected_benefits:
  - 外側から見ると Clean Architecture の整然とした同心円
  - ズームインすると各モジュールが POSD の Deep Module
  - 規律と裁量の両立
  - 新人は構造に従い、達人は内部で力を発揮
  - フラクタル景観 (Appendix B §5) の理想形
trade_offs:
  - 両方の哲学への理解が必要
  - チームメンバーのスキル分布が広い必要がある
  - 一貫性の管理が複雑
  - レビュー時に両方の観点を持つ必要
reference_implementations:
  - "良くできた OSS の大規模プロジェクト (例: PostgreSQL, Redis)"
  - "成熟したフレームワーク (例: Linux Kernel)"
  - "Apple, Google の内部設計指針 (公開部分)"
code_skeleton: "src/skeletons/clean-posd-fusion/{language}.template"
related_books:
  - すべての Appendix E §2 の書籍が部分的に関連

enforcement_strategy:
  outer_structure: enforced  # Clean Architecture 部分は強制
  module_internals: advised  # POSD 部分は推奨
  
zoom_level_specific_checks:
  level_1: clean_architecture_rules  # システム全体
  level_2: clean_architecture_rules  # モジュール間
  level_3: posd_depth_metrics        # モジュール内部
```

これは「現代ソフトウェアの理想形」と呼びうる組み合わせで、Hybrid Mode の典型例。Phase 4+ で本格的に対応する。

### 4.6 コミュニティによる拡張

レシピ集はコミュニティで継続的に育てる:

- `mystical-recipe-microservices`
- `mystical-recipe-game-engine`
- `mystical-recipe-ml-pipeline`
- `mystical-recipe-embedded`
- `mystical-recipe-ddd-tactical` ★v0.2 追加候補
- `mystical-recipe-event-sourcing` ★v0.2 追加候補
- `mystical-recipe-functional-core` ★v0.2 追加候補

クレートとして配布。

---

## 5. 式の合成 (Style Composition)

### 5.1 概念

複数の式が組み合わさってシステムを構成する**式の合成**には、以下のパターンがある:

- **並列合成**: 異なるモジュールが異なる式 (Appendix B 既出)
- **入れ子合成**: 大きな式の中に別の式が部分的に埋め込まれる
- **重ね合わせ合成**: 同じモジュールが複数の式の性質を同時に持つ
- **変換合成**: 時間の経過とともに式が別の式へ変化
- **★v0.2 追加: ズームレベル合成**: ズームレベルに応じて式が変わる (Clean-POSD Fusion が典型)

### 5.2 ハイブリッド式の例

- ベルカ + Alchemy = 契約付き Reducer (Elm)
- Vesperia + RWBY = リアクティブ並列ストリーム (RxJava)
- Fate + Alchemy = 契約による依存性注入 (ZIO Layer)
- ベルカ + 黒執事 = Saga パターン
- ミッドチルダ + 夜天 = プラグイン可能アーキテクチャ
- **★v0.2 追加: Fate + Profundus = 各層が deep module の Clean Architecture**
- **★v0.2 追加: ベルカ + Profundus = Reducer が deep module (Elm Update 関数)**
- **★v0.2 追加: Vesperia + Profundus = ミドルウェアそれぞれが deep module**
- **★v0.2 追加: RWBY + Profundus = 並列ワーカーそれぞれが deep module (Erlang プロセス)**

### 5.3 未発見の合成領域

まだ広く実装されていない興味深い組み合わせ:

- うみねこ式 (離散分類) + RWBY式 (フラクタル並列)
- 黒執事式 (不可逆契約) + 夜天の書式 (メタ) — リソース管理のメタプログラミング
- Vesperia式 + Alchemy式 — 横断的契約
- ★v0.2 追加: Profundus + 夜天の書式 — 内部にメタ機能を持つ deep module

本ツールが新しいアーキテクチャパターンの発見器として機能する可能性を示唆する。

### 5.4 合成の図像表現

- 並列合成: 隣接配置 + 結合点
- 入れ子合成: 内部に別の式が描き込まれる
- 重ね合わせ合成: 透明度を持つ複数の式の重ね描き
- 変換合成: アニメーションまたは時系列スライダー
- **★v0.2 追加: ズームレベル合成**: ズームに応じて自動的に式が切り替わる

特に重ね合わせ合成とズームレベル合成は、立体ビュー (Phase 6) との親和性が高い。

---

## 6. Designer Mode のインターフェース設計

### 6.1 対話的なコマンド

```bash
$ mystical design --interactive

? どんな性質を求めますか? (複数選択可)
  ❯ ◉ fault-tolerance
    ◯ 型安全性
    ◯ 横断的関心事の分離
    ◯ 認知負荷の低減 (★v0.2 追加: POSD的観点)
    ◯ 規律ある構造 (★v0.2 追加: Clean Architecture的観点)
    
? 適用ドメインは?
  ❯ API ハンドラ / ワーカープロセス / 状態管理 / ドメインロジック
  
? スタンスは?
  ❯ Advisor (推奨のみ、強制しない)
    Enforcer (構造を強制、CI で違反検出)
    Hybrid (一部強制、一部推奨)

推奨される式とレシピ:
  1. RWBY式 (fault-tolerance-via-fractal) — スコア 0.87
  2. Clean Architecture (clean-architecture-fullstack) — スコア 0.65
  3. POSD (posd-deep-module) — スコア 0.52

? どれを採用しますか?
```

### 6.2 非対話的なコマンド

```bash
# レシピを直接指定
$ mystical design --recipe fault-tolerance-via-fractal --lang rust

# 書籍を直接指定 (★v0.2: Appendix E 連携)
$ mystical design --book "POSD" --domain "library-design"
$ mystical design --book "Clean Architecture" --stance enforcer

# 性質を指定
$ mystical design --property "type safety, immutability"

# 異質適用を試す
$ mystical design --domain state-management --force-style alchemy

# 既存コードを別の式で再設計
$ mystical refactor --from src/state/reducer.rs --to-style alchemy

# ★v0.2 追加: 複数書籍の比較
$ mystical compare --books "POSD,Clean Architecture" --domain "module design"
```

### 6.3 出力形式

Designer Mode の出力:

1. 魔法陣の SVG (設計スケッチ)
2. 骨格コード (式に基づくテンプレート)
3. 設計ノート (なぜこの式か、トレードオフ)
4. 参照実装へのリンク
5. **★v0.2 追加: 適用される自動検査ルール** (スタンスに応じて)

---

## 7. リファクタリング支援としての式変換

### 7.1 式変換 (Style Refactoring)

既存コードの式を別の式に変換する、リファクタリングの新しい単位。Appendix E §2.7 (Refactoring by Fowler) の発展形として位置づけられる。

### 7.2 変換の難易度

式の変換難易度は、対称性のオーダーの距離に概ね比例:

- 同じオーダー内のモード変換: 容易
- 隣接オーダーの変換: 中程度
- 離れたオーダーの変換: 困難
- 夜天の書式への変換: 最も困難
- **★v0.2 追加: Profundus 系への変換**: モジュール再設計に近い大改修

### 7.3 変換の不可逆性

一部の式変換は情報を失う:

- 黒執事式 → Vesperia式: 静的契約の保証が失われる
- Alchemy式 → ベルカ式: 契約情報が失われる
- 夜天の書式 → ミッドチルダ式: メタレベルの構造が失われる
- **★v0.2 追加: Profundus → Shallow Module**: 内部の最適化が失われる

これらの不可逆変換を行う際は、ツールが警告を出す。

---

## 8. 実装ロードマップへの統合

### 8.1 Phase との対応

- **Phase 1**: Analyzer Mode のみ
- **Phase 2-3**: 基本的な Designer Mode (レシピ集から選んで骨格を出力)
- **Phase 4-5**: 異質式適用の自動推奨、ハイブリッド式生成、式変換
- **Phase 5+ (★v0.2 追加)**: Appendix E のレシピ統合、書籍別の `--book` オプション
- **Phase 6+**: 立体ビューでの合成可視化、AI 統合による新レシピの自動発見

### 8.2 既存仕様書への影響

(v0.1 と同様、各 Appendix の更新を予定)

---

## 9. 設計支援ツールとしての歴史的位置づけ

### 9.1 先行する設計支援の系譜

- CASE ツール (1980s-)
- パターンランゲージ (1977-)
- モデル駆動開発 (2000s-)
- DSL (2000s-)
- AI 支援開発 (2020s-)

### 9.2 AI との関係

```
[人間] → 求める性質を言語化
    ↓
[Mystical Designer] → 性質に対応する式を選択、骨格を生成
    ↓
[AI Code Assistant] → 骨格を埋める詳細コードを生成
    ↓
[Mystical Analyzer] → 生成されたコードが意図通りの式になっているか検証
```

本ツールは設計レベルの意図と実装レベルの現実を繋ぐ装置として、AI と補完関係に立つ。

---

## 10. ツールの最終的な位置づけの再定義

(v0.1 と同様)

**目的の二重化**: Analyzer Mode (現状診断) + Designer Mode (設計支援)

**価値の系譜**: 可視化ツール、パターン言語、設計支援ツール、AI支援開発と補完

**独自性**: 視覚的対称性に基づく式の体系、ファンタジー図像学、双方向性、異質式適用

**到達点**: コードベース全体の景観可視化、設計レシピ集と式駆動設計、立体ビュー、VR/AR 対応

---

## 11. ★v0.2 新規: ツールのスタンス選択

### 11.1 スタンスの概念

Appendix C v0.2 §6 で論じた「裁量 vs 規範の二項対立」を踏まえ、本ツール自身のスタンスをユーザーが選択できる仕組みを設ける。これは本ツールが**あらゆる開発文化に適合する**ための重要な設計判断である。

3つのスタンスが用意される:

```
Advisor Mode  ← 裁量重視 (デフォルト)
   ↓
Hybrid Mode   ← 中間
   ↓
Enforcer Mode ← 規範重視
```

スタンスは設定ファイル `mystical.toml` で宣言され、ツール全体の挙動を決定する。

### 11.2 Advisor Mode (アドバイザーモード)

**デフォルトのスタンス**。裁量モデル (POSD, Pragmatic Programmer 系) と親和性が高い。

**挙動**:
- 式の選択を推奨するが強制しない
- 違反を警告として表示するが、ビルドを通す
- 開発者の判断を信頼する
- 「Could be」「Consider」のような提案的な言葉を使う
- 自動修正は提案しない (人間が判断する)

**設定例**:
```toml
# mystical.toml
[stance]
mode = "advisor"

[advisor]
verbosity = "low"  # warning は最小限
auto_suggest = true  # 推奨は表示
auto_fix = false  # 自動修正なし
```

**適合する文化**:
- スタートアップ、少数精鋭チーム
- 個人開発、オープンソースの個人プロジェクト
- POSD, Lisp, Haskell など裁量重視の言語/哲学
- 実験的・探索的なフェーズ

### 11.3 Enforcer Mode (エンフォーサーモード)

オプション。規範モデル (Clean Architecture, Code Complete 系) と親和性が高い。

**挙動**:
- 特定の式をプロジェクト規約として強制
- 違反を CI でビルド失敗にする
- Dependency Rule のような強制ルールを定義可能
- 「Must」「Required」のような断定的な言葉を使う
- 自動修正を提案する場合もある

**設定例**:
```toml
[stance]
mode = "enforcer"

[enforcer]
project_style = "clean-architecture-fullstack"  # レシピ ID
required_dependency_direction = "inward"
forbidden_violations = ["dependency-inversion", "circular-dependency", "layer-skip"]
build_failure_on_violation = true
auto_fix_suggestions = true
strict_mode = true  # 警告も failure 扱い
```

**適合する文化**:
- エンタープライズ、大企業
- 長期保守の業務システム
- 多人数チーム、新人を含む構成
- Clean Architecture, DDD, Code Complete など規範重視の哲学
- 規制業界 (金融、医療、公共)

### 11.4 Hybrid Mode (ハイブリッドモード)

中間のスタンス。Clean-POSD Fusion レシピ (§4.5) に対応する。

**挙動**:
- 一部のルールは強制 (循環依存禁止、Dependency Rule 違反など重要なもの)
- 他は推奨に留める
- ズームレベルごとに異なるスタンスを適用可能 (システム全体は Enforcer、モジュール内部は Advisor)
- 領域別にスタンスを変更可能 (Core は Enforcer、Supporting は Advisor)

**設定例**:
```toml
[stance]
mode = "hybrid"

[hybrid]
# システム全体の構造は強制
[hybrid.system_level]
enforcement = "strict"
rules = ["dependency-rule", "cyclic-dependency-detection", "layer-isolation"]

# モジュール内部は裁量
[hybrid.module_level]
enforcement = "advisory"
rules_as_warnings = ["depth-score", "interface-complexity"]

# DDD 戦略的設計の領域別
[[hybrid.domain]]
name = "core-domain"
enforcement = "strict"  # Core Domain は規範

[[hybrid.domain]]
name = "supporting-subdomain"
enforcement = "balanced"  # Supporting は中間

[[hybrid.domain]]
name = "generic-subdomain"
enforcement = "advisory"  # Generic は裁量
```

**適合する文化**:
- 成熟した中〜大規模プロジェクト
- 多様なスキルレベルのチーム
- 段階的な品質向上を進めるフェーズ
- DDD を採用しているプロジェクト
- 現実の多くのチーム (純粋な Advisor / Enforcer はむしろ稀)

### 11.5 スタンス選択の戦略

**プロジェクトフェーズによる動的変更**:

```
プロトタイプ期 → Advisor Mode (素早く動かす)
   ↓
成長期 → Hybrid Mode (徐々に規律を導入)
   ↓
安定期 → Enforcer Mode (品質保守を優先)
   ↓
リファクタリング期 → Hybrid Mode (一部緩めて再構築)
   ↓
新パラダイム移行期 → Advisor Mode (探索的な変更を許容)
```

これは Phase 6 の時間軸スライダーと組み合わせて、**プロジェクトのライフサイクル全体を通じてスタンス変遷を追跡**できる。

**チーム構成による静的決定**:

```
シニア中心 (5人未満) → Advisor Mode
シニア + ミドル (5-15人) → Hybrid Mode
ミドル + ジュニア (15人以上) → Enforcer Mode
規制業界 → Enforcer Mode (規模に関わらず)
```

これはあくまでヒューリスティクスで、各チームが自分に合うスタンスを試行錯誤で見つける必要がある。

### 11.6 スタンス選択が促す自己認識

スタンス選択は単なる技術的決定ではなく、**チームの文化的自己認識**を促す。

「我々は Advisor か Enforcer か?」という問いに答えるには、チームの:
- 構成 (スキル分布、経験年数)
- ドメイン (安定性、変化速度)
- 開発文化 (実験的か保守的か)
- 顧客 (内部 vs 外部、規制有無)

を考慮する必要がある。これらを言語化することが、チームの設計哲学を自覚するきっかけになる。

これは技術的選択を超えて、**組織文化の鏡**として本ツールが機能することを意味する。Conway's Law の作用 (組織が設計を決める) を、ツール自身が**意識的に活用する**設計と言える。

### 11.7 スタンスの実装上の影響

各スタンスは、ツールの以下の動作に影響する:

| 動作 | Advisor | Hybrid | Enforcer |
|---|---|---|---|
| 違反時の出力 | warning | warning + error (重要度別) | error |
| ビルド | 通す | 一部 fail | fail |
| 自動修正の提案 | しない | 求められたら | 積極的に |
| メッセージのトーン | "Could be" | "Consider / Must" | "Must" |
| CI 統合 | 情報表示のみ | 一部チェック | フルチェック |
| 推奨レシピの強制度 | 0% | 部分的 | 100% |
| 学習曲線 | 自分のペース | 段階的 | 即座に規範学習 |

### 11.8 スタンスとレシピの相互作用

各レシピ (§4) は、推奨されるスタンスを宣言している:

- POSD レシピ → `stance: advisor`
- Clean Architecture レシピ → `stance: enforcer`
- Clean-POSD Fusion → `stance: hybrid`

ユーザーが指定したスタンスとレシピが推奨するスタンスが食い違う場合、ツールは警告を出す:

```
警告: 選択したレシピ "Clean Architecture" は Enforcer Mode を推奨していますが、
      現在のスタンスは Advisor Mode です。
      Enforcer Mode に切り替えるか、別のレシピを選択してください。
      
      代替案:
        - Clean-POSD Fusion (Hybrid Mode 対応)
        - POSD レシピ (Advisor Mode 対応)
```

これにより、ユーザーは自分の文化的選択 (スタンス) と技術的選択 (レシピ) の整合性を確認できる。

### 11.9 スタンスの設定可能性の階層

スタンスは以下の階層で設定可能 (上ほど優先):

1. CLI フラグ (`--stance enforcer`) — 一時的な上書き
2. プロジェクトの `mystical.toml` — プロジェクト規約
3. ユーザーの `~/.mystical/config.toml` — 個人のデフォルト
4. システムのデフォルト = Advisor Mode

これにより、組織のプロジェクトでは Enforcer を強制しつつ、開発者個人は実験中に CLI フラグで一時的に Advisor に切り替える、といった柔軟性が得られる。

### 11.10 スタンス選択の哲学的位置づけ

スタンス選択をツールが提供することは、**「裁量 vs 規範の選択権をユーザーに委ねる」**という、メタレベルの裁量モデルである。

つまり:
- 一階の選択: コードをどう書くか (Advisor/Enforcer の使い分け)
- 二階の選択: コードの書き方をどう決めるか (どのスタンスを採用するか) ← ツールが提供

本ツール自身は二階の選択においては裁量モデルを採用する (ユーザーが決める)。一階の選択では、ユーザーが選んだスタンスに従う。

これは「裁量 vs 規範」の対立を**入れ子化** (Strange Loop) することで超越する設計判断である。Hofstadter 的な自己参照構造が、ツール設計に活かされている。

---

## 12. (旧 §9 から移動) 設計支援ツールとしての歴史的位置づけ

v0.1 の §9 と同じ内容のため省略。

---

## 13. (旧 §10 から移動) ツールの最終的な位置づけの再定義

v0.1 の §10 を、v0.2 の追加要素 (スタンス選択、Appendix E 連携) を踏まえて再記述。

**目的の三重化** (v0.2):
- (a) Analyzer Mode (現状診断)
- (b) Designer Mode (設計支援)
- (c) ★v0.2 追加: スタンス選択 (文化適合) — メタレベルでの柔軟性

**価値の系譜**:
- 可視化ツールの延長
- パターン言語の進化形
- 設計支援ツールの現代版
- AI 支援開発と補完関係
- ★v0.2 追加: 既存設計知識資産との接続 (Appendix E)
- ★v0.2 追加: 組織文化の鏡 (スタンス選択)

**独自性**:
- 視覚的対称性に基づく式の体系
- ファンタジー図像学を借用した記号体系
- 双方向性 (Analyzer ⇄ Designer)
- 異質式適用による新パラダイムの引き寄せ
- ★v0.2 追加: 裁量 vs 規範の入れ子化 (Strange Loop)
- ★v0.2 追加: 設計書群の式体系への翻訳

**到達点**:
- コードベース全体の景観可視化
- 設計レシピ集と式駆動設計
- 立体ビューによる合成の表現
- VR/AR 対応の空間 UI
- ★v0.2 追加: 全ての開発文化に対応するスタンス選択

本ツールは「真面目に作れば」、コード可視化ツールの枠を超えて、**ソフトウェア設計と組織文化を媒介する記号体系**として機能する。それは Christopher Alexander が建築で行ったことの、ソフトウェア版である。

---

## バージョン履歴

v0.1 — 初版。ツールの双方向性、異質式適用、式駆動設計、レシピ集、式の合成、式変換、Designer Mode のインターフェース、設計支援ツールとしての歴史的位置づけを集約。

v0.2 — Clean Architecture, POSD, Clean-POSD Fusion の3レシピを §4 に正式追加。新セクション §11 (ツールのスタンス選択) を追加し、Advisor/Enforcer/Hybrid モードを正式仕様化。設定ファイル仕様、スタンス選択戦略、スタンスとレシピの相互作用、哲学的位置づけ (Strange Loop) を体系化。§10 を §13 にスライドし、目的の三重化、独自性、到達点を v0.2 の知見で再記述。
