# MagiaMagica Appendix B: コードベース景観とモジュール結合 v0.1

> **本文書の位置づけ**
> 本文書は『MagiaMagica 仕様書 v0.1』、『MagiaMagica Phase 2 以降の議論ノート v0.1』、『MagiaMagica Appendix A: 描画様式カタログ v0.1』に続く補遺 B である。
>
> Appendix A で確立した「式 (RenderStyle)」の体系を、関数単位からモジュール単位・コードベース全体へとスケールアップさせる議論を集約する。本ツールの最終形態として想定される「魔導都市的可視化」の基礎理論と実装方針を記述する。

---

## 1. スケールアップの構造

### 1.1 これまでのスケール

Phase 1 仕様書および Appendix A までで議論してきた式 (RenderStyle) は、いずれも**関数1つ**を描画単位としていた。1つの関数 = 1つの魔法陣 = 1つの式。

### 1.2 観察される事実

実際のコードベースでは、モジュール内の関数群は**同じ性格を共有することが多い**。これは設計者が無意識に従っている設計原則の表れである。

- 状態管理モジュールの関数は全て Reducer 的な3項変換を扱う
- API ハンドラ群は全て分類的ディスパッチを扱う
- リソース管理モジュールは全て不可逆操作を扱う
- 並列処理モジュールは全て自己相似的な分割を扱う

この観察から、**モジュール単位でも式を識別できる**という仮説が導かれる。

### 1.3 階層的な式

ツールは以下の階層で式を扱う:

```
Project (System)
  └── Module (主たる式を持つ)
        └── Function (固有の式を持つ)
              └── Operation (式の最小構成要素)
```

各階層で、その階層に固有の式が決定される。関数の式とそれを含むモジュールの式は、通常は整合するが、稀にミスマッチが起きることもある。このミスマッチ自体が診断情報となる。

---

## 2. モジュール単位の式選択

### 2.1 集約ロジック

モジュールの主たる式は、内部の関数群の式スコアを集約することで決定する。

```rust
fn infer_module_style(module: &Module) -> RenderStyle {
    let mut aggregate_scores: HashMap<RenderStyle, f64> = HashMap::new();
    
    for sigil in &module.sigils {
        for (style, score) in &sigil.style_scores {
            *aggregate_scores.entry(style.clone()).or_insert(0.0) += score;
        }
    }
    
    // 関数数で正規化
    for score in aggregate_scores.values_mut() {
        *score /= module.sigils.len() as f64;
    }
    
    // 最高スコアの式を採用
    aggregate_scores.into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(style, _)| style)
        .unwrap_or(RenderStyle::default())
}
```

### 2.2 一貫性スコア

集約だけでなく、**モジュール内の式の一貫性**もスコア化する。

```rust
fn module_consistency(module: &Module) -> f64 {
    let dominant_style = infer_module_style(module);
    let matching = module.sigils.iter()
        .filter(|s| s.preferred_style == Some(dominant_style.clone()))
        .count();
    matching as f64 / module.sigils.len() as f64
}
```

一貫性が高い (例: 90%以上の関数が同じ式) モジュールは設計上の凝集度が高い。低い (50%未満など) モジュールは責務が雑居している可能性がある。

### 2.3 ディレクトリ構造からのヒント

ファイルシステムのディレクトリ構造も、モジュールの式の推定に使える。

```
src/
├── state/         → ベルカ式の確率を上げる
├── handlers/      → うみねこ式の確率を上げる
├── workers/       → RWBY式の確率を上げる
├── di/            → Fate式の確率を上げる
├── resources/     → 黒執事式の確率を上げる
├── middleware/    → Vesperia式の確率を上げる
└── domain/        → Alchemy式の確率を上げる
```

ディレクトリ名はあくまでヒントで、最終的な判定は AST 解析が優先される。ただし、ディレクトリ名と AST 解析の結果が食い違う場合は、命名の妥当性に疑問符が付く (例: `state/` というディレクトリにベルカ式以外の関数が多い)。

---

## 3. 結合点 (Junction Glyph)

### 3.1 概念

異なる式を持つモジュール同士を繋ぐとき、単純な線で繋いだだけでは図像として歪む。これは設計上も「両側の語彙を翻訳する層が必要」という現実を反映している。

**結合点 (Junction Glyph)** とは、異なる式の魔法陣の間に置かれる小さな魔法陣で、両側の式の特徴を部分的に含み、片側から見ても反対側から見ても自然に接続される図像を持つ。

### 3.2 構造

