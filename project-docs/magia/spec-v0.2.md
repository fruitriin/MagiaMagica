# MagiaMagica 仕様書 v0.2

> **本文書の位置づけ**
> 本書は『MagiaMagica 仕様書 v0.1』(`spec-v0.1.md`) を基底とする**増補改訂版**である。
> 記載のない節は v0.1 がそのまま有効であり、本書は (a) Phase 1 実装 (M1〜M7 + 1.8) で
> 確定した事実の追認と、(b) Phase 2 実装 (2.1〜2.4) の契約となる新セクションのみを規定する。
> 昇格元は『Phase 2 以降の議論ノート v0.1』(`phase2plus-notes-v0.1.md`) 付録 A の指定に従う。
>
> 改訂方式の判断: v1.0 までは増補差分の積層とし、全文書き直しは v1.0 で行う
> (CLAUDE.repo.md「v1.0 前は破壊的変更を躊躇しない」と整合。差分が明確な方がレビュー可能)。

---

## §4 追補 — 中間表現 (IR) スキーマ

### §4.2 追補: Phase 1 実装で確定した型

v0.1 §4.2 に未記載のまま実装側で補完された型を正式仕様とする (擬似 Rust 構文)。
実体は `crates/magia-core/src/ir/` にあり、本節と乖離した場合は**実装側を正とする**
(v1.0 までは仕様が実装を追認する)。

```rust
/// 処理単位に紐づく追加情報 (Phase 1.2)
struct OperationPayload {
    source_excerpt: Option<String>, // 元ソース抜粋 (ホバー・デバッグ用)
    call_target: Option<String>,    // OperationKind::Call の呼び出し先フルパス候補
    early_return: bool,             // return / `?` 由来の早期リターンか
}

/// Sigil の重み・情報量 (Phase 1.1)。レンダリングの直径・太さの基礎値
struct Cardinality {
    weight: f64,            // Phase 1.3 以降は Operation 数を充填
    density: Option<f64>,
}

/// Edge の多次元レイヤー情報 (Phase 1.1)
struct EdgeLayerData {
    call_frequency: Option<u64>,  // 動的解析 (Phase 5+)
    data_volume: Option<f64>,     // データフロー量 (Phase 3+)
    labels: Vec<String>,
}

/// プロジェクトメタ情報 (Phase 1.1)
struct ProjectMetadata {
    name: String,
    version: Option<String>,
    root_path: Option<String>,
}

/// AuxRing が親リングに対して持つ役割 (Phase 1.3)。
/// レイアウトが「親リング上のどこに補助リングを置くか」を決める基準。
/// AuxRingKind / LoopKind も Phase 1.3 で導入 (既定値は IfBranch / For)。
struct AuxRingRole {
    kind: AuxRingKind,
    anchor_operation: u32, // 親 content 内の対応 Operation 添字 (入口 = 出口)
    ordinal: u32,          // 同一制御構造内の序数 (if 連鎖・match アーム)
    label: Option<String>, // match アームのパターン文字列等
}

enum AuxRingKind {
    IfBranch,
    ElseBranch,
    MatchArm,
    LoopBody(LoopKind),
}

enum LoopKind { For, While, Loop }
```

`ControlFlowInfo` (v0.1 §5.2 の `control_flow` レイヤーの実体) は
`branch_count` / `loop_count` / `early_return_count` / `role: Option<AuxRingRole>` を持つ。
カウント系は**そのリングの content に直接現れる構造のみ**を数える
(入れ子は対応する AuxRing 側に計上。総和はリングを辿って取る)。

**match アームガードの扱い (Phase 1.4 確定)**: `pat if cond =>` のガード式は
親スコープで評価されるため、ガード中の call site は親リング側に係留する。
ガード自体はアームの分岐数に影響しない (通常アームと同一視)。

### §4.3 追補: EdgeLayerData 非対称問題の Phase 3 方針

`LayerData` が `Option<XxxInfo>` の積層であるのに対し、`EdgeLayerData` はフラットな
直値を持ち非対称である (Phase 1.1 からの既知課題)。**Phase 3 でデータフロー解析を
導入する際、`EdgeLayerData` を `LayerData` と同様の設計パターン (`Option<XxxInfo>`
の積層構造) へ破壊的に再設計することを既定方針とする**。Phase 3 計画書はこの再設計を前提に起こす。

### §4.4 新設: JSON 表現の規約

- IR の JSON は serde の既定表現に従う。enum は externally-tagged であり、
  データを持つバリアントはオブジェクト形式になる (例: `AuxRingKind::LoopBody(For)` は
  `{"LoopBody": "For"}`、単純バリアントは `"IfBranch"`)。
