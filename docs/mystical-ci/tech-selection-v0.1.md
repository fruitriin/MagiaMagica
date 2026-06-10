# Mystical CI 技術選定・実装着手ガイド v0.1 (draft)

> 位置づけ: `mystical-ci-spec-v0.1.md` §3 のアーキテクチャ決定を、クレートレベルの具体選定と実装順序に落とす文書。INDEX v0.5 の「今後追加が想定される文書」のうち `mystical-ci-implementation-guide-v0.1.md` のたたき台に相当する。
> 作成日: 2026-06-10

---

## 1. ワークスペース構成

spec §3.2 の3モジュールを cargo workspace として構成する。

```
mystical/
├── Cargo.toml            # [workspace]
├── crates/
│   ├── mystical-core/    # IR定義・解析・レイアウト・SVGレンダラ
│   ├── mystical-rust/    # Rust → IR アダプタ
│   └── mystical-cli/     # CLI
└── fixtures/             # スナップショットテスト用サンプルコード
```

IR定義は将来 `mystical-ir` として独立クレート化する可能性があるが、Phase 1 では `mystical-core::ir` モジュールで開始し、Phase 2 着手時に分離を判断する（早すぎる分割を避ける）。

## 2. クレート選定

### 2.1 構文解析: `syn` 2.x（主）+ 意味解決は段階導入

spec §10.3 の収集情報のうち、関数定義・制御構造・早期リターン・async/await・unsafe・call site の構文位置は **`syn` 単体（features = ["full", "visit", "extra-traits"]）で全て取得可能**。

唯一の難所は「呼び出し先関数のフルパス解決」。完全な名前解決には rust-analyzer の HIR が必要だが、`ra_ap_*` クレート群は重く API が不安定。そこで段階導入とする:

- **Phase 1a**: `syn` のみ。call site のパスは記述されたまま + 同ファイル内の `use` 文の機械的展開で近似解決する。effects 推定（spec §5.2）は crate 名先頭セグメントのヒューリスティックなので、この近似で実用上十分
- **Phase 1b（必要になったら）**: `ra_ap_hir` / `ra_ap_ide` による意味解決を `mystical-rust` 内のオプション機能（feature flag `semantic`）として追加。core の IR には影響しない

この分割により「動くものを最速で出す」と「正確さの天井」を両立する。

### 2.2 グラフ構造: `petgraph`

Sigil/Edge のグラフ操作（到達可能性、トポロジカル順序、交差最小化の前処理）に使用。IR 自体は spec §4.2 の素直な struct で持ち、解析時に petgraph へ射影する（IR をグラフライブラリの型に依存させない）。

### 2.3 幾何・パス生成: `kurbo`

リングの円弧、ベジェ接続線、極座標配置の計算に linebender の `kurbo` を使用。SVG パス文字列への変換も `kurbo::BezPath::to_svg()` がそのまま使える。自前の三角関数地獄を回避できる。

### 2.4 SVG 生成: 自前ビルダー（`svg` クレートは不採用）

spec §6.1.5 の要件（レイヤーごとの `<g>` 分離、class 属性、決定論的出力）はテンプレート的な構造なので、`svg` クレートの抽象を挟むより `std::fmt::Write` ベースの薄い自前ビルダーのほうが出力を完全制御できる。属性順序も固定でき、スナップショットテストが安定する。

### 2.5 CLI: `clap` 4.x（derive）

```
mystical render <FILE> --fn <NAME> --layers control_flow,effects -o out.svg
mystical list <FILE>          # 関数一覧
```

### 2.6 テスト: `insta` スナップショットテスト

spec §6.1.4「同じ IR からは常に同一の SVG」という決定論要件は、insta による SVG 全文スナップショットがそのまま回帰テストになる。`fixtures/` の Rust サンプル → SVG のゴールデンテストを最初から CI に組み込む。

### 2.7 その他

- シリアライズ: `serde` + `serde_json`（IR の `--emit-ir` デバッグ出力。将来の `mystical-ci-ir-schema` 文書の実体化にもなる）
- エラー: `thiserror`（ライブラリ側）+ `anyhow`（CLI側）
- 乱数: 不使用を原則とする。レイアウトのタイブレークが必要な場合のみ `SigilId` の決定論的ハッシュで代替（spec §6.1.4 の固定シードよりさらに強い保証）
- インクリメンタル計算 (`salsa`): **Phase 1 では不採用**。単発 CLI に不要。Phase 2 の dev-server 化で導入を再検討（spec 付録Aに既載）

## 3. 実装順序（波乗り用マイルストーン）

各マイルストーンは独立に動作確認できる。

1. **M1: IR スケルトン** — spec §4.2 の型定義を全フィールド込みで実装（Phase 1 で埋めないフィールドも `Option`/空で確保）。serde 導出。ユニットテストで JSON round-trip
2. **M2: syn → IR** — 単一関数の AST から MainRing + Operation 列を構築。`--emit-ir` で JSON 確認
3. **M3: 制御構造の AuxRing 化** — if/match/loop を AuxRing として抽出、ControlFlow Edge で接続
4. **M4: 召喚記号と効果判定** — call site 抽出、crate 名ヒューリスティックで EffectSet 付与
5. **M5: レイアウトエンジン** — spec §6.1.4 の優先順位（中央固定 → 極座標 → 召喚記号 → 交差最小化）を実装。決定論性のテスト
6. **M6: SVG レンダラ** — kurbo でパス生成、`<g class="layer-*">` 分離、色相規約（spec §6.1.3）適用。insta ゴールデンテスト開始
7. **M7: CLI 統合と磨き** — clap、エラーメッセージ、README、fixtures 拡充

M2 完了時点で「IR が出る」、M6 完了時点で「魔法陣が出る」。Fable に投げる場合も、このマイルストーン単位でセッションを区切ると暴走しにくい。

## 4. リポジトリ初期設定

- Rust edition 2024 / MSRV はその時点の stable
- `rustfmt` + `clippy`（`-D warnings`）を最初から CI に
- コミットログは日本語、Conventional Commits 風プレフィックス（`feat:` `fix:` など）は任意
- ライセンス: 未定（公開意向があるなら MIT/Apache-2.0 デュアルが Rust 慣習）

## 5. 本文書で決めなかったこと

- `ra_ap_*` 導入の具体時期（Phase 1a の精度実測後に判断）
- IR スキーマ文書の正式化（M1 完了後に `--emit-ir` 出力から生成）

## 6. 決定事項の追記 (2026-06-10)

**Phase 1 のフラッグシップ式: ミッドチルダ式（簡略・ConcentricRings バリアント）** — Appendix A v0.2 §2.4 で唯一「実装 Phase: Phase 1」と指定されている式であり、本日の選定基準とも一致する。

- 汎用性: (Four, Balanced)。全方位等価で特定ドメインの意味論を背負わず、デフォルト式に適する
- 実装容易性: 同心円の多重連環＋内接正方形のみで構成され、円・正多角形・固定角度で描画可能。kurbo での実装コストが全式中最小
- 対抗馬: ベルカ式（Three, Convergent）も幾何学的には単純だが、Convergent の「中心の変換器」という意味論がデフォルト式には強すぎるため次点。Appendix A でも Phase 3 指定

M6（SVG レンダラ）はミッドチルダ式 ConcentricRings の1式のみを実装し、`RenderStyle` enum は将来の式追加に開いた形で定義する。