```rust
struct JunctionGlyph {
    self_module: ModuleId,
    other_module: ModuleId,
    self_side: PartialStyle,       // 自モジュールの式の接続面
    other_side: PartialStyle,      // 相手モジュールの式の接続面
    translation_logic: TranslationKind,
    quality: Option<JunctionQuality>,
}

enum TranslationKind {
    Adapter,            // Adapter パターン
    Facade,             // Facade パターン
    AntiCorruption,     // DDD の Anti-Corruption Layer
    Mapper,             // データ変換
    Gateway,            // API Gateway
    Bridge,             // Bridge パターン
}

struct PartialStyle {
    full_style: RenderStyle,
    exposed_features: Vec<StyleFeature>,
}

enum StyleFeature {
    Vertex(VertexId),      // ベルカ式の三角形の一頂点
    Arm(ArmId),            // うみねこ式の十字の一腕
    Petal(PetalId),        // Vesperia 式の有機的装飾の一葉
    Axis(AxisId),          // ミッドチルダ式の正方形の一軸
    Spike(SpikeId),        // 黒執事式の放射トゲの一本
    Crystal(CrystalId),    // RWBY式の雪片の一腕
    Port(PortId),          // Fate式の八方位の一ポート
    Arc(ArcId),            // Alchemy式の五芒星の一辺
}
```

### 3.3 図像的な接続

ベルカ式 (三角形) とうみねこ式 (十字) を繋ぐ結合点の例:

```
ベルカ式モジュール側:
    ベルカ式の三角形の一頂点が、結合点に向かって伸びる

結合点 (Junction Glyph):
    三角形の頂点を受け取る入口と、
    十字の腕を受け取る入口を持つ、
    小さな多角形の魔法陣
    内部に「翻訳記号」(変換のシジル) を持つ

うみねこ式モジュール側:
    うみねこ式の十字の一腕が、結合点から伸びてくる
```

両側の式の特徴を半分ずつ持つキメラ的な小魔法陣として描画される。これはファンタジー的にも整合的で、「異なる魔術体系を繋ぐ翻訳の儀式陣」というメタファーは古典オカルト文献にも存在する概念である。

### 3.4 結合点の品質指標

結合点の品質は以下の観点で評価する:

**接続面の整合性**: 両側の式の特徴が、結合点上で滑らかに繋がっているか。歪んだ接続 (両側が衝突している) は警告対象。

**翻訳ロジックの明示性**: 結合点に明示的な変換ロジック (Adapter コードなど) があるか。直接呼び出しで両側を繋いでいる場合は、Anti-Corruption Layer の不在として警告。

**情報量の保持**: 両側の式の情報量が、結合点を通過する際に大きく欠落していないか。過剰な抽象化により情報が失われている結合点は警告対象。

**双方向性**: 結合点が両側を理解しているか。片側だけを理解している結合点 (一方の語彙を強制している) は、leaky abstraction の兆候。

---

## 4. 結合トポロジー

モジュール間の結合は、いくつかの典型的なトポロジーに分類できる。各トポロジーは図像的に異なる特徴を持つ。

### 4.1 直列結合 (Pipeline)

```
[Module A] → [Module B] → [Module C]
```

データやイベントが一方向に流れる。各モジュールの魔法陣が**矢のように連結**され、流れる方向が明確に視覚化される。

**対応する技術**:
- ETL パイプライン
- 関数合成 (`f . g . h`)
- Unix パイプ
- Middleware chain

**図像的特徴**: モジュール魔法陣の間に矢印型の結合点。全体は左から右へ、または上から下へ向かう流れとして描かれる。

### 4.2 並列結合 (Fan-out / Fan-in)

```
            ┌→ [Module B] ─┐
[Module A] ─┼→ [Module C] ─┼→ [Module D]
            └→ [Module E] ─┘
```

中央から放射状に分岐し、再集約する。Scatter-Gather パターン。

**対応する技術**:
- MapReduce
- 並列フューチャー (`tokio::join!`)
- ファンアウト/ファンイン
- 分散処理

**図像的特徴**: 中央のモジュールから放射状に他モジュールへ繋がる線。RWBY式の雪片構造に似た全体像。

### 4.3 階層結合 (Containment)

```
[Outer Module]
    └── [Inner Module]
            └── [Innermost Module]
```

モジュールが別のモジュールを内包する。

**対応する技術**:
- クラスとそのプライベートヘルパー
- Facade パターンと内部実装
- Nested module structure
- Microservices の集約

