# Mystical CI Appendix A: 描画様式カタログ v0.2

> **本文書の位置づけ**
> 本文書は『Mystical CI 仕様書 v0.1』および『Mystical CI Phase 2 以降の議論ノート v0.1』の補遺 A として、描画様式 (RenderStyle) の体系を詳細にカタログ化したものである。本ツールが扱う「式 (Style)」の網羅的な分類と、各式の技術的対応、自動推奨ヒューリスティクスを記述する。
>
> 本文書は仕様書本体ではなく**カタログ**として位置づけられ、新しい式の発見に伴って継続的に拡張される。
>
> **v0.2 での主な変更**: 深層式 (Profundus) の新規追加。これは Ousterhout の Philosophy of Software Design の分析から派生した、垂直方向の非対称性を扱う初めての式である。

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
    Aperiodic, // 周期性を持たない (深層式など) ★v0.2 追加
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
    Profundus,    // 入口は浅く、内部は深い (垂直非対称) ★v0.2 追加
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
| **深層式 (Profundus)** | **Aperiodic** | **Profundus** | **Ousterhout POSD ★v0.2 追加** |

### 1.4 偶奇による性格差

偶数オーダー (2, 4, 6, 8) は反転対称性 (mirror symmetry) を持ち、「対」を作りやすい構造であるため、入出力、対称分割、座標系、並列性を扱うのに向く。

奇数オーダー (3, 5) は反転対称性を持たず、循環的・非対称な変換に向く。関係性駆動、契約、変換チェーンを扱うのに向く。

これは群論的に C_n 群の構造的性質がそのまま「扱えるドメイン」を決めている現象として整理できる。

### 1.5 水平対称性と垂直対称性 ★v0.2 追加

これまでの式 (ベルカ〜夜天) は全て、平面上の**水平方向の対称性**を扱っていた。すなわち、紙面上で回転や反射により図像が不変になる構造である。

しかし、ソフトウェアには別の重要な軸が存在する: **垂直方向 (内外方向、深さ方向) の非対称性**。これは「表面は単純だが内部は複雑」「インターフェースは小さく実装は大きい」といった、深さ方向に情報密度が変化する構造である。

深層式 (Profundus) は、この垂直方向の非対称性を扱う初めての式として導入される。水平対称性の式群とは直交する軸を持ち、他の式と組み合わせる (合成する) ことで真価を発揮する。

---

## 2. 各式の詳細カタログ

### 2.1 ベルカ式 (Three, Convergent)

(v0.1 と同じ内容のため省略 - v0.1 を参照)

**図像的特徴**: 三つの円または三角形を三点に配置し、結んで大きな三角形を作る。中央に小さな収束点 (中心円) を持ち、そこに変換器が降ろされる。

**構造**: 3点+1中心 (合計4点)。3点が等格で、中心が変換を担う。

**技術ドメイン**: State + Action + Reducer、MVC、Port + Adapter + Domain、Command + Event + State、Producer + Channel + Consumer、Test + Spec + Implementation。

**実装 Phase**: Phase 3

### 2.2 うみねこ式 (Four, Orthogonal)

**図像的特徴**: 二重円の内側に十字を引いて4象限に分割。

**技術ドメイン**: 4-valued logic、HTTP ステータス分類、テスト結果、Eisenhower Matrix、enum/sum type/tagged union (4バリアント)、監査ログのトリアージ。

**実装 Phase**: Phase 4

### 2.3 Vesperia式 (Four, Organic)

**図像的特徴**: 4方向に円が配置され、内側に向かって有機的な装飾が伸びる。

**技術ドメイン**: AOP、React の Context・useEffect 連鎖、Pub/Sub、RxJS、Middleware chain。

**実装 Phase**: Phase 4

### 2.4 ミッドチルダ式 (Four/Eight, Balanced)

**図像的特徴**: 均等な円の多重連環、または内接する正四角形、または正方形を45度ずらして重ねた八芒星。各頂点にギリシャ文字や数学記号。

**技術ドメイン**: 関数空間の基底分解、構造化された OOP の階層、Trait + Impl の多重実装、レイヤードアーキテクチャ、関数型のコンビネータ合成。

**実装 Phase**: Phase 1 (ConcentricRings バリアント), Phase 2+ (他バリアント)

### 2.5 Alchemy式 (Five, Balanced)

**図像的特徴**: 五芒星 (正五芒星) + 中心の三角形 + 錬金術記号。

**技術ドメイン**: Design by Contract、Refinement Types、Hoare Logic、Rust の所有権システム、Haskell の型クラス制約、TLA+、Property-based test。

**実装 Phase**: Phase 5

### 2.6 黒執事式 (Five, Divergent)

**図像的特徴**: 五芒星 + 外側に放射状のトゲ・槍。

**技術ドメイン**: Rust の Move semantics、Drop trait、Affine/Linear type、Async cancellation、DB transaction commit、ファイナライザ、Append-only ログ。

