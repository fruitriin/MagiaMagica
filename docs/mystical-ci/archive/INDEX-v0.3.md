# Mystical CI プロジェクト文書 INDEX v0.3

## プロジェクト概要

**Mystical CI** は、ソースコードを魔法陣状の図形に変換することで、コードレビューとリファクタリングにおける注意誘導を支援する**双方向**のコード可視化・設計支援ツールである。オリジナルプログラミング言語 Mystical (Denis M.) の図形思想を継承しつつ、群論的な対称性分類とファンタジー図像学、および古典的な設計書の哲学を組み合わせた独自の視覚言語を構築する。

ツールは二つのモードを持つ:

- **Analyzer Mode**: コードを解析し、対応する魔法陣を生成する (現状診断)
- **Designer Mode**: 求める性質から魔法陣を選び、骨格コードを生成する (設計支援)

両モードは同じ視覚言語を共有し、双方向に行き来できる。さらに、ツール自身のスタンス選択 (Advisor / Enforcer / Hybrid) が可能で、裁量重視と規範重視の両方の開発文化に適合する。

---

## 文書構造 (v0.3 更新)

本プロジェクトに関する文書は、以下のファイル群で構成される。

### 中核文書

**`mystical-ci-spec-v0.1.md`** — Mystical CI 仕様書 v0.1

Phase 1 のスコープを厳密に定義した正式仕様書。

**実装時の主参照文書**。

**`mystical-ci-phase2plus-notes-v0.1.md`** — Phase 2 以降の議論ノート v0.1

Phase 2 から Phase 6 までの構想を集約。

**長期ロードマップの主参照文書**。

### 補遺文書

**`mystical-ci-appendix-A-rendering-styles-v0.2.md`** — Appendix A: 描画様式カタログ v0.2 ★更新

ツールが扱う「式 (RenderStyle)」の体系をカタログ化。**v0.2 で深層式 (Profundus) を新規追加**し、水平対称性と垂直対称性の区別を導入。

**描画様式に関する設計判断の主参照文書**。

**`mystical-ci-appendix-B-codebase-landscape-v0.1.md`** — Appendix B: コードベース景観とモジュール結合 v0.1

関数単位の式をモジュール単位・プロジェクト単位にスケールアップさせる議論。

**スケール上位の設計判断の主参照文書**。

**`mystical-ci-appendix-C-theoretical-foundations-v0.1.md`** — Appendix C: 理論的基礎 v0.1

群論、視覚認知科学、計算理論、ソフトウェア工学、哲学からの正当化を集約。

**理論的根拠の参照文書**。Phase 2-3 で v0.2 への更新が予定されている (裁量 vs 規範の哲学史的位置づけを追加)。

**`mystical-ci-appendix-D-design-mode-v0.1.md`** — Appendix D: 設計支援モードと式駆動設計 v0.1

ツールの双方向性を確立する文書。

**設計支援に関する設計判断の主参照文書**。Phase 2-3 で v0.2 への更新が予定されている (Advisor/Enforcer/Hybrid モード、Clean Architecture / POSD レシピの追加)。

**`mystical-ci-appendix-E-design-philosophies-v0.1.md`** — Appendix E: 設計哲学カタログ v0.1 ★新規

歴史的に重要な設計書 (POSD, Clean Architecture, DDD, Functional DDD, GoF, PoEAA, Refactoring, Pragmatic Programmer, Code Complete, Working with Legacy Code, Functional Programming, Microservices, Mythical Man-Month など) を本ツールの式体系に翻訳して分類・分析する文書。書籍間の補完・対立関係をマップ化し、本ツールのレシピソースとして統合する。

**既存設計知識資産との接続点となる重要文書**。

---

## 文書間の依存関係 (v0.3 更新)

```
[Spec v0.1] ── 実装の基盤
    │
    ├──> [Phase 2+ Notes] ── 将来構想の集約
    │         │
    │         ├──> [Appendix A v0.2] ★ ── 描画様式の詳細
    │         │         │  (Profundus 式の追加で大幅拡張)
    │         │         │
    │         │         ├──> [Appendix D] ── 設計支援への展開
    │         │         │         │
    │         │         │         └──> [Appendix E] ★新規 ── 設計哲学カタログ
    │         │         │                  ↑
    │         │         │                  │ レシピソース提供
    │         │         │                  │
    │         │         │                  └──> [Appendix D の §4 レシピ集] を充実化
    │         │         │
    │         │         ├──> [Appendix B] ── 景観論の詳細
    │         │         │         │
    │         │         │         └──> [Appendix E の §2.2-2.3] と接続
    │         │         │              (Clean Architecture, DDD は景観論と直結)
    │         │         │
    │         │         └──> [Appendix C] ── 理論的基礎
    │         │                    │
    │         │                    └──> [Appendix E の §3, §6] と接続
    │         │                         (書籍間の関係マップ、哲学史的位置づけ)
    │
    └──> 将来の Spec v0.2 (Phase 2 着手時)
              ├── Phase 2+ Notes から昇格
              ├── Appendix A の確定部分を取り込み
              ├── Appendix B の確定部分を取り込み
              ├── Appendix D の Designer Mode 仕様を取り込み
              └── Appendix E のレシピ統合機構を取り込み
```

