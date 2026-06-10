# Mystical CI プロジェクト文書 INDEX v0.5

## プロジェクト概要

**Mystical CI** は、ソースコードを魔法陣状の図形に変換することで、コードレビューとリファクタリングにおける注意誘導を支援する、**三重の目的**を持つコード可視化・設計支援ツールである。

ツールの三重目的:

- **Analyzer Mode**: コードを解析し、対応する魔法陣を生成する (現状診断)
- **Designer Mode**: 求める性質から魔法陣を選び、骨格コードを生成する (設計支援)
- **Stance Selection**: ツール自身のスタンス (Advisor / Enforcer / Hybrid) を選択することで、あらゆる開発文化に適合する (文化適合)

そしてツールの**射程**は厳密に画定されている: 本ツールは**形式 (Form) の領域**のみを扱い、**意味 (Meaning) の領域**には立ち入らない。命名の良し悪し、ドメインロジックの正しさ、セキュリティ、性能などは射程外であり、専用ツール・AI・人間との補完関係で対応する。

この射程の限定こそが、ツールの徹底性、信頼性、高速性、普遍性、教育的価値、哲学的純粋さを担保する。

---

## 文書構造 (v0.5 更新)

### 中核文書

**`mystical-ci-spec-v0.1.md`** — Mystical CI 仕様書 v0.1

Phase 1 のスコープを厳密に定義した正式仕様書。実装時の主参照文書。

**`mystical-ci-phase2plus-notes-v0.1.md`** — Phase 2 以降の議論ノート v0.1

Phase 2 から Phase 6 までの構想を集約。長期ロードマップの主参照文書。

### 補遺文書 (現行バージョン)

**`mystical-ci-appendix-A-rendering-styles-v0.2.md`** — Appendix A: 描画様式カタログ v0.2

10式 (深層式 Profundus 含む) と二軸分類モデル。

**`mystical-ci-appendix-B-codebase-landscape-v0.1.md`** — Appendix B: コードベース景観とモジュール結合 v0.1

結合点 (Junction Glyph)、結合トポロジー、フラクタル景観、品質指標。

**`mystical-ci-appendix-C-theoretical-foundations-v0.2.md`** — Appendix C: 理論的基礎 v0.2

裁量 vs 規範の二項対立、ソフトウェア哲学史的位置づけ。

**`mystical-ci-appendix-D-design-mode-v0.2.md`** — Appendix D: 設計支援モードと式駆動設計 v0.2

Clean Architecture / POSD / Fusion レシピ、Advisor / Enforcer / Hybrid スタンス。

**`mystical-ci-appendix-E-design-philosophies-v0.2.md`** — Appendix E: 設計哲学カタログ v0.2 ★更新

**v0.2 でメタ分類軸を新規導入**。設計書を5タイプ (パターン集型 / 単一哲学型 / 戦略+戦術型 / 態度・原則型 / 規範書型) に分類。GoF を「カタログ式 (Catalog Style)」として位置づけ、新しい構造的概念を導入。

**`mystical-ci-appendix-F-scope-and-limits-v0.1.md`** — Appendix F: ツールの射程と限界 v0.1 ★新規

ツールが**何を扱えるか**と**何を扱えないか**を厳密に区別。形式と意味の区別、見えるもの/見えないものの体系的カタログ、専用ツール・AI・人間との補完関係、ヴィトゲンシュタインとの対比による哲学的位置づけ。**他の Appendix の射程を限定する枠組み文書**として位置づけられる。

---

## 文書間の依存関係 (v0.5 更新)

```
[Spec v0.1] ── 実装の基盤
    │
    ├──> [Phase 2+ Notes] ── 将来構想の集約
    │         │
    │         ├──> [Appendix A v0.2] ── 描画様式の詳細
    │         │         │
    │         │         ├──> [Appendix D v0.2] ── 設計支援
    │         │         │         │
    │         │         │         └──> [Appendix E v0.2] ★ ── 設計哲学カタログ
    │         │         │                  │      (メタ分類軸の追加)
    │         │         │                  │
    │         │         │                  └─→ Designer Mode の振る舞いを
    │         │         │                       メタタイプで変える
    │         │         │
    │         │         ├──> [Appendix B] ── 景観論
    │         │         │
    │         │         └──> [Appendix C v0.2] ── 理論的基礎
    │         │                    
    │         └─→ [Appendix F v0.1] ★新規 ── 射程と限界
    │                    │
    │                    ├─→ 他の Appendix の射程を限定
    │                    ├─→ Spec §1.1 の「注意のスポットライト」を厳密化
    │                    ├─→ 形式と意味の区別を導入
    │                    └─→ 専用ツール・AI・人間との補完関係を明示
    │
    └──> 将来の Spec v0.2 (Phase 2 着手時)
              ├── Phase 2+ Notes から昇格
              ├── Appendix A v0.2 の確定部分を取り込み
              ├── Appendix B の確定部分を取り込み
              ├── Appendix D v0.2 の Designer Mode + スタンス仕様を取り込み
              ├── Appendix E v0.2 のレシピ統合機構を取り込み
              └── Appendix F v0.1 の射程定義を取り込み (重要)
```