**実装 Phase**: Phase 5

### 2.7 RWBY式 (Six, Divergent fractal)

**図像的特徴**: 雪片状の6方向放射構造。各腕が同形 (フラクタル)。

**技術ドメイン**: Rust の rayon、MapReduce、CUDA SIMD、Spark RDD、Actor モデル、関数型コンビネータ、並列フューチャー。

**実装 Phase**: Phase 5

### 2.8 Fate式 (Eight, Convergent + core)

**図像的特徴**: 八芒星 + 二重円 + 中央三点。外周にルーン文字。

**技術ドメイン**: ヘキサゴナルアーキテクチャの拡張、Clean Architecture、マイクロカーネル + 周辺サービス、OS の基本サブシステム、NestJS/Spring の DI コンテナ。

**実装 Phase**: Phase 5

### 2.9 夜天の書式 (Meta-recursive)

**図像的特徴**: 自己を内包する魔法陣。他の式の魔法陣を内部に含み、書き換えながら駆動する。

**技術ドメイン**: マクロ展開、コード生成、リフレクション、eval、JIT、DSL 解釈器、自己ホスティングコンパイラ。

**実装 Phase**: Phase 6+

### 2.10 深層式 (Profundus, Aperiodic) ★v0.2 新規追加

**図像的特徴**: 不規則なサイズのリングが並ぶ。各リングは、表面 (周囲) から見ると小さな円だが、ズームすると内部に広大で複雑な空間が広がっている。エッシャー的な逆遠近法、TARDIS 的な「外は小さく、中は広い」構造。

**構造**: 多数の独立したリングが並列に存在。リング同士の対称性や関係性より、個々のリングの**深さの比率**が主役。垂直方向 (内外方向) の極端な非対称性。

**表現する関係性**: モジュールの「深さ」 = 実装の豊かさ / インターフェースの小ささ。「Information Hiding」「Deep Module」「General-purpose Module」といった、Ousterhout の Philosophy of Software Design (POSD) の中核概念を視覚化する。

**深さスコアの定義**:
```
DepthScore(module) = ImplementationVolume / InterfaceSize
```
- 高い: Deep module (POSD が推奨)
- 低い: Shallow module (POSD が警告)

**技術ドメイン**:
- POSD (A Philosophy of Software Design) の Deep Module 思想
- Unix システムコール (小さな API、巨大な kernel 実装)
- 良くできた SDK (シンプルなインターフェース、複雑な内部)
- Lisp の defun (小さい構文、強力な実装)
- React の useState (極小 API、複雑な状態管理)
- Brian Kernighan & Dennis Ritchie の Unix 哲学
- 「単純なものに見えるが、その単純さこそが最大の到達点」というスタイル

**必要な解析**: 
- インターフェースサイズの計算 (公開シンボル数、パラメータ数、公開メソッド数)
- 実装サイズの計算 (行数、内部関数数、cyclomatic complexity)
- 隠蔽度の評価 (private vs public のバランス)
- コメント密度 (POSD はコメントを設計の一部と位置づける)

**推奨条件 (自動判定)**:
- 公開関数が少なく、private 関数が多い
- 公開関数のパラメータが少ない (理想は1〜3個)
- 1つの関数で複雑な処理を完結している
- API ドキュメントが充実している (POSD はコメント重視)
- `pub` 修飾子が選択的に使われている

**逆に Profundus 式から遠ざかる兆候 (Shallow Module の警告)**:
- 公開関数が多すぎる
- 多数のパラメータを取る公開関数 (>5個)
- 実装の中身が薄い (delegation only)
- 内部詳細が公開シンボルから漏れている (leaky abstraction)

**他の式との合成**: 
Profundus は単独でも有効だが、他の式と組み合わせることで真価を発揮する。これは Appendix D §5 で扱う「式の合成」の典型例となる。

- **Fate + Profundus** = Clean Architecture の各層が deep module
- **ベルカ + Profundus** = Reducer が deep module (Elm の Update 関数のように)
- **Vesperia + Profundus** = ミドルウェアそれぞれが deep module
- **RWBY + Profundus** = 並列ワーカーそれぞれが deep module (Erlang プロセス)

**実装 Phase**: Phase 5+ (深さスコアの計算とレンダリング)

**視覚化の挑戦**: 
Profundus は平面では表現が困難で、本質的に**立体性または対話的なズーム**を要求する。平面上では、リングの大きさで「深さ」を比例表現する近似手法が考えられるが、これは Phase 1-2 では十分でない。Phase 6 の立体ビューと、Phase 2 のズーム可能な dev-server の両方で初めて完全に表現できる。

このため、Profundus は本ツールが**フラクタル景観**と**立体ビュー**を実装する強い動機となる。Ousterhout の哲学を視覚化したいなら、平面で十分という考えを捨てなければならない。

**思想的意義**:
深層式は、これまでの式群が**水平方向の対称性**にのみ着目していたという、本ツールの暗黙の偏りを暴く。垂直方向の構造もまたコードの本質的な性質であり、それを扱う式が必要である、という気付きを与える。