---

## 読み方ガイド (v0.3 更新)

### ケース 1: 実装に着手する開発者

1. `mystical-ci-spec-v0.1.md` を熟読
2. 必要に応じて `mystical-ci-phase2plus-notes-v0.1.md` で将来計画を確認
3. Appendix A v0.2 の §1 (二軸分類モデル) で式の体系を把握
4. Appendix C, D, E は実装には直接必要ないが、設計の意図を理解したい場合に参照

### ケース 2: プロジェクトの全体像を理解したい新規参画者

1. 本 INDEX 文書
2. `mystical-ci-spec-v0.1.md` の §1〜§2
3. `mystical-ci-phase2plus-notes-v0.1.md` の §1 と §11
4. Appendix A v0.2 の §1 と §2 のいくつかの式 (特に §2.10 Profundus 式)
5. Appendix B の §1〜§3
6. Appendix D の §1
7. Appendix E の §1 と §2.1, §2.2 (POSD と Clean Architecture)
8. Appendix C は興味があれば

### ケース 3: 設計の哲学的根拠を知りたい技術者

1. `mystical-ci-spec-v0.1.md` の §2
2. `mystical-ci-phase2plus-notes-v0.1.md` の §11
3. Appendix C 全文
4. Appendix D の §9 と §10
5. Appendix E §3 (書籍間の関係マップ) と §6 (時代的変遷)

### ケース 4: 描画様式 (式) の追加・変更を提案する人

1. Appendix A v0.2 全文
2. Appendix C §1〜§2
3. Appendix D §2 (異質式適用)
4. Appendix E の §2 (各書籍が対応する式を確認、新式の発見元として書籍を活用)

### ケース 5: モジュール結合や景観可視化の改善を提案する人

1. Appendix B 全文
2. Appendix C §5
3. `mystical-ci-phase2plus-notes-v0.1.md` の §4
4. Appendix D §5 (式の合成)
5. Appendix E §2.2-2.3 (Clean Architecture, DDD は景観論と直結)

### ケース 6: 設計支援機能を活用したい開発者

1. Appendix D 全文
2. Appendix A v0.2 (式の体系を理解)
3. Appendix B (式の合成と景観の理解)
4. Appendix E の §4 (推奨される読書順序、自分のレベルに合う設計書を選ぶ)

### ケース 7: 新しいレシピを提案したい人

1. Appendix D の §4 (レシピ集) と §3 (式駆動設計)
2. Appendix E の §5 (ツールへの統合)
3. Appendix A v0.2 (式の選択肢を理解)
4. 提案レシピの基盤となる設計書を Appendix E で確認

### ケース 8: 既存の設計知識を活用したい開発者 ★新規

1. **Appendix E 全文** が中心
2. 自分の状況 (ズームレベル × 文化スタンス) に応じた書籍を §4 から選ぶ
3. 対応する式を Appendix A v0.2 で確認
4. ツールの Designer Mode で実装に活用 (Phase 3+)

### ケース 9: ユーザー (将来の)

実装後にユーザー向けドキュメント (チュートリアル、CLI リファレンス、Cookbook) が別途整備される予定。

---

## バージョン管理方針

(v0.1 から変更なし)

各文書は独立してバージョン管理。INDEX 文書自体もバージョン管理対象。

### v0.3 における変更

- Appendix A: v0.1 → v0.2 (Profundus 式追加)
- Appendix E v0.1 を新規追加
- 文書数: 6 → 7

(Appendix C, D は次の更新サイクルで v0.2 へ予定)

---

## 今後追加が想定される文書 (v0.3 更新)

### 計画されている更新

- `mystical-ci-appendix-C-theoretical-foundations-v0.2.md` (予定) — 裁量 vs 規範の二項対立、ソフトウェア哲学史的位置づけを追加
- `mystical-ci-appendix-D-design-mode-v0.2.md` (予定) — Clean Architecture / POSD / Fusion レシピを追加、Advisor/Enforcer/Hybrid モードのスタンスセクションを追加

### 短期 (Phase 1 実装期)

- `mystical-ci-implementation-guide-v0.1.md`
- `mystical-ci-ir-schema-v0.1.md`

### 中期 (Phase 2-3 実装期)

- `mystical-ci-filter-dsl-spec-v0.1.md`
- `mystical-ci-layout-algorithm-v0.1.md`
- `mystical-ci-glyph-symbol-catalog-v0.1.md`
- `mystical-ci-designer-mode-spec-v0.1.md`
- `mystical-ci-recipe-collection-v0.1.md` — Appendix E のレシピを実装可能な形式に展開

