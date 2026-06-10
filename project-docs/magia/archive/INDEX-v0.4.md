# Mystical CI プロジェクト文書 INDEX v0.4

## プロジェクト概要

**Mystical CI** は、ソースコードを魔法陣状の図形に変換することで、コードレビューとリファクタリングにおける注意誘導を支援する、**三重の目的**を持つコード可視化・設計支援ツールである。

ツールの三重目的:

- **Analyzer Mode**: コードを解析し、対応する魔法陣を生成する (現状診断)
- **Designer Mode**: 求める性質から魔法陣を選び、骨格コードを生成する (設計支援)
- **Stance Selection**: ツール自身のスタンス (Advisor / Enforcer / Hybrid) を選択することで、あらゆる開発文化に適合する (文化適合)

裁量重視と規範重視の両方の開発文化に適合する、メタレベルの柔軟性を持つ。本ツールは**ソフトウェア設計と組織文化を媒介する記号体系**として機能する。

---

## 文書構造 (v0.4 更新)

本プロジェクトに関する文書は、以下のファイル群で構成される。

### 中核文書

**`mystical-ci-spec-v0.1.md`** — Mystical CI 仕様書 v0.1

Phase 1 のスコープを厳密に定義した正式仕様書。実装時の主参照文書。

**`mystical-ci-phase2plus-notes-v0.1.md`** — Phase 2 以降の議論ノート v0.1

Phase 2 から Phase 6 までの構想を集約。長期ロードマップの主参照文書。

### 補遺文書 (現行バージョン)

**`mystical-ci-appendix-A-rendering-styles-v0.2.md`** — Appendix A: 描画様式カタログ v0.2

ツールが扱う「式 (RenderStyle)」の体系をカタログ化。10式 (深層式 Profundus を含む) と二軸分類モデル。

**`mystical-ci-appendix-B-codebase-landscape-v0.1.md`** — Appendix B: コードベース景観とモジュール結合 v0.1

関数単位の式をモジュール単位・プロジェクト単位にスケールアップさせる議論。結合点 (Junction Glyph)、結合トポロジー、フラクタル景観、品質指標。

**`mystical-ci-appendix-C-theoretical-foundations-v0.2.md`** — Appendix C: 理論的基礎 v0.2 ★更新

群論、視覚認知科学、計算理論、ソフトウェア工学、哲学からの正当化を集約。**v0.2 で「裁量と規範の二項対立」セクション (§6) を追加**し、POSD vs Clean Architecture をソフトウェア哲学史の系譜 (Brooks vs Parnas、C vs Pascal、Lisp vs Java、Perl vs Python、Vim vs IDE、Rails vs Sinatra、Static vs Dynamic Typing、Monolith vs Microservices) の現代的現れとして位置づけ。

**`mystical-ci-appendix-D-design-mode-v0.2.md`** — Appendix D: 設計支援モードと式駆動設計 v0.2 ★更新

ツールの双方向性を確立する文書。**v0.2 で Clean Architecture / POSD / Clean-POSD Fusion の3レシピを §4 に正式追加**、新セクション §11 として Advisor / Enforcer / Hybrid モードのスタンス選択を正式仕様化。

**`mystical-ci-appendix-E-design-philosophies-v0.1.md`** — Appendix E: 設計哲学カタログ v0.1

13の主要設計書を本ツールの式体系に翻訳して分類・分析する文書。既存設計知識資産との接続点。

---

## 文書間の依存関係 (v0.4 更新)

```
[Spec v0.1] ── 実装の基盤
    │
    ├──> [Phase 2+ Notes] ── 将来構想の集約
    │         │
    │         ├──> [Appendix A v0.2] ── 描画様式の詳細 (Profundus 含む 10式)
    │         │         │
    │         │         ├──> [Appendix D v0.2] ── 設計支援への展開
    │         │         │         │   (Clean Arch/POSD/Fusion レシピ追加、
    │         │         │         │    Advisor/Enforcer/Hybrid スタンス)
    │         │         │         │
    │         │         │         ├──> [Appendix E v0.1] ── 設計哲学カタログ
    │         │         │         │      ↑     │
    │         │         │         │      │     ↓
    │         │         │         │  レシピ供給 各書籍のレシピ化
    │         │         │         │
    │         │         │         └──> [Appendix C v0.2] §6
    │         │         │                  ↑
    │         │         │                  │ スタンス選択の哲学的根拠
    │         │         │
    │         │         ├──> [Appendix B v0.1] ── 景観論
    │         │         │         │
    │         │         │         └──> [Appendix E v0.1] §2.2, §2.3
    │         │         │              (Clean Architecture, DDD は景観論と直結)
    │         │         │
    │         │         └──> [Appendix C v0.2] ── 理論的基礎
    │         │                    │
    │         │                    ├──> [Appendix E §3, §6] と接続
    │         │                    │    (書籍間の関係マップ、哲学史的位置づけ)
    │         │                    │
    │         │                    └──> §6 が [Appendix D §11] と接続
    │         │                         (Strange Loop による
    │         │                          裁量 vs 規範の入れ子化)
    │
    └──> 将来の Spec v0.2 (Phase 2 着手時)
              ├── Phase 2+ Notes から昇格
              ├── Appendix A v0.2 の確定部分を取り込み
              ├── Appendix B の確定部分を取り込み
              ├── Appendix D v0.2 の Designer Mode + スタンス仕様を取り込み
              └── Appendix E のレシピ統合機構を取り込み
```

