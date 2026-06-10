# clap 4 derive で CLI を統合するパターン

> Phase 1.7 (magia CLI) で確立。ライブラリ群 (core / 言語アダプタ) を薄い CLI で
> 束ねるときの定型と落とし穴。

## 発見した知見

- **予約語フラグ**: `--fn` のような Rust 予約語のフラグは
  `#[arg(long = "fn", value_name = "NAME")]` + フィールド名 `fn_name` で実現する
- **カンマ区切りリスト**: `#[arg(long, value_delimiter = ',')]` で
  `--layers a,b,c` が `Option<Vec<String>>` に直接受かる (自前 split 不要)
- **エラーメッセージの責務分担**: ライブラリ側 thiserror の `Display` に含まれない
  表示用情報 (syn 構文エラーの行番号等) は、CLI 層で variant を match して
  `anyhow!("{error} ({}行目...)")` と付加する。プレゼンテーション都合で
  ライブラリの `Display` を太らせない (表示の複雑さは表示層が持つ)
- **assert_cmd**: `Command::cargo_bin("magia")` は `[[bin]] name` を参照する
  (クレート名ではない)。統合テストの `CARGO_MANIFEST_DIR` はクレートルートなので、
  workspace ルートの資産 (fixtures/) へは `../../` で到達する
- **構造化出力の後処理フィルタ**: SVG のレイヤーフィルタ (`--layers`) は
  「1行1要素・レイヤー `<g>` は入れ子なし」というレンダラの出力規約に依存した
  行単位スキップで十分。**規約依存の処理は、依存元と依存先の双方のコメントで
  相互参照させる** (片方だけ変更される事故の防止)
- exit code: `main` で `run()?` を受けて `eprintln!("エラー: {error:#}")` +
  `std::process::exit(1)`。anyhow の `{:#}` で Context チェーンが1行に繋がる

## プロジェクトへの適用

- `crates/magia-cli/src/main.rs` — render / list / emit-ir の3サブコマンド
- fixture はワークスペースルート `fixtures/` に置き、CLI 統合テストと README の
  使用例が同じファイルを共有する (テストが通る = README が再現可能)

## 注意点・制約

- `cargo install --path crates/magia-cli` の動作確認は、シェルの PATH に
  cargo bin ディレクトリが入っていない環境ではフルパスで叩いて検証する
- ファイル名と同名の代表関数を fixture に置く規約にすると、統合テストの
  ループが `(name, name.rs)` の1配列で書ける

## 参照

- `crates/magia-cli/src/main.rs`
- `crates/magia-cli/tests/cli_integration.rs`
- `project-docs/magia/tech-selection-v0.1.md` §2.5