---

## 読み方ガイド (v0.5 更新)

### ケース 1: 実装に着手する開発者

1. `mystical-ci-spec-v0.1.md` を熟読
2. `mystical-ci-phase2plus-notes-v0.1.md` で将来計画を確認
3. Appendix A v0.2 の §1 で式の体系を把握
4. **Appendix F v0.1 を読み、ツールの射程を理解** ★v0.5 新規必読
5. その他の Appendix は実装時の参照

### ケース 2: プロジェクトの全体像を理解したい新規参画者

1. 本 INDEX 文書
2. `mystical-ci-spec-v0.1.md` の §1〜§2
3. Appendix A v0.2 の §1
4. **Appendix F v0.1 §1, §2, §3** (ツールの射程と限界) ★v0.5 必読
5. Appendix B の §1〜§3
6. Appendix D v0.2 の §1 と §11
7. Appendix E v0.2 の §1.3 (メタ分類)
8. Appendix C v0.2 は哲学的背景に興味があれば

### ケース 3: 設計の哲学的根拠を知りたい技術者

1. `mystical-ci-spec-v0.1.md` の §2
2. Appendix C v0.2 全文 (特に §6)
3. Appendix D v0.2 の §9, §11
4. Appendix E v0.2 §3, §6
5. **Appendix F v0.1 §10 (ヴィトゲンシュタインとの対比)** ★v0.5 新規

### ケース 4: 描画様式 (式) の追加・変更を提案する人

1. Appendix A v0.2 全文
2. Appendix C v0.2 §1〜§2
3. Appendix D v0.2 §2 (異質式適用)
4. Appendix E v0.2 §1.4 (カタログ式の発見、新式提案のヒント)
5. **Appendix F v0.1** (新式提案が形式の領域に留まるか確認)

### ケース 5: モジュール結合や景観可視化の改善を提案する人

1. Appendix B 全文
2. Appendix C v0.2 §5
3. Appendix D v0.2 §5
4. Appendix E v0.2 §2.2-2.3
5. **Appendix F v0.1 §2.2, §2.3, §2.4** (結合点・参照の散らばり・循環参照の可視化が射程内であることを確認)

### ケース 6: 設計支援機能を活用したい開発者

1. Appendix D v0.2 全文 (特に §4 レシピ集、§11 スタンス選択)
2. Appendix A v0.2
3. Appendix B
4. Appendix E v0.2 (特に §4 推奨読書順序、§1.3 メタ分類)
5. Appendix F v0.1 §5 (見えないものを補う方法)

### ケース 7: 新しいレシピを提案したい人

1. Appendix D v0.2 §4 と §3
2. Appendix E v0.2 §5 と §1.3 (メタタイプ別の振る舞い)
3. Appendix A v0.2
4. **Appendix F v0.1 §2-§3** (レシピが形式の領域に対応しているか確認)

### ケース 8: 既存の設計知識を活用したい開発者

1. **Appendix E v0.2 全文** (特に §1.3 メタ分類、§2.5 GoF のカタログ式)
2. §4 で自分のレベルに合う設計書を選ぶ
3. 対応する式を Appendix A v0.2 で確認
4. Designer Mode で活用 (Phase 3+)

### ケース 9: ツールのスタンスを検討するチーム

1. Appendix D v0.2 §11
2. Appendix C v0.2 §6
3. プロジェクト規約として `mystical.toml` を作成

### ケース 10: 哲学・組織論との接続に興味がある研究者

1. Appendix C v0.2 §6
2. Appendix C v0.2 §10 (関連分野リスト)
3. Appendix D v0.2 §11.10 (Strange Loop)
4. Appendix E v0.2 §3, §6
5. **Appendix F v0.1 §10 (ヴィトゲンシュタイン)** ★v0.5 新規

### ケース 11: ツールに過剰な期待を持っている人 ★v0.5 新規

1. **Appendix F v0.1 §3 (見えないものリスト)** を最初に読む
2. Appendix F §5 (見えないものを補う方法) で代替手段を理解
3. その上で、ツールの強み (Appendix F §6) を活用する方針を立てる

このケースが追加された理由: ツールが万能であるかのような誤解を防ぐため。射程の明確化はツールの正しい使い方を促す重要な情報。