- **この不統一 (文字列とオブジェクトの混在) を v0.2 の正式仕様とする**。
  根拠: 現状の JSON コンシューマーは serde を介する Rust 実装 (dev-server 含む) のみで
  実害がなく、`#[serde(tag)]` 等への統一はスナップショット全更新を伴う。
  非 Rust の外部コンシューマーが現れた時点で統一を再判断する (再訪条件の明示)。
- 混在の具体例 (`AuxRingRole` の JSON):

  ```json
  {"kind": "IfBranch", "anchor_operation": 0, "ordinal": 1, "label": null}
  {"kind": {"LoopBody": "For"}, "anchor_operation": 2, "ordinal": 0, "label": null}
  ```

- 全 struct は `#[serde(default)]` を持ち、`deny_unknown_fields` は付けない
  (前方互換の読み込みを許す。v0.1 §4.1 原則の具体化)。

---

## §5 拡張 — レイヤーシステム (Phase 2)

§5.2 への追補 (Phase 1 の追認) と、notes §2 を正式化する新設節 (§5.3〜5.5。
いずれも v0.1 に存在しない節番号であり衝突しない)。

### §5.2 追補: Phase 1 効果判定の近似仕様 (Phase 1.4 確定)

効果カテゴリ判定は意味解決を行わない近似である (tech-selection §2.1 Phase 1a):

- 呼び出し先パスは「記述されたまま + 同ファイル内 `use` 文の機械的展開」で解決する。
  `use` の Path/Name/Rename/Group を再帰展開し、Glob は無視。同名は後勝ち
- メソッド呼び出し (`x.method()`) はレシーバ型が不明のため `.method` 形式で保持し
  **pure 扱い**とする (Phase 1b の意味解決導入で再訪)
- マクロは名前ベースの白リスト判定のみ (展開しない):
  `println` `eprintln` `print` `eprint` `dbg` `format` `write` `writeln` → io
  (表記は `!` なしのマクロ名。実体は `effects.rs::IO_MACROS` を参照)。
  白リスト外は pure。マクロのトークン列内部の call は抽出されない (既知の限界)
- `format!` を io に倒すのは様子見の暫定判断 (オーナー確定 2026-06-10)。
  false positive が目立てば pure へ変更する
- パス前方一致はセグメント境界つき (`std::io` は `std::iox` に一致しない)。
  効果表は `effects.rs` に集約し、`tokio::io` は意図的に未登録 (小さく始める方針)
- 複数効果フラグの表示色は unsafe > network > db > filesystem > io > pure の優先順位で
  1色に潰す (レンダラの色相規約、v0.1 §6.1.3 の補完)

### §5.3 新設: レイヤーの実装モデル

- レイヤーは「関心の軸」であり、**IR への問い合わせ (射影) として実装する**。
  IR 自体はレイヤーを知らない。
- SVG 出力はレイヤーごとに `<g class="layer-<kebab-case>">` で分離する (v0.1 §6.1.5)。
  Phase 2 の対話的切替は SVG を再生成せず CSS クラス操作のみで行う。

### §5.4 新設: 位置共有制約 (必須要件)

同じ関数の同じ Sigil は、**どのレイヤー組み合わせでも常に同じ位置**に描画されなければ
ならない。レイアウトエンジンは「レイヤーに依存しない位置決定」を行い、レイヤーは
「決まった位置の上に情報を重ねる」分業を厳守する。この制約は Phase 2.2 でテストとして
機械化する (レイヤー選択が `LayoutResult` に影響しないことの検証)。

### §5.5 新設: レイヤー操作 (Phase 2.2 実装範囲)

- 表示 ON/OFF、透明度調整 (CSS opacity)
- 切替状態の URL クエリ反映 (リロード・共有での再現)
- 色相変更・AND/OR 合成・プリセット保存は Phase 2.3 のフィルター言語と合流して実装

---

## §6 追補 — レンダリングと視覚化

### §6.1.4 追補: 衝突回避 (Phase 1.8 確定)

安定レイアウト要件 (決定論) を保ったまま、以下の衝突回避を行う:

1. 兄弟 AuxRing は必要角度幅 `2·asin((r+margin)/d)` に基づき動的配分する
   (同一 anchor は中央寄せ扇、異 anchor 間は貪欲な最小間隔強制 + 均等割フォールバック)
2. 1軌道の容量 `floor(TAU/step)` を超える子は距離を1段伸ばした第2軌道以降へ送る
3. 入れ子 AuxRing は親の outward を中心とした**半円**に展開を制限する
   (祖父母方向への逆流の防止。anchor はビン中央 `(anchor+0.5)/len` で写像)
4. 親をまたぐリングの残存重なりは固定パス数の決定論的緩和 (大きい SigilId 側の押し出し)
   で解消する。重なりが無ければ完全に no-op