**図像的特徴**: 外側の大きな魔法陣の**中に**小さな魔法陣がフラクタル的に埋め込まれる。ズームすると内部の魔法陣が見える。

### 4.4 網状結合 (Mesh)

```
[A] ←→ [B]
 ↕      ↕
[C] ←→ [D]
```

モジュール同士が相互に呼び合う。

**対応する技術**:
- マイクロサービス
- Actor モデルでの相互通信
- P2P システム

**図像的特徴**: 各魔法陣の周囲を結合点が網の目状に繋ぐ構造。図像が複雑になるため、ズームレベルの制御が重要。

### 4.5 環状結合 (Cycle)

```
[A] → [B] → [C] → [A]
```

モジュール間に循環依存がある。

**対応する技術**:
- 循環依存 (多くの場合バグまたは設計問題)
- 一部の関数型データ構造 (相互再帰)

**図像的特徴**: 閉じたループとして描かれる。**ループは赤系統の色で警告色を付ける**。アーキテクチャ上の問題のサイン。

### 4.6 ハブ&スポーク結合 (Hub-and-Spoke)

```
        [A]
         │
[B] ── [Hub] ── [D]
         │
        [C]
```

中央のハブモジュールに全てが接続する。

**対応する技術**:
- API Gateway
- メッセージブローカー (Kafka, RabbitMQ)
- イベントバス
- DI コンテナ

**図像的特徴**: 中央に Fate式 (8-fold + core) の大きな魔法陣、周囲に各種式のモジュール魔法陣が配置される。

---

## 5. フラクタルなズームレベル

### 5.1 ズーム階層

可視化は以下の階層を持つ:

**Zoom Level 1: プロジェクト全体俯瞰**

プロジェクト全体を一望する。各モジュールは点または小さな円として配置され、結合関係が線として描かれる。配置アルゴリズムは:

- Voronoi 分割で各モジュールに領域を割り当て
- または Treemap 状にタイリング
- または Force-directed layout

**Zoom Level 2: モジュール単位**

単一モジュールにフォーカス。そのモジュールの主たる式で描かれた大きな魔法陣が表示され、内部の関数群が構成要素として配置される。隣接モジュールは外周にぼかして表示される。

**Zoom Level 3: 関数単位**

Phase 1 仕様書で扱う粒度。関数1つの魔法陣。

**Zoom Level 0 (将来): エコシステム俯瞰**

複数プロジェクトの集合体。組織レベル、企業レベルの可視化。Phase 6 以降の遠い将来。

### 5.2 連続的なズーム

各レベル間は連続的にズームできる。ズームアウトすると関数の魔法陣がモジュールの魔法陣の一部のリングになり、さらにズームアウトするとモジュールの魔法陣がプロジェクト全体の景観の一点になる。

これは**完全なフラクタル可視化**で、Google Maps の都市から建物までの連続ズームと同じ体験を提供する。

### 5.3 ズームに伴う情報の段階的開示

ズームレベルに応じて、表示される情報の粒度が変わる。

| Level | 表示される情報 |
|---|---|
| 0 (エコシステム) | プロジェクト同士の関係性 |
| 1 (プロジェクト) | モジュールの式、結合トポロジー |
| 2 (モジュール) | 主たる式、内部関数の配置、結合点 |
| 3 (関数) | 制御フロー、副作用、外部呼び出し |
| 4 (式) | 個々の Operation、記号の詳細 |

ユーザーは関心に応じてズームレベルを選び、必要な情報だけを取得する。これは Phase 1 仕様書の設計原則「デフォルトは最も単純な射影を提示する」と整合する。

---

## 6. 全体景観の構築

### 6.1 配置アルゴリズム

プロジェクト全体を Zoom Level 1 で描画するとき、各モジュールをどう配置するか。複数の選択肢がある。

**選択肢A: 結合グラフベース (Force-Directed)**

モジュール間の結合関係をエッジ、各モジュールをノードとして、Force-Directed Layout で配置する。結合が強いモジュールは近くに、弱いモジュールは遠くに配置される。

利点: 結合の強弱が距離として直感的に見える。
欠点: 配置が不安定 (小さな変更で大きく動く可能性)。

**選択肢B: ディレクトリ構造ベース (Treemap)**

ファイルシステムのディレクトリ階層に従って、Treemap 状にタイリングする。

利点: 安定した配置、開発者にとって馴染みのある構造。
欠点: 結合関係が距離に反映されない。

**選択肢C: アーキテクチャ層ベース (Layered)**

