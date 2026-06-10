# Knowhow Index

> 自動生成。`/addf-knowhow-index reindex` で再生成できる。

## Claude Code 設定・運用

| ファイル | 要約 | キーワード |
|---|---|---|
| [ADDF/claude-md-at-mention.md](ADDF/claude-md-at-mention.md) | CLAUDE.md の @FileName メンション展開の仕組みと使い分け | @展開, メンション, クオート, ネスト展開, CLAUDE.md, インライン展開, ファイル参照, ブートシーケンス |
| [ADDF/ignore-file-strategy.md](ADDF/ignore-file-strategy.md) | .gitignore / .claudeignore / .git/info/exclude の役割分けと運用戦略 | .gitignore, .claudeignore, .git/info/exclude, respectGitignore, settings.json, settings.local.json, Glob, Grep, ファイル除外 |

## Rust 実装

| ファイル | 要約 | キーワード |
|---|---|---|
| [rust-cargo-workspace-bootstrap.md](rust-cargo-workspace-bootstrap.md) | Cargo workspace 立ち上げの定型パターン (workspace.package 継承, lints 一括設定, publish 準備, doc_markdown 抑制) | cargo, workspace, clippy, pedantic, doc_markdown, MSRV, unsafe_code, crates.io, publish, Cargo.lock |
| [clap-cli-integration-pattern.md](clap-cli-integration-pattern.md) | clap 4 derive で CLI を統合するパターン (予約語フラグ, value_delimiter, エラー表示の責務分担, assert_cmd, 出力規約依存フィルタの相互参照) | clap, CLI, derive, --fn, value_delimiter, anyhow, assert_cmd, cargo_bin, 統合テスト, fixtures, exit code, thiserror |
| [mini-dsl-pattern.md](mini-dsl-pattern.md) | 小さな DSL を足すときの定型 (enum+FromStr の語彙一元化, 予約語の明示エラー, 分類と着色の分離, xxx_with の API 拡張, 行番号つきパースエラー) | DSL, FilterSpec, FromStr, enum, 予約語, パーサ, .magia, show, hide, conflicts_with, render_with |
| [minimal-dev-server-pattern.md](minimal-dev-server-pattern.md) | 同期スレッドモデルの最小 dev-server (tiny_http + SSE の live-reload, 親ディレクトリ監視 + デバウンス, エラー中の直前出力保持, --port 0 統合テスト定型, preview での実機確認) | dev-server, tiny_http, SSE, EventSource, notify, live-reload, デバウンス, mpsc, Read アダプタ, チャンク転送, port 0, assert_cmd, launch.json, preview |
| [deterministic-layout-pattern.md](deterministic-layout-pattern.md) | 決定論的レイアウトエンジンの定型 (BTreeMap 出力, ファン回転交差最小化, petgraph neighbors の順序, kurbo API, 決定論性テスト, 衝突回避4段構成, 半円展開制限, 局所修正による意匠保全, ベースライン回帰テスト) | layout, 決定論, BTreeMap, HashMap, petgraph, neighbors, kurbo, Rect, Point, Vec2, 交差最小化, hill-climbing, 放射状, 極座標, spec §6.1.4, 衝突回避, asin, 軌道, 緩和パス, フォールバック, 意匠保全, ベースライン |
| [svg-deterministic-rendering.md](svg-deterministic-rendering.md) | 決定論的 SVG 生成の定型 (raw string と色コードの衝突, 固定桁数値, XML エスケープ, insta 設定, qlmanage での目視確認, y 反転と反時計回り) | SVG, render, raw string, r##, needless_raw_string_hashes, insta, snapshot, qlmanage, XML エスケープ, textPath, kurbo::Arc, palette, 決定論, y反転 |
| [rust-ir-skeleton-pattern.md](rust-ir-skeleton-pattern.md) | spec 駆動 IR を Phase 1 から全フィールド揃えて一括定義するパターン (サブモジュール分割, serde(default) 一括, enum Default, struct_excessive_bools 局所 allow, doc 系 lint 対処, 必須テスト3種) | IR, intermediate representation, serde, serde(default), deny_unknown_fields, derivable_impls, struct_excessive_bools, doc_nested_refdefs, doc_lazy_continuation, newtype, SigilId, ModuleId, LayerData, EffectSet |
| [structural-diff-pattern.md](structural-diff-pattern.md) | ID 非依存の構造 diff の定型 (NodeKey 構造キーマッチング, Ord derive による決定論, 同キー貪欲対応, 共有メトリクス一本化, ライフタイム付きクロージャの関数化, JSON 契約の CLI 側明示構築) | diff, SpellDiff, NodeKey, 構造マッチング, SigilId 不安定, Ord, BTreeMap, 貪欲対応, MetricsDelta, metrics::measure, serde_json::json!, 情報隠蔽, 4象限, spec §9.2 |
| [syn-visitor-patterns.md](syn-visitor-patterns.md) | syn::Visit で AST から情報を集めるときの定型 (1関心1visitor, 走査統合, list/parse 規約一致, Allocator, lifetime 'ast 統一, thiserror 候補提示, RingBuilder 再帰展開・ID 順 sort・二重計上防止, call site 抽出・UseMap 近似解決・visit_macro の罠) | syn, syn::Visit, visit_expr_await, visit_expr_try, visit_expr_unsafe, visit_macro, ItemFn, ToTokens, SigilIdAllocator, thiserror, AST, lifetime, RingBuilder, AuxRing, 再帰展開, ParseContext, anchor, UseMap, use展開, call site, SummonGlyph, EffectSet, セグメント境界 |