---

## 読み方ガイド (v0.4 更新)

### ケース 1: 実装に着手する開発者

1. `mystical-ci-spec-v0.1.md` を熟読
2. 必要に応じて `mystical-ci-phase2plus-notes-v0.1.md` で将来計画を確認
3. Appendix A v0.2 の §1 (二軸分類モデル) で式の体系を把握
4. Appendix C v0.2, D v0.2, E v0.1 は実装には直接必要ないが、設計の意図を理解したい場合に参照

### ケース 2: プロジェクトの全体像を理解したい新規参画者

1. 本 INDEX 文書 (現在読んでいる)
2. `mystical-ci-spec-v0.1.md` の §1〜§2
3. `mystical-ci-phase2plus-notes-v0.1.md` の §1 と §11
4. Appendix A v0.2 の §1 と §2 のいくつかの式
5. Appendix B の §1〜§3
6. Appendix D v0.2 の §1 (ツールの双方向性) と §11 (スタンス選択)
7. Appendix E v0.1 の §1 と §2.1, §2.2 (POSD と Clean Architecture)
8. Appendix C v0.2 §6 (裁量 vs 規範) は哲学的背景に興味があれば

### ケース 3: 設計の哲学的根拠を知りたい技術者

1. `mystical-ci-spec-v0.1.md` の §2 (設計原則)
2. `mystical-ci-phase2plus-notes-v0.1.md` の §11
3. Appendix C v0.2 全文 (特に §6 が v0.2 の目玉)
4. Appendix D v0.2 の §9, §11 (歴史的位置づけとスタンス選択の哲学的位置づけ)
5. Appendix E v0.1 §3 (書籍間の関係マップ) と §6 (時代的変遷)

### ケース 4: 描画様式 (式) の追加・変更を提案する人

1. Appendix A v0.2 全文
2. Appendix C v0.2 §1〜§2
3. Appendix D v0.2 §2 (異質式適用)
4. Appendix E v0.1 の §2 (各書籍が対応する式を確認、新式の発見元として書籍を活用)

### ケース 5: モジュール結合や景観可視化の改善を提案する人

1. Appendix B 全文
2. Appendix C v0.2 §5
3. `mystical-ci-phase2plus-notes-v0.1.md` の §4
4. Appendix D v0.2 §5 (式の合成)
5. Appendix E v0.1 §2.2-2.3 (Clean Architecture, DDD は景観論と直結)

### ケース 6: 設計支援機能を活用したい開発者

1. **Appendix D v0.2 全文** が中心 (特に §4 レシピ集、§11 スタンス選択)
2. Appendix A v0.2 (式の体系を理解)
3. Appendix B (式の合成と景観の理解)
4. Appendix E v0.1 の §4 (推奨される読書順序、自分のレベルに合う設計書を選ぶ)

### ケース 7: 新しいレシピを提案したい人

1. Appendix D v0.2 §4 (レシピ集) と §3 (式駆動設計)
2. Appendix E v0.1 §5 (ツールへの統合)
3. Appendix A v0.2 (式の選択肢を理解)
4. 提案レシピの基盤となる設計書を Appendix E で確認

### ケース 8: 既存の設計知識を活用したい開発者

1. **Appendix E v0.1 全文** が中心
2. 自分の状況 (ズームレベル × 文化スタンス) に応じた書籍を §4 から選ぶ
3. 対応する式を Appendix A v0.2 で確認
4. ツールの Designer Mode で実装に活用 (Phase 3+)

### ケース 9: ツールのスタンスを検討するチーム ★v0.4 新規