レイヤードアーキテクチャに従って、垂直方向に層を作る (UI → Application → Domain → Infrastructure)。

利点: アーキテクチャ違反 (層を飛び越える依存) が視覚的に明白。
欠点: レイヤード構造を採用していないプロジェクトに不向き。

**選択肢D: ハイブリッド**

主たる配置はディレクトリベース、結合の強弱は線の太さと色で表現、層構造があれば縦軸として補助的に表現。

推奨は選択肢D。安定性、可読性、表現力のバランスが取れる。

### 6.2 景観の美的統一

異なる式の魔法陣が並ぶとき、全体としての景観の統一感をどう確保するか。

- カラーパレットの統一 (全式で同じ基調色 + アクセント色)
- 線の太さの整合 (関数レベル < モジュールレベル < プロジェクトレベル)
- フォント・記号スタイルの統一
- 背景色・グリッドの統一

これらは Phase 6 のデザインシステムとして整備される。Tailwind CSS や Material Design のような、内部一貫性を保つデザイントークンセットが必要。

---

## 7. 新しい品質指標

モジュール単位の式と結合点が IR に組み込まれることで、新しい品質指標が定義可能になる。

### 7.1 式の一貫性 (Style Consistency)

**定義**: モジュール内の関数のうち、モジュール主たる式と整合する関数の割合。

**意味**: 高いほどモジュールの凝集度が高い。低いと責務が雑居している可能性。

**しきい値の例**: 80%以上を「良い」、50-80%を「許容」、50%未満を「警告」。

### 7.2 結合点の健全性 (Junction Health)

**定義**: モジュール間の結合点が、明示的な変換ロジック (Adapter, Facade等) を持っている割合。

**意味**: 高いほどモジュール境界がクリーン。低いと leaky abstraction の兆候。

### 7.3 式の遷移の滑らかさ (Style Transition Smoothness)

**定義**: 隣接モジュール間で式が大きく変わる箇所のうち、適切な結合点 (Anti-Corruption Layer等) が介在している割合。

**意味**: 高いほどアーキテクチャ境界が明示的。低いと継ぎ接ぎの寄せ集め。

### 7.4 式の局所性 (Style Locality)

**定義**: 同じ式を持つモジュールが、ディレクトリ構造上で集まっている度合い。

**意味**: 高いと責務が地理的に集約されている。低いと同種の責務がプロジェクト全体に散らばっている。

### 7.5 環状結合の存在 (Cyclic Coupling)

**定義**: モジュール間の循環依存の有無と数。

**意味**: 多くの場合バグまたは設計問題。0が理想。

### 7.6 ハブの過剰集中 (Hub Centrality)

**定義**: 一つのモジュールに対する結合数の偏り (Gini 係数的な指標)。

**意味**: 過剰に集中しているとモノリシックな god module の兆候。

### 7.7 複合指標としての景観スコア (Landscape Score)

上記の指標を統合した総合スコア。プロジェクトの**アーキテクチャ・ヘルス**を一つの数値で表す。CI で時系列追跡することで、アーキテクチャの劣化を早期検出できる。

```
LandscapeScore = w1 * StyleConsistency
               + w2 * JunctionHealth
               + w3 * TransitionSmoothness
               + w4 * StyleLocality
               - w5 * CyclicCoupling
               - w6 * HubCentrality
```

重み w_i はチーム/ドメインによって調整可能。

---

## 8. IR スキーマへの拡張

Phase 1 仕様書 §4.2 の IR スキーマを、本文書の議論に対応するように拡張する。

```rust
struct MagiaGraph {
    modules: Vec<Module>,
    cross_module_edges: Vec<Edge>,
    junction_glyphs: Vec<JunctionGlyph>,        // 新規
    project_metadata: ProjectMetadata,
    landscape_metrics: Option<LandscapeMetrics>, // Phase 3+で埋まる
}

struct Module {
    id: ModuleId,
    name: String,
    sigils: Vec<Sigil>,
    edges: Vec<Edge>,
    inferred_style: Option<RenderStyle>,
    style_consistency: Option<f64>,
    junction_points: Vec<JunctionPointRef>,
}

struct JunctionGlyph {
    // §3.2 で定義済み
}

struct LandscapeMetrics {
    style_consistency_avg: f64,
    junction_health_avg: f64,
    transition_smoothness: f64,
    style_locality: f64,
    cyclic_couplings: Vec<CyclicLoop>,
    hub_centrality_gini: f64,
    landscape_score: f64,
}
```