5. SummonGlyph は全配置済みリングの占有角度帯を避け、**衝突した個体のみ**帯の出口へ
   退避する。占有帯は「glyph 中心とリング中心の距離が clearance (= 両半径 + margin)
   を下回る角度範囲」を余弦定理で角度に変換して得る (接触可能性の厳密判定は半径方向
   フィルタ |R−g| ≤ clearance が担う)。無衝突の glyph は基本配置 (全周均等割) のまま
6. シグネチャ円弧はラベル帯に半径方向で食い込む上半分要素がある場合のみ外側へ拡張する

force-directed・乱数は引き続き使用しない。

---

## §7 正式化 — dev-server (Phase 2.1)

notes §1 を Phase 2.1 の実装範囲に絞って正式化する。

- `magia serve <FILE> --fn <NAME> [--port]` で常駐サーバを起動し、SVG をインライン
  埋め込みした HTML を配信する (`<FILE>` / `--fn` の形式は Phase 1.7 実装済みの
  `magia render` と同一規約。serve は新設サブコマンド)
- 対象ファイルを監視し、変更検知で parse → layout → render を再実行、
  WebSocket (または SSE) でブラウザへ更新を push する
- 解析エラー時は直前の正常な図を保持したままエラーを表示する (会話を切らない)
- notes §1.2 の3層分離 (永続キャッシュ / インクリメンタル再計算 / 差分パッチ配信) は
  **将来構想として保留**する。単一関数の全再生成は数ミリ秒で済み、Phase 2 規模では
  不要 (導入判断は多関数対応のとき)

---

## §8 正式化 — フィルター言語 (Phase 2.3)

notes §3 のサブセットを最小文法として確定する。`.magia` ファイル (git 管理可能):

```
# コメント
show: control_flow + effects[network, db]
hide: type_info
```

- ディレクティブは `show:` / `hide:` の2種。`hide` が `show` に優先する
- レイヤー名は §5 の語彙 (CLI `--layers` と共通)。`effects[カテゴリ, ...]` で
  効果カテゴリの絞り込みができる (該当カテゴリの記号のみ残す)
- `highlight:` / `filter:` (メトリクス条件) は**予約語**とし、Phase 3 で導入する
- 効果カテゴリ絞り込みは CSS では表現できないため render 時に適用する
  (`FilterSpec` を render の入力に加える API 拡張。v1.0 前の破壊的変更として許容)

---

## §15 新設 — アクセシビリティ (Phase 2.4)

notes §9 を正式化する。

- 同じ IR から構造化テキスト「呪文書き起こし (Incantation Transcript)」を生成する。
  SVG と書き起こしが同一 IR の射影であることが内容の一致を保証する
- 出力は装飾なしのプレーンテキスト・決定論的。含む情報:
  メインリングの規模 (Operation 数)、補助リングの数と種別 (入れ子含む)、
  外部呼び出しの集計 (呼び出し先・効果カテゴリ・回数)、早期リターン経路数、
  戻り値型 (Result/Option)、async/await
- CLI `magia transcribe`、および dev-server の HTML への埋め込みで提供する
  (ARIA 属性 `aria-label` または視覚的に隠したテキスト要素としてスクリーンリーダーに露出する)
- レイヤー切替対応は Phase 3、動的解析情報を含む包括版は Phase 5 で拡張する

---

## 付録 — v0.1 からの差分一覧

| 節 | 種別 | 内容 |
|---|---|---|
| §4.2 追補 | 追認 | Phase 1.1/1.3 実装の補完型 (OperationPayload, Cardinality, EdgeLayerData, ProjectMetadata, AuxRingRole, AuxRingKind, LoopKind)、ControlFlowInfo のカウント規約、アームガードの扱い |
| §4.3 追補 | 方針 | EdgeLayerData 非対称の Phase 3 再設計方針 |
| §4.4 新設 | 規約 | JSON 表現 (serde 既定・LoopBody のオブジェクト形式を正式化、再訪条件つき) |
| §5.3〜5.5 新設 | 契約 | レイヤー = IR への射影、位置共有制約、Phase 2.2 の操作範囲 |
| §5.2 追補 | 追認 | Phase 1.4 効果判定の近似仕様 (use 展開・メソッド pure・マクロ白リスト) |
| §6.1.4 追補 | 追認 | Phase 1.8 の衝突回避6項目 |
| §7 正式化 | 契約 | dev-server の Phase 2.1 実装範囲 (3層分離は保留) |
| §8 正式化 | 契約 | フィルター言語の最小文法 (show/hide + effects[]) |
| §15 新設 | 契約 | 呪文書き起こし (Phase 2.4) |

## 付録 — バージョン履歴

- v0.2 (2026-06-11) — Phase 1 完了時点の実装追認 + Phase 2 (2.1〜2.4) の契約を増補。
  基底は v0.1 (記載のない節は v0.1 が有効)
- v0.1 — 初版 (Phase 1 スコープの厳密定義)