1. **Appendix D v0.2 §11 (ツールのスタンス選択)** が中心
2. Appendix C v0.2 §6 (裁量 vs 規範の二項対立) で哲学的背景を理解
3. 自チームの構成 (規模、スキル分布、ドメイン安定性) を Appendix C v0.2 §6.2-6.3 のチェックリストで評価
4. Appendix D v0.2 §11.5 (スタンス選択の戦略) で具体的な指針を確認
5. プロジェクト規約として `mystical.toml` を作成 (Appendix D v0.2 §11.9 参照)

### ケース 10: 哲学・組織論との接続に興味がある研究者 ★v0.4 新規

1. Appendix C v0.2 §6 (二項対立の系譜)
2. Appendix C v0.2 §10 (関連分野リスト、哲学・組織論を含む)
3. Appendix D v0.2 §11.10 (Strange Loop による超越)
4. Appendix E v0.1 §3, §6 (書籍間の関係と時代的変遷)
5. 関連: Hofstadter, Conway, Bohr の相補性原理など

### ケース 11: ユーザー (将来の)

実装後にユーザー向けドキュメント (チュートリアル、CLI リファレンス、Cookbook) が別途整備される予定。

---

## バージョン管理方針

(v0.1 から変更なし)

各文書は独立してバージョン管理。INDEX 文書自体もバージョン管理対象。

### v0.4 における変更

- Appendix C: v0.1 → v0.2 (裁量 vs 規範の二項対立を §6 として追加)
- Appendix D: v0.1 → v0.2 (Clean Arch/POSD/Fusion レシピ追加、スタンス選択 §11 を追加)
- 文書数は 7 のまま (バージョン更新のみ)

これで、当面の補遺シリーズ A-E が出揃った。当面は Appendix E の継続的な拡張、および新しい補遺 (F以降) の追加が主たる進化となる。

---

## 今後追加が想定される文書 (v0.4 更新)

### 短期 (Phase 1 実装期)

- `mystical-ci-implementation-guide-v0.1.md` — Phase 1 実装ガイド
- `mystical-ci-ir-schema-v0.1.md` — IR スキーマ詳細仕様

### 中期 (Phase 2-3 実装期)

- `mystical-ci-filter-dsl-spec-v0.1.md`
- `mystical-ci-layout-algorithm-v0.1.md`
- `mystical-ci-glyph-symbol-catalog-v0.1.md`
- `mystical-ci-designer-mode-spec-v0.1.md` — Appendix D v0.2 から昇格した正式仕様
- `mystical-ci-recipe-collection-v0.1.md` — Appendix E のレシピを実装可能な形式に展開
- `mystical-ci-stance-configuration-guide-v0.1.md` ★v0.4 新規 — スタンス選択の運用ガイド

### 長期 (Phase 4 以降)

- `mystical-ci-appendix-F-language-adapters-v0.1.md`
- `mystical-ci-appendix-G-dynamic-analysis-v0.1.md`
- `mystical-ci-appendix-H-accessibility-v0.1.md`
- `mystical-ci-appendix-I-style-refactoring-v0.1.md`
- `mystical-ci-appendix-J-organizational-patterns-v0.1.md` ★v0.4 新規 — 組織パターンと式の対応 (Team Topologies 系)

### Appendix E の継続的拡張

予定される追加対象:
- Test-Driven Development (Beck)
- Smalltalk Best Practice Patterns (Beck)
- Continuous Delivery (Humble, Farley)
- Site Reliability Engineering (Google)
- Accelerate (Forsgren et al.)
- Team Topologies (Skelton, Pais) — ★v0.4 重要 (スタンス選択との関連が深い)
- The Art of Computer Programming (Knuth)
- SICP (Abelson, Sussman)

### ユーザー向け文書 (実装完了後)

- `mystical-ci-user-guide.md`
- `mystical-ci-cli-reference.md`
- `mystical-ci-cookbook.md`
- `mystical-ci-design-tutorial.md`
- `mystical-ci-philosophy-reader.md` — Appendix E のユーザー向け簡易版
- `mystical-ci-stance-tutorial.md` ★v0.4 新規 — スタンス選択のチュートリアル

---

## 命名規則 (v0.1 から変更なし)

`mystical-ci-<カテゴリ>-<内容>-<バージョン>.md`

---

## このプロジェクトの起源について (v0.4 拡張)

本プロジェクトは、2026年5月19日付の GIGAZINE 記事「魔法陣のようなプログラミング言語『Mystical』」を契機とする対話から生まれた。対話の中で以下の発見が積み重ねられた:

1. 図形表現はコードレビューの注意誘導装置として機能しうる
2. 立体ビューは情報密度を高めるが、デフォルトでは平面が適切
3. 描画様式は単なる意匠ではなく、解析軸 (Call Graph vs Data Flow) の選択である
4. ファンタジー作品の魔法陣群は、群論的な対称性分類と独立に収斂進化している
5. モジュール単位の式と結合点により、コードベース全体が魔導都市的景観として可視化可能である
6. ツールは双方向に使える。「コード → 魔法陣」だけでなく「求める性質 → 魔法陣 → コード」も可能。異質式適用により新パラダイムの引き寄せができる。
7. ファンタジーの図像だけでなく、設計書 (POSD, Clean Architecture など) からも新しい式が派生する。Profundus 式は POSD から導出された、垂直方向の非対称性を扱う初めての式である。
8. 古典的設計書群は、本ツールの式体系に翻訳することで、再利用可能なレシピ集として機能する。
9. POSD と Clean Architecture の対比は、裁量 vs 規範というソフトウェア哲学史の二項対立の現代的現れである。
10. ★v0.4 追加: ツール自身が裁量 vs 規範のどちらを選ぶかを**ユーザーに委ねる**ことで、二項対立を**Strange Loop で超越**できる。本ツールは Advisor / Enforcer / Hybrid のスタンスを選択可能とすることで、あらゆる開発文化に適合する。
11. ★v0.4 追加: スタンス選択は技術的決定を超えて、**チームの文化的自己認識**を促す装置として機能する。これにより本ツールは Conway's Law を意識的に活用する設計となる。

これらの発見が、本文書群として体系化された。発見 6-11 は本ツールを単なる可視化ツールから**ソフトウェア設計と組織文化を媒介する記号体系**へと拡張する転換点となった。

プロジェクトは現時点で**設計フェーズ**にあり、実装は未着手である。

---

## 統計情報 (v0.4 時点)

- 文書数: 7 (中核 2 + 補遺 5)
- 総文字数: 約 140,000 字 (推定)
- 議論された式の数: 10 (ベルカ、うみねこ、Vesperia、ミッドチルダ3バリアント、Alchemy、黒執事、RWBY、Fate、夜天、Profundus)
- 分析された設計書: 13 (Appendix E §2)
- 提案されたレシピ数: 約 28 (既存 12 + Clean Arch + POSD + Fusion + Appendix E 由来 13)
- 議論されたハイブリッド合成数: 11
- 提案されたスタンス: 3 (Advisor / Enforcer / Hybrid)

これらの数値は今後の文書追加とともに更新される。

---

## ★v0.4 新規: 本ツールの三層構造

v0.4 までの議論を統合すると、本ツールは三層の構造を持つ:

### 表層: 可視化と設計支援 (Surface Layer)

- Analyzer Mode で既存コードを魔法陣として可視化
- Designer Mode で求める性質から骨格コードを生成
- 立体ビュー、レイヤー切替、対話的ズーム

### 中層: 知識資産との接続 (Knowledge Layer)

- 設計書群 (Appendix E) のレシピ化
- 既存設計知識資産 (POSD, Clean Architecture, DDD 等) との橋渡し
- 異質式適用による新パラダイムの引き寄せ
- 式の合成による未発見パターンの探索

### 深層: 哲学的・組織的位置づけ (Meta Layer)

- 裁量 vs 規範のスタンス選択 (Appendix C v0.2 §6, Appendix D v0.2 §11)
- 組織文化との接続 (Conway's Law)
- Strange Loop による二項対立の超越
- ツール自身の存在論的位置づけ

各層は独立して価値を持つが、三層が統合されることで本ツールは**現代ソフトウェア工学における新しい記号体系**として機能する。

---

## バージョン履歴

v0.1 — 初版。Spec、Phase 2+ Notes、Appendix A/B/C の 5 文書が出揃った段階での INDEX。

v0.2 — Appendix D (設計支援モードと式駆動設計) の追加に伴う更新。文書数 6 に増加。

v0.3 — Appendix A v0.2 (Profundus 式追加) と Appendix E (設計哲学カタログ) の新規追加。文書数 7 に増加。

v0.4 — Appendix C v0.2 (裁量 vs 規範の二項対立、ソフトウェア哲学史) と Appendix D v0.2 (Clean Arch/POSD/Fusion レシピ、Advisor/Enforcer/Hybrid スタンス) の更新を反映。読み方ガイドにケース 9 (スタンス検討チーム) とケース 10 (哲学・組織論研究者) を追加。新セクション「本ツールの三層構造」を追加。Appendix シリーズ A-E が一通り出揃ったマイルストーンとして位置づけ。