Phase 1 ではこれらのフィールドは全て `None` または空。Phase 2 以降で段階的に埋まる。

---

## 9. レンダリングへの含意

### 9.1 階層的なレンダリングパス

ズームレベルに応じて、異なるレンダリングパスを実装する必要がある。

**Pass 1 (Zoom 3: 関数)**: Phase 1 で既に実装。単一関数の SVG。

**Pass 2 (Zoom 2: モジュール)**: モジュールの主たる式で大きな魔法陣を描き、内部に関数の小魔法陣を配置。Phase 2 で実装。

**Pass 3 (Zoom 1: プロジェクト)**: モジュール群の景観を描画。配置アルゴリズム、結合点、トポロジーが反映される。Phase 4 で実装。

### 9.2 結合点のレンダリング

結合点は、両側の式の特徴を半分ずつ持つキメラ的な図像として描かれる。これは個別の式のレンダラとは別の専用レンダラが必要。

```rust
trait StyleRenderer {
    fn render_module(&self, module: &Module) -> SvgGroup;
    fn render_junction_face(&self, partial: &PartialStyle) -> SvgGroup;
}

struct JunctionRenderer {
    self_renderer: Box<dyn StyleRenderer>,
    other_renderer: Box<dyn StyleRenderer>,
}

impl JunctionRenderer {
    fn render(&self, junction: &JunctionGlyph) -> SvgGroup {
        // 両側の式の特徴を半分ずつ描画し、中央で接続
        let self_face = self.self_renderer.render_junction_face(&junction.self_side);
        let other_face = self.other_renderer.render_junction_face(&junction.other_side);
        let translation_symbol = self.render_translation(&junction.translation_logic);
        compose(self_face, translation_symbol, other_face)
    }
}
```

### 9.3 SVG の階層構造

最終的な SVG は、ズームレベルごとに分離された階層構造を持つ。

```xml
<svg>
  <g class="zoom-level-1 project-landscape">
    <g class="module" data-style="midchilda">...</g>
    <g class="module" data-style="belka">...</g>
    <g class="junction" data-translation="adapter">...</g>
  </g>
  <g class="zoom-level-2 module-detail" style="display:none">
    ...
  </g>
  <g class="zoom-level-3 function-detail" style="display:none">
    ...
  </g>
</svg>
```

CSS による表示切替で、ズーム操作を実装する。Phase 2 の対話的可視化の基盤となる。

---

## 10. ファンタジー的メタファーの整合性

### 10.1 大儀式・合同魔法陣

ファンタジー作品において、複数の魔法陣を組み合わせる「大儀式」「合同魔法陣」「結界の重ね合わせ」といった概念は普遍的に存在する。

- 複数の術者による合同詠唱
- 結界術の重ね掛け
- 召喚儀式における準備陣 + 召喚陣 + 制約陣
- 古代の遺跡における巨大魔法陣

これらは技術的には「複数のモジュールが連携して一つの機能を実現する」状況と同型である。本ツールの「コードベース景観」は、ファンタジーの大儀式の構造を、現代のソフトウェアアーキテクチャに転用したものとして位置づけられる。

### 10.2 結界としての境界

DDD の Bounded Context は、しばしば「境界」として図示される。これは魔術における「結界」と概念的に同型である。

- Bounded Context = 結界 (内側と外側で意味が変わる空間)
- Anti-Corruption Layer = 結界の境界における翻訳の儀式
- Context Map = 複数の結界が共存する世界地図

これらの対応関係は偶然ではなく、「複雑な関係性を空間で理解する」という認知パターンに根ざしている。詳細は Appendix C で議論する。

---

## 11. 実装ロードマップ

### Phase 1 (現行)
モジュール構造を IR に保持するが、式は未推定。単一関数の描画のみ。

### Phase 2
モジュールごとの主たる式を推定。Zoom Level 2 のレンダリング実装。dev-server 化と対話的レイヤー切替。

### Phase 3
結合点の検出と図像化。データフロー解析の導入による、より高精度な式判定。

### Phase 4
Zoom Level 1 のプロジェクト全体景観レンダリング。配置アルゴリズム実装。

### Phase 5
新しい品質指標 (景観スコア) の算出と CI 統合。動的解析の合流による精度向上。

### Phase 6
立体ビューと連携。Zoom Level 0 のエコシステム可視化検討。VR/AR 対応。

---

## バージョン履歴

v0.1 — 初版。モジュール結合と景観の概念を Phase 2 以降のノートの延長として体系化。結合点、トポロジー、ズームレベル、品質指標を提案。