### ケース 12: ツールと他のツール (AI, 静的解析, APM) を併用したい開発者 ★v0.5 新規

1. Appendix F v0.1 §5 (見えないものを補う方法) で併用候補を確認
2. 各専用ツールが本ツールとどう補完するかを確認
3. Phase 6 の AI 注釈チャネル (Appendix F §5.2) を将来計画として把握

### ケース 13: ユーザー (将来の)

実装後にユーザー向けドキュメントが整備される予定。

---

## バージョン管理方針 (v0.1 から変更なし)

各文書は独立してバージョン管理。INDEX 文書自体もバージョン管理対象。

### v0.5 における変更

- Appendix E: v0.1 → v0.2 (メタ分類軸の新規導入、GoF をカタログ式として位置づけ)
- Appendix F v0.1 を新規追加 (ツールの射程と限界)
- 文書数: 7 → 8

---

## 今後追加が想定される文書 (v0.5 更新)

### 短期 (Phase 1 実装期)

- `mystical-ci-implementation-guide-v0.1.md`
- `mystical-ci-ir-schema-v0.1.md`

### 中期 (Phase 2-3 実装期)

- `mystical-ci-filter-dsl-spec-v0.1.md`
- `mystical-ci-layout-algorithm-v0.1.md`
- `mystical-ci-glyph-symbol-catalog-v0.1.md`
- `mystical-ci-designer-mode-spec-v0.1.md`
- `mystical-ci-recipe-collection-v0.1.md`
- `mystical-ci-stance-configuration-guide-v0.1.md`

### 長期 (Phase 4 以降)

- `mystical-ci-appendix-G-language-adapters-v0.1.md`
- `mystical-ci-appendix-H-dynamic-analysis-v0.1.md`
- `mystical-ci-appendix-I-accessibility-v0.1.md`
- `mystical-ci-appendix-J-style-refactoring-v0.1.md`
- `mystical-ci-appendix-K-organizational-patterns-v0.1.md`
- `mystical-ci-appendix-L-catalog-style-rendering-v0.1.md` ★v0.5 新規 — カタログ式の表現様式専用文書
- `mystical-ci-appendix-M-ai-integration-v0.1.md` ★v0.5 新規 — AI 注釈チャネルの詳細

### Appendix E の継続的拡張 (v0.3, v0.4...)

予定される追加対象:
- Test-Driven Development (Beck)
- Smalltalk Best Practice Patterns (Beck)
- Continuous Delivery (Humble, Farley) — 戦略+戦術型
- Site Reliability Engineering (Google) — 戦略+戦術型
- Accelerate (Forsgren et al.) — 規範書型
- Team Topologies (Skelton, Pais) — 戦略+戦術型 (スタンス選択との関連が深い)
- The Art of Computer Programming (Knuth) — 規範書型
- SICP (Abelson, Sussman) — 単一哲学型

### Appendix F の継続的拡張 (v0.2, v0.3...)

予定される追加:
- LLM との詳細な協業プロトコル
- 形式から意味への示唆の自動化
- 文化別の形式規則のカタログ
- 「示すことができるが語ることはできない」の事例集

### ユーザー向け文書 (実装完了後)

- `mystical-ci-user-guide.md`
- `mystical-ci-cli-reference.md`
- `mystical-ci-cookbook.md`
- `mystical-ci-design-tutorial.md`
- `mystical-ci-philosophy-reader.md`
- `mystical-ci-stance-tutorial.md`
- `mystical-ci-scope-faq.md` ★v0.5 新規 — 「これはツールでできますか?」によくある質問

---

## 命名規則 (v0.1 から変更なし)

`mystical-ci-<カテゴリ>-<内容>-<バージョン>.md`

---

## このプロジェクトの起源について (v0.5 拡張)

本プロジェクトは、2026年5月19日付の GIGAZINE 記事「魔法陣のようなプログラミング言語『Mystical』」を契機とする対話から生まれた。対話の中で以下の発見が積み重ねられた:

1. 図形表現はコードレビューの注意誘導装置として機能しうる
2. 立体ビューは情報密度を高めるが、デフォルトでは平面が適切
3. 描画様式は単なる意匠ではなく、解析軸 (Call Graph vs Data Flow) の選択である
4. ファンタジー作品の魔法陣群は、群論的な対称性分類と独立に収斂進化している
5. モジュール単位の式と結合点により、コードベース全体が魔導都市的景観として可視化可能である
6. ツールは双方向に使える (Analyzer ⇄ Designer)。異質式適用により新パラダイムの引き寄せができる
7. ファンタジーの図像だけでなく、設計書 (POSD など) からも新しい式が派生する。Profundus 式は POSD から導出
8. 古典的設計書群は、本ツールの式体系に翻訳することで、再利用可能なレシピ集として機能する
9. POSD と Clean Architecture の対比は、裁量 vs 規範というソフトウェア哲学史の二項対立の現代的現れ
10. ツール自身が裁量 vs 規範のどちらを選ぶかをユーザーに委ねることで、二項対立を Strange Loop で超越
11. スタンス選択は技術的決定を超えて、チームの文化的自己認識を促す装置として機能する
12. ★v0.5 追加: **GoF は他の設計書と根本的に異なる「カタログ型」**である。23 個の独立した小魔法陣の集合体であり、統一的な大きな構造を持たない。これは設計書のメタ分類 (パターン集型 / 単一哲学型 / 戦略+戦術型 / 態度・原則型 / 規範書型) の発見につながった。
13. ★v0.5 追加: **ツールは形式 (Form) の領域のみを扱い、意味 (Meaning) の領域には立ち入らない**。命名の良し悪し、ドメインロジック、セキュリティなどは射程外。この射程の限定こそが、ツールの徹底性・信頼性・高速性・普遍性・教育的価値・哲学的純粋さを担保する。
14. ★v0.5 追加: 本ツールの自己定義はヴィトゲンシュタインの **「語りえぬものについては沈黙しなければならない」** と構造的に同型である。形式の領域に徹することで、ソフトウェア工学版の Tractatus 的な記号体系として機能する可能性がある。

これらの発見が、本文書群として体系化された。発見 12-14 は本ツールの自己認識を完成させる重要な転換点で、Appendix E v0.2 と Appendix F v0.1 として明文化された。

プロジェクトは現時点で**設計フェーズ**にあり、実装は未着手である。

---

## 統計情報 (v0.5 時点)

- 文書数: 8 (中核 2 + 補遺 6)
- 総文字数: 約 170,000 字 (推定)
- 議論された式の数: 11 (ベルカ、うみねこ、Vesperia、ミッドチルダ3バリアント、Alchemy、黒執事、RWBY、Fate、夜天、Profundus、+ カタログ式)
- 分析された設計書: 13
- メタタイプの分類: 5 (パターン集型、単一哲学型、戦略+戦術型、態度・原則型、規範書型)
- 提案されたレシピ数: 約 28
- 議論されたハイブリッド合成数: 11
- 提案されたスタンス: 3 (Advisor / Enforcer / Hybrid)
- 明示化された「見えないもの」の項目: 10 (Appendix F §3)

---

## 本ツールの三層構造 (v0.4 から拡張)

v0.4 で導入した三層構造に、v0.5 の発見を加える。

### 表層: 可視化と設計支援 (Surface Layer)

- Analyzer / Designer Mode
- 立体ビュー、レイヤー切替、対話的ズーム
- ★v0.5: カタログ式の表現 (魔導書のページ的ビュー)

### 中層: 知識資産との接続 (Knowledge Layer)

- 設計書群 (Appendix E) のレシピ化
- ★v0.5: メタタイプ別のレシピ振る舞い
- 異質式適用、式の合成

### 深層: 哲学的・組織的位置づけ (Meta Layer)

- 裁量 vs 規範のスタンス選択
- 組織文化との接続 (Conway's Law)
- Strange Loop による二項対立の超越
- ★v0.5: 形式と意味の区別 (ヴィトゲンシュタイン的)

### ★v0.5 追加: 限界層 (Limit Layer)

これまでの三層に加え、**ツールの限界を明示する第四層**を Appendix F が提供する。

- 形式の領域に徹する
- 意味の領域への侵入を拒む
- 専用ツール・AI・人間との補完関係
- ヴィトゲンシュタイン的な「示すが語らない」哲学

四層構造により、本ツールは「機能の拡張」と「限界の明示」の両方を完備した、自己完結的な記号体系として位置づけられる。

---

## バージョン履歴

v0.1 — 初版。Spec、Phase 2+ Notes、Appendix A/B/C の 5 文書。

v0.2 — Appendix D 追加。文書数 6。

v0.3 — Appendix A v0.2 (Profundus 追加) と Appendix E v0.1 (設計哲学カタログ) 追加。文書数 7。

v0.4 — Appendix C v0.2 (哲学史的位置づけ) と Appendix D v0.2 (スタンス選択) の更新。三層構造を導入。

v0.5 — Appendix E v0.2 (メタ分類軸の導入、GoF をカタログ式として位置づけ) と Appendix F v0.1 (ツールの射程と限界) を追加。文書数 8。「形式と意味の区別」「カタログ式」という2つの新概念により、本ツールの自己認識が完成段階に到達。四層構造を導入。読み方ガイドにケース 11 (過剰期待の防止) とケース 12 (他ツールとの併用) を追加。