### 長期 (Phase 4 以降)

- `mystical-ci-appendix-F-language-adapters-v0.1.md`
- `mystical-ci-appendix-G-dynamic-analysis-v0.1.md`
- `mystical-ci-appendix-H-accessibility-v0.1.md`
- `mystical-ci-appendix-I-style-refactoring-v0.1.md` — 式変換 (Style Refactoring) の詳細

### Appendix E の継続的拡張

Appendix E は最も継続的な拡張が想定される文書である。新しい設計書の発見、新しいパラダイムの登場、コミュニティからの貢献により、定期的に v0.2, v0.3 と更新される。

予定される追加対象:
- Test-Driven Development (Beck)
- Smalltalk Best Practice Patterns (Beck)
- Continuous Delivery (Humble, Farley)
- Site Reliability Engineering (Google)
- Accelerate (Forsgren et al.)
- Team Topologies (Skelton, Pais)
- The Art of Computer Programming (Knuth)
- SICP (Abelson, Sussman)

### ユーザー向け文書 (実装完了後)

- `mystical-ci-user-guide.md`
- `mystical-ci-cli-reference.md`
- `mystical-ci-cookbook.md`
- `mystical-ci-design-tutorial.md`
- `mystical-ci-philosophy-reader.md` ★新規 — Appendix E のユーザー向け簡易版

---

## 命名規則 (v0.1 から変更なし)

`mystical-ci-<カテゴリ>-<内容>-<バージョン>.md`

---

## このプロジェクトの起源について (v0.3 拡張)

本プロジェクトは、2026年5月19日付の GIGAZINE 記事「魔法陣のようなプログラミング言語『Mystical』」を契機とする対話から生まれた。対話の中で以下の発見が積み重ねられた:

1. 図形表現はコードレビューの注意誘導装置として機能しうる
2. 立体ビューは情報密度を高めるが、デフォルトでは平面が適切
3. 描画様式は単なる意匠ではなく、解析軸 (Call Graph vs Data Flow) の選択である
4. ファンタジー作品の魔法陣群は、群論的な対称性分類と独立に収斂進化している
5. モジュール単位の式と結合点により、コードベース全体が魔導都市的景観として可視化可能である
6. ツールは双方向に使える。「コード → 魔法陣」だけでなく「求める性質 → 魔法陣 → コード」も可能。異質式適用により新パラダイムの引き寄せができる。
7. ★v0.3 追加: ファンタジーの図像だけでなく、**設計書 (POSD, Clean Architecture など) からも新しい式が派生する**。Profundus 式は POSD から導出された、垂直方向の非対称性を扱う初めての式である。
8. ★v0.3 追加: 古典的設計書群は、本ツールの式体系に翻訳することで、再利用可能なレシピ集として機能する。**Appendix E は既存設計知識資産と本ツールを接続する架け橋**として位置づけられる。
9. ★v0.3 追加: **POSD と Clean Architecture の対比は、裁量 vs 規範というソフトウェア哲学史の二項対立の現代的現れ**である。これはツール自身のスタンス選択 (Advisor / Enforcer) を促す重要な視点となる。

これらの発見が、本文書群として体系化された。発見 6-9 は本ツールを単なる可視化ツールから**ソフトウェア設計の記号体系**へと拡張する転換点となった。

プロジェクトは現時点で**設計フェーズ**にあり、実装は未着手である。

---

## 統計情報 (v0.3 時点)

- 文書数: 7 (中核 2 + 補遺 5)
- 総文字数: 約 120,000 字 (推定)
- 議論された式の数: 10 (ベルカ、うみねこ、Vesperia、ミッドチルダ3バリアント、Alchemy、黒執事、RWBY、Fate、夜天、Profundus)
- 分析された設計書: 13 (POSD, Clean Architecture, DDD, Functional DDD, GoF, PoEAA, Refactoring, Pragmatic Programmer, Code Complete, Working with Legacy Code, Functional Programming 各種, Microservices, Mythical Man-Month)
- 提案されたレシピ数: 12 + Appendix E の各書籍 ≈ 25
- 議論されたハイブリッド合成数: 8

これらの数値は今後の文書追加とともに更新される。

---

## バージョン履歴

v0.1 — 初版。Spec、Phase 2+ Notes、Appendix A/B/C の 5 文書が出揃った段階での INDEX。

v0.2 — Appendix D (設計支援モードと式駆動設計) の追加に伴う更新。文書数 6 に増加。ツールの双方向性を反映。

v0.3 — Appendix A の v0.2 への更新 (Profundus 式追加) と、Appendix E (設計哲学カタログ) の新規追加。文書数 7 に増加。設計書からの式の派生という新しい知見を反映。読み方ガイドにケース 8 (既存設計知識の活用) を追加。Appendix C, D の v0.2 への更新が予定されていることを明記。