これは Phase 6 以降に向けて、本ツールが「対称性の式」と「非対称性の式 (Profundus 系)」の両方を扱う、より完全な記号体系へと拡張されることを示唆する。Profundus は単一の式ではなく、**新しい式系の最初の一つ**として位置づけられる。

将来的には、Profundus 系の他の式 (例: 「螺旋式」「フラクタル次元式」「自己相似深さ式」など) が発見・追加される可能性がある。

---

## 3. 自動推奨ロジックの実装

(v0.1 と同じ内容のため省略 - v0.1 §3 を参照)

要約: AST 解析結果から各式への適合度をスコア化し、最高スコアの式を推奨する。スコアが拮抗する場合は複数候補を提示する。明示指定と自動推奨のミスマッチは警告として表示する。

Profundus 式については、深さスコア (DepthScore) を独立して計算し、他の式のスコアとは別チャネルで提示する。深層式は他の式と直交するため、両方のスコアを同時に表示することが意味を持つ。

---

## 4. RenderStyle の IR 統合

```rust
struct RenderStyle {
    order: SymmetryOrder,
    mode: ExpressionMode,
    variant: Option<StyleVariant>,
    depth_score: Option<f64>,    // ★v0.2 追加: Profundus 系で使用
}
```

`depth_score` フィールドは Profundus 式に固有の情報を保持する。他の式では `None` または常に固定値となる。

---

## 5. 未収集の組み合わせ

二軸分類モデルにおいて、(Order, Mode) の全ての組み合わせが既存ファンタジー作品の魔法陣として見つかっているわけではない。以下は現時点で未対応 (図像的参照が手元にない) の組み合わせ:

- (Two, *) — 2-fold は単純すぎて魔法陣として描かれにくい
- (Three, Divergent) — 3点放出
- (Six, Convergent) — 6方向から中心への集約
- (Six, Orthogonal) — 6つの離散カテゴリ
- (Eight, Organic) — 8方位の有機的伝播
- (Higher, *) — 12-fold, 16-fold などの高次対称性
- (Aperiodic, Convergent/Divergent/Orthogonal/Organic/Balanced) — Profundus 以外の Aperiodic 系 ★v0.2 追加

Profundus 系には他にも複数の式が存在しうる。例えば:

- **螺旋式 (Spiralis)**: ズームすると同じ構造が繰り返し現れる (フラクタル次元)
- **逆漏斗式 (Reverse Funnel)**: 入口は大きく内部に向かって狭くなる (Shallow → Deep の遷移)
- **多孔式 (Porous)**: 表面に多数の小さな穴があり、それぞれが別の空間に繋がる

これらは Phase 6+ の研究課題として記録する。

---

## 6. 命名と中立訳語

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
| **Profundus** ★v0.2 | **深層式** | **深層モジュール式** |

Profundus は元から学術用語的なラテン語 (「深い」を意味する) であり、Ousterhout の "Deep Module" 概念を直接反映している。作品由来名というより、技術用語としての命名であるため、特別な中立訳語は不要 (「深層式」「深層モジュール式」をそのまま使用)。

---

## 7. カタログの今後の拡張

新しい式の発見・追加は、以下のプロセスで行う:

1. 新しいファンタジー作品の魔法陣を観察、または新しい設計哲学から派生
2. (Order, Mode) の組み合わせとして分類
3. 既存式との重複でないことを確認
4. 対応する技術ドメインを言語化
5. 推奨ヒューリスティクスを定義
6. 本カタログに追記

Profundus 式は、ファンタジー作品ではなく**設計書 (POSD)** から派生した最初の式である。これは新しい式の発見元として「設計書・技術書」が有効な情報源であることを示している。Appendix E (設計哲学カタログ) は、この情報源を体系的に整理する文書として位置づけられる。

---

## 付録: 参照すべき作品リスト

未収集の図像で、収集価値が高いと思われる作品 (v0.1 と同じ):

- ロード・オブ・ザ・リング、ハリー・ポッター、ベルセルク、葬送のフリーレン、まどか☆マギカ、SAO、ダンジョン飯、メイドインアビス、七つの大罪、BLEACH

加えて、設計書からの式の派生:

- Patterns of Enterprise Application Architecture (Fowler)
- Domain-Driven Design (Evans)
- Functional Programming in Scala
- Effective Java (Bloch)
- Refactoring (Fowler)

これらは Appendix E で詳細に分析される。

---

## バージョン履歴

v0.1 — 初版。9式 + 中立訳語対応表を含む。

v0.2 — 深層式 (Profundus) の新規追加。垂直方向の非対称性を扱う初めての式として位置づけ。Aperiodic オーダーの追加。水平対称性と垂直対称性の区別を §1.5 として新設。他の式との合成パターンを示唆。設計書からの式の派生プロセスを §7 に追記。
