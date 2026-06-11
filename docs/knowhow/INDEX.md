# Knowhow Index

> 自動生成。`/addf-knowhow-index reindex` で再生成できる。

## Claude Code 設定・運用

| ファイル | 要約 | キーワード |
|---|---|---|
| [ADDF/claude-md-at-mention.md](ADDF/claude-md-at-mention.md) | CLAUDE.md の @FileName メンション展開の仕組みと使い分け | @展開, メンション, クオート, ネスト展開, CLAUDE.md, インライン展開, ファイル参照, ブートシーケンス |
| [ADDF/ignore-file-strategy.md](ADDF/ignore-file-strategy.md) | .gitignore / .claudeignore / .git/info/exclude の役割分けと運用戦略 | .gitignore, .claudeignore, .git/info/exclude, respectGitignore, settings.json, settings.local.json, Glob, Grep, ファイル除外 |

## プロセス・計画運用

| ファイル | 要約 | キーワード |
|---|---|---|
| [milestone-gated-ui-plan.md](milestone-gated-ui-plan.md) | 目視判定ゲート付き UI 計画の運用 (ゲート=同期点、追加要望の仕分けと即計画化、URL 全状態同期で判定素材が安価、基盤入れ替え=潜在バグの炙り出し装置) | 判定ゲート, マイルストーン, 目視判定, オーナー判定, 素材送付, スコープ分離, URL 同期, ヘッドレス Chrome, スクショ, 基盤入れ替え, 移植, 機能等価, 潜在バグ, 炙り出し, golden, 受け入れ基準 |

## フロントエンド (Web)

| ファイル | 要約 | キーワード |
|---|---|---|
| [viteplus-bun-frontend-bootstrap.md](viteplus-bun-frontend-bootstrap.md) | Vite+ (vp) + Bun でのフロントエンド基盤立ち上げ (bunx の bin 解決の罠, vp create は vanilla-ts 生成 + PATH に vp 必須, 同一採番ピン, ::1 listen, PORT 差し替え, vanilla→Vue 化最小セット, vp check 一括窓口) | Vite+, vite-plus, vp, vp create, vp check, bunx, Bun, oxfmt, oxlint, Vue 3, vue-tsc, @vitejs/plugin-vue, UnoCSS, preset-attributify, Pinia, vue-router, consistent-type-definitions, erasableSyntaxOnly, strict, env.d.ts, *.vue shim, IPv6, localhost, autoPort, PORT, proxy, overrides, vite-plus-core |

## Rust 実装

