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
| [rust-ir-skeleton-pattern.md](rust-ir-skeleton-pattern.md) | spec 駆動 IR を Phase 1 から全フィールド揃えて一括定義するパターン (サブモジュール分割, serde(default) 一括, enum Default, struct_excessive_bools 局所 allow, 必須テスト3種) | IR, intermediate representation, serde, serde(default), deny_unknown_fields, derivable_impls, struct_excessive_bools, doc_nested_refdefs, newtype, SigilId, ModuleId, LayerData, EffectSet |
| [syn-visitor-patterns.md](syn-visitor-patterns.md) | syn::Visit で AST から情報を集めるときの定型 (1関心1visitor, 走査統合, list/parse 規約一致, Allocator, lifetime 'ast 統一, thiserror 候補提示, RingBuilder 再帰展開・ID 順 sort・二重計上防止) | syn, syn::Visit, visit_expr_await, visit_expr_try, visit_expr_unsafe, ItemFn, ToTokens, SigilIdAllocator, thiserror, AST, lifetime, RingBuilder, AuxRing, 再帰展開, ParseContext, anchor |
