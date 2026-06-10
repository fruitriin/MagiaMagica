# まぎあマギカ計画 (MagiaMagica)

コードベースを「魔法陣」として描画する可視化システム。

プログラムの構造を、ファンタジー世界の魔法陣として空間にマッピングする。単なるビジュアル趣味ではなく、**空間認知（近接・グルーピング・境界・対立）を母語とする思考者のためにコードを翻訳する装置**である。フローチャートより魔法陣のほうが「読める」人のための可視化ツール（[コンセプト文書](project-docs/concept.md) より）。

Phase 1 では **単一の Rust 関数をミッドチルダ式 ConcentricRings の SVG として描画する** CLI を提供する。

## インストール

```bash
cargo install --path crates/magia-cli
```

## 使い方

### 関数を魔法陣として描画する

```bash
magia render fixtures/simple_compute.rs --fn simple_compute -o simple_compute.svg
```

`-o` を省略すると標準出力に SVG を吐く。`--layers control_flow,effects,type_info`（カンマ区切り）で描画レイヤーを絞り込める。

```bash
# 効果カテゴリ (色相) のレイヤーだけを描画
magia render fixtures/io_print.rs --fn io_print --layers effects
```

### ファイル内の関数を一覧する

```bash
magia list fixtures/match_dispatch.rs
```

### 中間表現 (MagiaIR) を JSON で見る

```bash
magia emit-ir fixtures/result_chain.rs --fn result_chain
```

### 自己ホスティング — MagiaMagica 自身を魔法陣化する

このリポジトリそのものが描画サンプルである（リポジトリを clone し、ルートで実行する）:

```bash
magia render crates/magia-core/src/layout/mod.rs --fn layout_with -o layout_with.svg
```

代表関数4つを一括生成するサンプルも用意してある:

```bash
cargo run -p magia-rust --example render_self   # → target/self-hosting/*.svg
```

## 読み方 (ミッドチルダ式)

- **メインリング** = 関数本体。3時の位置を起点に反時計回りに処理（ドット）が並ぶ
- **補助リング** = 制御構造（if の分岐先・match のアーム・ループ本体）。Y 字 = 分岐、内側の矢印 = ループ
- **召喚記号** = 他関数の呼び出し。色相が効果カテゴリを表す:
  黒 = 純粋計算 / 青 = IO / 紫 = ネットワーク / 緑 = DB / 茶 = ファイルシステム / 赤 = unsafe
- **二重線のメインリング** = async fn、**リングを内から外へ抜ける矢印** = 早期リターン（`return` / `?`）、**9時から出る実線・破線の分岐** = Result/Option 戻り値の正常・異常パス

`fixtures/` に効果カテゴリ別の合成サンプルが10個ある。

## Phase 1 のスコープと制限

- 解析対象は **Rust の単一関数**（1関数 = SVG 1枚）。`syn` ベースの構文解析のみで、意味解決はしない
- 呼び出し先のパスは「記述されたまま + 同ファイル内 `use` 文の機械的展開」の近似。メソッド呼び出しはレシーバ型が分からないため黒（純粋扱い）になる
- マクロは `println!` 等の白リストを名前判定するのみ（展開しない）
- レイアウト・描画は決定論的（同じコードからは常に同じ SVG が出る）

詳細仕様は [project-docs/magia/spec-v0.1.md](project-docs/magia/spec-v0.1.md) を参照。

## ライセンス

MIT
