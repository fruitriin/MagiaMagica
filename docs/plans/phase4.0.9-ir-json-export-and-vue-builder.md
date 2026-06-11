# Phase 4.0.9 — IR JSON エクスポート + Vue IR ビルダ (案2)

## 出典

- オーナー方針 (2026-06-11): IR から SVG を作るロジックも Vue にリメイク。4.0.7 で作ったコンポーネントは案2 でも活用 (案2)
- 親計画: Phase 4.0.7 (案1) で確立した `MagicCircleSchema` + Vue コンポーネントツリーは流用。**Schema の埋め方を SVG パース → IR ビルドに差し替える**
- 後続: Phase 4.1 (ピン中心ビュー) は本計画完了後の世界 (Vue 1本化) で書く。Phase 4.3 (静止画) も IR + Vue SSR 経路に統一

## 目的

Rust 側で IR (Phase 1.1 〜 3.4 で育てた `MagiaIR`) を JSON 形式でエクスポートし、Vue 側に **IR → `MagicCircleSchema` ビルダ** を実装する。
4.0.7 で書いた SVG パーサを廃止し、Vue が **意味論ベース** で魔法陣を構築する世界に移行する。

これにより:
- SSE 更新が IR JSON だけになる (SVG 文字列を毎回送る必要なし)
- Vue がリアクティビティを最大限活用できる (`<Transition>` / `v-for` / `:class` が宣言的)
- Phase 4.3 (静止画) の Vue SSR 経路で同じ IR が使える (動的UI と静止画でレンダラ完全1本化)

## スコープ

### やること

- **Rust: IR JSON エクスポート**:
  - `magia-core::Ir` に `Serialize` 実装 (既存箇所が serde で派生済か確認、未対応分を追加)
  - `magia-cli/serve`: `GET /spell/<fn>?format=ir` で IR JSON を返す。`format=svg` (既定) は当面互換維持、本計画完了時に廃止判定
  - JSON スキーマは `project-docs/magia/spec-v0.3.md` に正式定義として追加 (本計画内で spec も更新)
- **Vue: IR ビルダ**:
  - `web/src/converters/irToSchema.ts` — IR JSON → `MagicCircleSchema` 変換
  - 配置済 (x,y) 座標は **Rust が IR に含めて返す** 設計に変更 (レイアウト計算は Rust に集約、POSD)
  - 効果カテゴリ・操作種別の意味論を Schema フィールドに素直に展開
- **4.0.7 の SVG パーサ廃止**:
  - `web/src/converters/svgToSchema.ts` 削除
  - `<MagicCircleView>` は `/spell/<fn>?format=ir` から IR を取得し、`irToSchema` で Schema 構築
- **Rust SVG レンダラの deprecate マーク**:
  - `midchilda.rs` / `belka.rs` の SVG 出力部分に「Phase 4.3 で削除予定」コメント
  - **削除は 4.3 で実施** (4.3 が Vue SSR 経路に切替えるタイミング)
- **テスト**:
  - IR JSON のスキーマ golden (既存 fixtures で版)
  - Vitest: `irToSchema` の単体 (IR → Schema 変換)
  - 既存の Vue コンポーネント (4.0.7 で書いた) が新ビルダ経由でも無修正で動く

### やらないこと (別計画)

- Rust SVG レンダラの削除本体 (Phase 4.3 で deprecate コメント → 削除)
- ピン中心レイアウト計算 (Phase 4.1 で Rust 側に実装、IR に追加フィールド)
- Vue SSR (Phase 4.3)

## 設計上の判断

### IR JSON にレイアウト済 (x,y) を含める

Rust = レイアウト計算、Vue = 描画 (POSD 分担) を守るため、IR JSON に **配置済座標を含めて返す**。
- Vue 側でレイアウトを再計算しない (二重実装を作らない)
- IR スキーマは spec v0.3 で正式定義
- レイアウト変更は Rust 側 1箇所で完結 (Phase 4.1 のリング配置も Rust に置く)

### `MagicCircleSchema` は無修正

4.0.7 で固めたスキーマを変えない。変えるのは「埋める入口」(SVG パース → IR ビルド) だけ。
これが本計画の最大のスムーズさ。Vue コンポーネント・Pinia store・操作ロジックは全て無修正で動く。

### `?format=svg` 互換は本計画完了時に廃止判定

4.0.9 完了時点で:
- Vue クライアントは `?format=ir` のみ使う
- Rust SVG レンダラは `?format=svg` でアクセス可能だが、Vue クライアントは使わない
- 4.3 で Rust SVG レンダラ削除と同時に `?format=svg` も削除

v1.0 前破壊的変更ポリシーに従い、過渡的 deprecate を残さない。

