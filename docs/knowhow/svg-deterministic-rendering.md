# 決定論的 SVG 生成の定型

> Phase 1.6 (ミッドチルダ式レンダラ) で確立。`std::fmt::Write` ベースの自前 SVG
> ビルダー + insta スナップショットの組み合わせで使うパターン集。

## 発見した知見

- **raw string と色コードの衝突**: `r#"…"#` は中身に `"#` の並び (例: `stroke="#000000"`)
  があるとそこで終了してしまい、難読なコンパイルエラーになる。`"#` を含むリテラルだけ
  `r##"…"##` にする。全部 `r##` に倒すと今度は clippy `needless_raw_string_hashes` が出る
- **数値属性は固定桁で文字列化**: `format!("{v:.2}")` + `-0.00` → `0.00` の正規化。
  最短表現 (`{}`) は `204.00000000000003` のような浮動小数ノイズを拾い、
  スナップショットが壊れやすい
- **テキストノードは XML エスケープ必須**: Rust のシグネチャは `&str` / `Result<T, E>` の
  形で `&` `<` `>` を普通に含む。`&→&amp;` を最初に置換すること (順序を誤ると二重エスケープ)
- **insta のスナップショット配置変更**:
  `insta::with_settings!({ snapshot_path => "fixtures/snapshots", prepend_module_to_snapshot => false }, { ... })`。
  初回生成は `INSTA_UPDATE=always cargo test` (cargo-insta 未インストールでも .snap が直接書かれる)
- **SVG の目視確認 (macOS)**: `qlmanage -t -s 800 -o outdir file.svg` で PNG サムネイルが
  作れる (rsvg-convert / resvg のインストール不要)。AI 自身も生成 PNG を Read して
  レイアウト破綻を確認できる
- **座標系**: 画面座標 (y 下向き) で「3時起点・反時計回り」を視覚的に保つには、数学座標
  (y 上向き・反時計回り正) を `y_svg = -y_math` と単純反転するだけでよい。math の CCW は
  反転後も画面上で右 → 上 → 左 → 下 の CCW に見える (符号としての向きは反転するが視覚は保たれる)
- 上半円の円弧 (シグネチャラベル用) は画面座標で `kurbo::Arc::new(center, radii, PI, PI, 0.0)`
  → `arc.to_path(0.1).to_svg()`。start=π (左) から +π 掃引で中点が画面の真上を通る

## プロジェクトへの適用

- `magia-core/src/render/` — `render(graph, layout, style)`。レイヤーは
  `<g class="layer-control-flow">` / `layer-effects` / `layer-type-info` の3層 (spec §6.1.5)
- 色相規約は `render/palette.rs` に分離。`EffectSet` の複数フラグは
  unsafe > network > db > filesystem > io > pure の優先順位で1色に潰す
- E2E ゴールデンテストは parse_function が必要なため magia-rust 側に置く
  (magia-core ↔ magia-rust の dev-dep 循環を避ける)

## 注意点・制約

- `qlmanage` の出力は `<元ファイル名>.png` 固定。テキスト (textPath) のレンダリングは
  ブラウザと差がありうるので、最終確認はブラウザで行う
- スナップショットは属性順・要素順に依存する。出力順は IR の `Vec` 順 +
  `BTreeMap` 走査順 (= SigilId 順) に固定してある

## 参照

- `crates/magia-core/src/render/midchilda.rs`
- `crates/magia-rust/tests/render_golden.rs`
- `crates/magia-rust/examples/render_self.rs`
- `project-docs/magia/spec-v0.1.md` §6.1.3 / §6.1.5
