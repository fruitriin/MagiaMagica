# Mystical CI プロジェクト文書 INDEX v0.2

## プロジェクト概要

**Mystical CI** は、ソースコードを魔法陣状の図形に変換することで、コードレビューとリファクタリングにおける注意誘導を支援する**双方向**のコード可視化・設計支援ツールである。オリジナルプログラミング言語 Mystical (Denis M.) の図形思想を継承しつつ、群論的な対称性分類とファンタジー図像学を組み合わせた独自の視覚言語を構築する。

ツールは二つのモードを持つ:

- **Analyzer Mode**: コードを解析し、対応する魔法陣を生成する (現状診断)
- **Designer Mode**: 求める性質から魔法陣を選び、骨格コードを生成する (設計支援)

両モードは同じ視覚言語を共有し、双方向に行き来できる。ツールの自己定義は「品質保証ツール」ではなく**注意のスポットライト**かつ**設計の羅針盤**である。判定の権限は常に人間に残す。

---

## 文書構造 (v0.2 更新)

本プロジェクトに関する文書は、以下のファイル群で構成される。

### 中核文書

**`mystical-ci-spec-v0.1.md`** — Mystical CI 仕様書 v0.1

Phase 1 のスコープを厳密に定義した正式仕様書。単一 Rust 関数を SVG 1枚の平面魔法陣として出力する最小実装の仕様を記述する。IR スキーマ、設計原則、レンダリング要件を確定している。

**実装時の主参照文書**。

**`mystical-ci-phase2plus-notes-v0.1.md`** — Phase 2 以降の議論ノート v0.1

Phase 2 から Phase 6 までの構想を集約した議論文書。dev-server 化、レイヤーシステム、フィルター DSL、CI 統合、多言語アダプタ、動的解析、立体ビューなどを論じる。

**長期ロードマップの主参照文書**。

### 補遺文書

**`mystical-ci-appendix-A-rendering-styles-v0.1.md`** — Appendix A: 描画様式カタログ v0.1

ツールが扱う「式 (RenderStyle)」の体系をカタログ化した文書。二軸分類モデル (対称性のオーダー × 表現様式)、9つの式の詳細、自動推奨ヒューリスティクス、IR への統合方法を記述する。

**描画様式に関する設計判断の主参照文書**。

**`mystical-ci-appendix-B-codebase-landscape-v0.1.md`** — Appendix B: コードベース景観とモジュール結合 v0.1

関数単位の式をモジュール単位・プロジェクト単位にスケールアップさせる議論。結合点 (Junction Glyph)、結合トポロジー、フラクタルなズームレベル、景観スコアなどの新概念を提案する。

**スケール上位の設計判断の主参照文書**。

**`mystical-ci-appendix-C-theoretical-foundations-v0.1.md`** — Appendix C: 理論的基礎 v0.1

これまでの議論で前提とされてきた、あるいは暗黙裡に依拠してきた理論的・哲学的な基礎を明示化する文書。群論、視覚認知科学、計算理論、ソフトウェア工学、哲学からの正当化を集約する。

**理論的根拠の参照文書**。

**`mystical-ci-appendix-D-design-mode-v0.1.md`** — Appendix D: 設計支援モードと式駆動設計 v0.1 ★新規

ツールの双方向性を確立する文書。Designer Mode の概念、異質式適用 (Cross-Paradigm Application)、式駆動設計 (Style-Driven Design)、レシピ集、式の合成 (Style Composition)、式変換 (Style Refactoring) を体系化する。本ツールを可視化ツールから**設計記号体系**へと拡張する重要な転換点。

**設計支援に関する設計判断の主参照文書**。

---

## 文書間の依存関係 (v0.2 更新)