### IR JSON サイズ管理

SVG 文字列より IR JSON の方が一般に軽い (装飾の冗長性がない)。
ただし fixture によっては配置済座標で膨らむ可能性。1関数あたり 100KB を超えたら gzip エンドポイントを追加検討 (現状 fixture では問題ない見込み)。

## 受け入れ基準

- [x] ~~`?format=ir`~~ → `/spell/<fn>` レスポンスの `ir` フィールドとして配信 (実装時判断: 使い手は自前クライアントのみで、v1.0 前は旧 `svg` を消す方針と整合)。スキーマは spec v0.3 §16 に定義
- [x] Vue クライアントが IR JSON 経由で魔法陣を描画 (オーナー判定済み:「左右同じに見える」。実測: fuzz 5% 残差 0.02〜0.03% = テキスト AA のみ)
- [x] `web/src/converters/svgToSchema.ts` 削除済み
- [x] `MagicCircleSchema` 型と Vue コンポーネント (4.0.7 のもの) は **symbols 追加** (記号の意味論化 — raws 素通しが IR 経由では作れないため) 以外無修正。既存コンポーネントは全て無修正で流用
- [x] Rust SVG レンダラに deprecate コメント追加 + ベルカは保守方針 (リメイクまで保守価値低め — オーナー判定 2026-06-11) を明記
- [x] `cargo test --workspace` (17 スイート — クロス検証テスト追加) / Vitest 30 / Playwright 9 通過

## 実装結果メモ (2026-06-11 完了)

- **スコープ確定 (オーナー判定)**: ベルカ式は SVG 温存 (Vue 移植は 4.3)。ミッドチルダのみ IR 直結
- **レイアウト/描画の境界**: 「どこに何があるか (座標・操作配置・衝突回避済み円弧 path)」= Rust (`render::ir_export`)、「どう描くか (記号の頂点・エッジ端点)」= Vue (`SymbolMark` / `irToSchema`)。二重実装なしで画素等価を達成
- **id の扱い**: SigilId は JSON 内の相互参照 (edges の from/to) 専用と spec §16 に明記 (永続化禁止 — Phase 3.2 方針)。4.0.7 で予約した Schema の from/to が IR 由来で埋まった
- **クロス検証テスト** (`ir_export_cross_check.rs`): SVG レンダラと ir_export の規則コピーが 4.3 までにずれる事故を、同一 fixture の両経路出力の座標比較で防ぐ。追加直後に **-0.0 (負のゼロ) の JSON 混入を検出** (screen_position の y 反転由来) — `nz()` 正規化で解消
- **レビューで実バグ1件**: TS 側 RETURN_BRANCH_LENGTH が 30 (Rust は 26.0)。言語間の手書き定数コピーはクロス検証 or 定数を契約に含める設計が要る (今回は描画定数を spec 記載 + コメント強化で対応)
- IR JSON サイズ: write_document で ~8KB (旧 svg ~11KB より小) — gzip 不要

## 後続候補

- Phase 4.1 で IR にピン中心レイアウト情報を追加 (focal 関数の中心配置 + 距離リング座標)
- Phase 4.3 で Rust SVG レンダラ削除 + Vue SSR 経路に統一

## 実装ステップ (粗粒度)

1. Rust: IR の `Serialize` 派生確認 + 不足分追加
2. Rust: `serve` に `?format=ir` ルート追加、既存 `?format=svg` 維持
3. spec v0.3 に IR JSON スキーマ正式定義 (`project-docs/magia/spec-v0.3.md` 追補)
4. Vue: `irToSchema.ts` 実装 + Vitest
5. `<MagicCircleView>` を `?format=ir` 経由に切替
6. `svgToSchema.ts` 削除
7. Rust SVG レンダラに deprecate コメント
8. 既存 fixtures で画素単位等価のオーナー判定
9. Stage 1 品質ゲート + コーディング知見
10. Stage 2 レビュー + 指摘対応
11. 完了処理 (4.1 と 4.3 の前提が揃った宣言)

## 想定リスク

- **IR JSON スキーマの完成度**: Phase 1.1 で「将来 Phase の場所を確保」した IR スキーマがそのまま JSON 化できるか実装時確認。serde の `#[serde(default)]` で過渡対応
- **配置済座標を IR に持たせる影響**: 既存 IR の責務 (パース・解析結果) にレイアウトが混ざる。`IrLaidOut` のような派生型で分けるか、IR を直接拡張するかは実装時判断 (POSD 的には派生型推奨)
- **Vue 側の表示差分**: 4.0.7 と画素単位等価にならない場合、Rust のレイアウト出力を IR にどう乗せるかで調整 (例: 微小回転、装飾要素の扱い)