| ファイル | 要約 | キーワード |
|---|---|---|
| [multi-style-renderer-pattern.md](multi-style-renderer-pattern.md) | 第2のレンダリング様式 (同じ IR の別射影) を足す定型 (RenderStyle 語彙一元化, 未対応オプションの明示エラー, 射影モデルと描画の分離, anchor 実行順走査, 暗黙の戻り値の末尾フラグ, phyllotaxis 散布, serve の全様式同梱 + 表示切替) | RenderStyle, --style, belka, 様式, 射影, 三角力場, 極, フロー集計, 実行順, anchor, phyllotaxis, 黄金角, radialGradient, ?style=, トグル, SELECTABLE, FromStr |
| [rust-cargo-workspace-bootstrap.md](rust-cargo-workspace-bootstrap.md) | Cargo workspace 立ち上げの定型パターン (workspace.package 継承, lints 一括設定, publish 準備, doc_markdown 抑制) | cargo, workspace, clippy, pedantic, doc_markdown, MSRV, unsafe_code, crates.io, publish, Cargo.lock |
| [clap-cli-integration-pattern.md](clap-cli-integration-pattern.md) | clap 4 derive で CLI を統合するパターン (予約語フラグ, value_delimiter, エラー表示の責務分担, assert_cmd, 出力規約依存フィルタの相互参照) | clap, CLI, derive, --fn, value_delimiter, anyhow, assert_cmd, cargo_bin, 統合テスト, fixtures, exit code, thiserror |
| [mini-dsl-pattern.md](mini-dsl-pattern.md) | 小さな DSL を足すときの定型 (enum+FromStr の語彙一元化, 予約語の明示エラー, 分類と着色の分離, xxx_with の API 拡張, 行番号つきパースエラー) | DSL, FilterSpec, FromStr, enum, 予約語, パーサ, .magia, show, hide, conflicts_with, render_with |
| [minimal-dev-server-pattern.md](minimal-dev-server-pattern.md) | 同期スレッドモデルの最小 dev-server (tiny_http + SSE の live-reload, SSE はチャンク経路禁止 → into_writer + flush, 親ディレクトリ監視 + デバウンス, エラー中の直前出力保持, --port 0 統合テスト定型, preview での実機確認) | dev-server, tiny_http, SSE, EventSource, notify, live-reload, デバウンス, mpsc, into_writer, stream_sse, flush, BufWriter, chunked_transfer, 二重バッファ, Connection close, port 0, assert_cmd, launch.json, preview, /spell, オンデマンド, GoodSnapshot, percent_decode, syntect, OnceLock, 意味述語 |
| [deterministic-layout-pattern.md](deterministic-layout-pattern.md) | 決定論的レイアウトエンジンの定型 (BTreeMap 出力, ファン回転交差最小化, petgraph neighbors の順序, kurbo API, 決定論性テスト, 衝突回避4段構成, 半円展開制限, 局所修正による意匠保全, ベースライン回帰テスト) | layout, 決定論, BTreeMap, HashMap, petgraph, neighbors, kurbo, Rect, Point, Vec2, 交差最小化, hill-climbing, 放射状, 極座標, spec §6.1.4, 衝突回避, asin, 軌道, 緩和パス, フォールバック, 意匠保全, ベースライン |
| [svg-deterministic-rendering.md](svg-deterministic-rendering.md) | 決定論的 SVG 生成の定型 (raw string と色コードの衝突, 固定桁数値, XML エスケープ, insta 設定, qlmanage での目視確認, y 反転と反時計回り) | SVG, render, raw string, r##, needless_raw_string_hashes, insta, snapshot, qlmanage, XML エスケープ, textPath, kurbo::Arc, palette, 決定論, y反転 |
| [rust-ir-skeleton-pattern.md](rust-ir-skeleton-pattern.md) | spec 駆動 IR を Phase 1 から全フィールド揃えて一括定義するパターン (サブモジュール分割, serde(default) 一括, enum Default, struct_excessive_bools 局所 allow, doc 系 lint 対処, 必須テスト3種, 新 Edge 種別追加時の kind フィルタ掃き) | IR, intermediate representation, serde, serde(default), deny_unknown_fields, derivable_impls, struct_excessive_bools, doc_nested_refdefs, doc_lazy_continuation, newtype, SigilId, ModuleId, LayerData, EffectSet, EdgeKind, EdgeLayerData, kind フィルタ, Edge ソート |
| [git-ci-integration-pattern.md](git-ci-integration-pattern.md) | CLI の git 連携と CI しきい値の定型 (git サブプロセスの隔離, ls-files によるパス正規化, 入力2系統の入口正規化, unsafe 最小しきい値, changed 列挙, 薄い YAML + ローカル再現スクリプト, sticky comment, git init 統合テスト) | git, gitio, git show, ls-files, git diff --name-only, CI, GitHub Actions, workflow, sticky comment, gh api, fail-on-new-unsafe, unsafe_ops, changed, DiffSources, conflicts_with, fetch-depth, jq, init_git_fixture, spec §9 |
| [structural-diff-pattern.md](structural-diff-pattern.md) | ID 非依存の構造 diff の定型 (NodeKey 構造キーマッチング, Ord derive による決定論, 同キー貪欲対応, 共有メトリクス一本化, ライフタイム付きクロージャの関数化, JSON 契約の CLI 側明示構築, overlay-diff 強調チャネル, ゴーストと viewBox 拡張, ハロー意匠) | diff, SpellDiff, NodeKey, 構造マッチング, SigilId 不安定, Ord, BTreeMap, 貪欲対応, MetricsDelta, metrics::measure, serde_json::json!, 情報隠蔽, 4象限, spec §9.2, overlay-diff, highlight, ゴースト, ハロー, viewBox, Rect::union, render_diff, 目視素材, git show |
| [syn-visitor-patterns.md](syn-visitor-patterns.md) | syn::Visit で AST から情報を集めるときの定型 (1関心1visitor, 走査統合, list/parse 規約一致, Allocator, lifetime 'ast 統一, thiserror 候補提示, RingBuilder 再帰展開・ID 順 sort・二重計上防止, call site 抽出・UseMap 近似解決・visit_macro の罠, span→原文切り出し・列規約) | syn, syn::Visit, visit_expr_await, visit_expr_try, visit_expr_unsafe, visit_macro, ItemFn, ToTokens, SigilIdAllocator, thiserror, AST, lifetime, RingBuilder, AuxRing, 再帰展開, ParseContext, anchor, UseMap, use展開, call site, SummonGlyph, EffectSet, セグメント境界, span, LineColumn, 列情報, call_excerpt, 原文切り出し, dedent, exclusive, dataflow, def/use, ScopeTracker, スコープ追跡, シャドーイング, 再代入, クロージャ除外, seeds, FunctionIndex, ImplItemFn, qualified, impl メソッド, walker 統一 |