```
[Spec v0.1] ── 実装の基盤
    │
    ├──> [Phase 2+ Notes] ── 将来構想の集約
    │         │
    │         ├──> [Appendix A] ── 描画様式の詳細
    │         │         │
    │         │         └──> [Appendix D] ── 設計支援への展開 ★新規
    │         │                    ↑
    │         │                    │ (Analyzer ⇄ Designer の双方向性)
    │         │                    │
    │         ├──> [Appendix B] ── 景観論の詳細
    │         │         │
    │         │         └──> [Appendix D の §5 式の合成] と接続
    │         │
    │         └──> [Appendix C] ── 理論的基礎
    │                    │
    │                    └──> [Appendix D の §9] と接続 (設計支援の歴史的位置づけ)
    │
    └──> 将来の Spec v0.2 (Phase 2 着手時)
              ├── Phase 2+ Notes から昇格
              ├── Appendix A の確定部分を取り込み
              ├── Appendix B の確定部分を取り込み
              └── Appendix D の Designer Mode 仕様を取り込み (Phase 3+)
```

---

## 読み方ガイド (v0.2 更新)

### ケース 1: 実装に着手する開発者

1. `mystical-ci-spec-v0.1.md` を熟読
2. 必要に応じて `mystical-ci-phase2plus-notes-v0.1.md` で将来計画を確認
3. Appendix A, B, C, D は実装には直接必要ないが、設計の意図を理解したい場合に参照

### ケース 2: プロジェクトの全体像を理解したい新規参画者

1. 本 INDEX 文書
2. `mystical-ci-spec-v0.1.md` の §1〜§2
3. `mystical-ci-phase2plus-notes-v0.1.md` の §1 と §11
4. Appendix A の §1 と §2 のいくつかの式
5. Appendix B の §1〜§3
6. Appendix D の §1 (ツールの双方向性) — ツール理解の核心
7. Appendix C は興味があれば

### ケース 3: 設計の哲学的根拠を知りたい技術者

1. `mystical-ci-spec-v0.1.md` の §2
2. `mystical-ci-phase2plus-notes-v0.1.md` の §11
3. Appendix C 全文
4. Appendix D の §9 と §10 (歴史的位置づけと最終的な位置づけ)

### ケース 4: 描画様式 (式) の追加・変更を提案する人

1. Appendix A 全文
2. Appendix C §1〜§2
3. Appendix D §2 (異質式適用) も参照 — 新式が既存式の異質適用として既に説明できないか確認

### ケース 5: モジュール結合や景観可視化の改善を提案する人

1. Appendix B 全文
2. Appendix C §5
3. `mystical-ci-phase2plus-notes-v0.1.md` の §4
4. Appendix D §5 (式の合成) — 景観の合成的な見方

### ケース 6: 設計支援機能を活用したい開発者 ★新規

1. Appendix D 全文
2. Appendix A (式の体系を理解するため)
3. Appendix B (式の合成と景観の理解のため)
4. 実装が進んだら、Designer Mode の CLI リファレンス (将来作成予定)

### ケース 7: 新しいレシピを提案したい人 ★新規

1. Appendix D の §4 (レシピ集) と §3 (式駆動設計)
2. Appendix A (式の選択肢を理解するため)
3. 実例として、既存レシピが対応している既存フレームワーク (Elm, Erlang OTP 等) の調査
4. 提案レシピを Appendix D §4.2 に追記する PR を提案

### ケース 8: ユーザー (将来の)

実装後にユーザー向けドキュメント (チュートリアル、CLI リファレンス、Cookbook) が別途整備される予定。

---

## バージョン管理方針 (v0.1 から変更なし)

### バージョン番号

各文書は独立してバージョンが付与される。バージョンは `vX.Y` 形式:

- `X`: メジャーバージョン
- `Y`: マイナーバージョン

INDEX 文書自体もバージョン管理対象。本文書は v0.2 (Appendix D 追加に伴うマイナー更新)。

### 文書の昇格

`mystical-ci-phase2plus-notes-v0.1.md` および各 Appendix の各セクションは、対応する Phase が実装段階に入る際に、正式な仕様書のセクションに昇格する。

Appendix D の主要セクションは Phase 3-5 のいずれかで正式仕様書 (Designer Mode 仕様書) に昇格する想定。

### 仕様書とノートと補遺の関係

- **仕様書 (Spec)**: 実装の基準。コード作成の正典。
- **ノート (Notes)**: 議論と構想。確定していない事項を含む。
- **補遺 (Appendix)**: 特定領域の詳細。仕様書とノートを補完する。

各補遺はそれぞれ独立した関心領域を持ち、相互に補完的:

- A: 「式」の詳細体系
- B: スケールアップ (モジュール、景観)
- C: 理論的根拠
- D: ツールの双方向性 (設計支援)

---

## 今後追加が想定される文書 (v0.2 更新)

### 短期 (Phase 1 実装期)

- `mystical-ci-implementation-guide-v0.1.md` — Phase 1 実装ガイド
- `mystical-ci-ir-schema-v0.1.md` — IR スキーマ詳細仕様

### 中期 (Phase 2-3 実装期)

- `mystical-ci-filter-dsl-spec-v0.1.md` — フィルター DSL の言語仕様
- `mystical-ci-layout-algorithm-v0.1.md` — レイアウトアルゴリズム詳細仕様
- `mystical-ci-glyph-symbol-catalog-v0.1.md` — 記号体系の詳細カタログ
- `mystical-ci-designer-mode-spec-v0.1.md` ★新規 — Designer Mode の正式仕様書 (Appendix D から昇格)
- `mystical-ci-recipe-collection-v0.1.md` ★新規 — レシピ集の初期セット

### 長期 (Phase 4 以降)

- `mystical-ci-appendix-E-language-adapters-v0.1.md` — 言語アダプタの設計指針
- `mystical-ci-appendix-F-dynamic-analysis-v0.1.md` — 動的解析統合の詳細
- `mystical-ci-appendix-G-accessibility-v0.1.md` — アクセシビリティ実装詳細
- `mystical-ci-appendix-H-style-refactoring-v0.1.md` — 式変換 (Style Refactoring) の詳細

### ユーザー向け文書 (実装完了後)

- `mystical-ci-user-guide.md` — ユーザーガイド
- `mystical-ci-cli-reference.md` — CLI リファレンス
- `mystical-ci-cookbook.md` — レシピ集 (典型的なユースケース)
- `mystical-ci-design-tutorial.md` ★新規 — Designer Mode 入門

---

## 命名規則 (v0.1 から変更なし)

`mystical-ci-<カテゴリ>-<内容>-<バージョン>.md`

---

## このプロジェクトの起源について (v0.2 拡張)

本プロジェクトは、2026年5月19日付の GIGAZINE 記事「魔法陣のようなプログラミング言語『Mystical』」を契機とする対話から生まれた。オリジナル言語 Mystical の図形思想に着想を得つつ、対話の中で以下の発見が積み重ねられた:

1. 図形表現はコードレビューの注意誘導装置として機能しうる
2. 立体ビューは情報密度を高めるが、デフォルトでは平面が適切
3. 描画様式は単なる意匠ではなく、解析軸 (Call Graph vs Data Flow) の選択である
4. ファンタジー作品の魔法陣群は、群論的な対称性分類と独立に収斂進化している
5. モジュール単位の式と結合点により、コードベース全体が魔導都市的景観として可視化可能である
6. ★新規: ツールは双方向に使える。「コード → 魔法陣」だけでなく「求める性質 → 魔法陣 → コード」も可能。異質式適用により新パラダイムの引き寄せができる。

これらの発見が、本文書群として体系化された。発見6 (双方向性) はツールの位置づけを根本的に拡張する転換点であり、Appendix D として独立した文書になった。

プロジェクトは現時点で**設計フェーズ**にあり、実装は未着手である。

---

## 統計情報 (v0.2 時点)

- 文書数: 6 (中核 2 + 補遺 4)
- 総文字数: 約 80,000 字 (推定)
- 議論された式の数: 9 (ベルカ、うみねこ、Vesperia、ミッドチルダ3バリアント、Alchemy、黒執事、RWBY、Fate、夜天)
- 提案されたレシピ数: 12
- 議論されたハイブリッド合成数: 5

これらの数値は今後の文書追加とともに更新される。

---

## バージョン履歴

v0.1 — 初版。Spec、Phase 2+ Notes、Appendix A/B/C の 5 文書が出揃った段階での INDEX として作成。

v0.2 — Appendix D (設計支援モードと式駆動設計) の追加に伴う更新。文書数 6 に増加。ツールの双方向性を反映した記述に全面的に書き換え。読み方ガイドにケース 6 (設計支援活用) とケース 7 (レシピ提案) を追加。
