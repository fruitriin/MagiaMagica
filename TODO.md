# TODO

`docs/plans/` の完了状態・優先度をトラックする。
`docs/plans/` と TODO が一致しなければ TODO を編集する。

## 現在のフェーズ: Phase 4 — フロントエンド充実

**Phase 3 (3.0〜3.5) は全完了 (2026-06-11)** — Spell Diff 系譜 + CI 統合 + ベルカ式。
残存する判定待ち: Phase 1.8 / 3.2 / 3.5 の意匠判定素材 (オーナー送付済み)。
**「Vue 1本化」コア系譜 (4.0.5 → 4.0.7 → 4.0.9) が完成 (2026-06-11)** — 4.1 / 4.3 の前提が揃った。
4.0.6 前半 (凡例) も完了。**4.1 (ピン中心ビュー + 召喚印インスペクタ + ホバープレビュー群) も完了 (判定合格 2026-06-12)**。
**4.2 (近接度本実装)・4.3 (Vue SSR 一本化、[break] Rust SVG レンダラ削除) も完了 (判定合格 2026-06-12)**。
**4.3.7 (live diff)・4.4.5 (エントリポイント切替) も完了 (2026-06-12)**。**4.4 (呼び出しジャンプ) も完了 (2026-06-12)**。**4.5 M1 (俯瞰) も完了 (2026-06-12)** — Rust/土台系の未着手は尽きた。残りは意匠系 (4.0.6 後半 / 4.3.5 / 4.6 / 4.7 — 後回し方針、オーナー指示 2026-06-12) と 4.5 M2+ (方針確認待ち)。
Phase 3 振り返り (二式並置ビュー・レイヤー差分分解・knowhow 一括昇格 PR — Rust 系と
Web 系で PR を分ける、計8件 + addf-knowhow 追記4例) はオーナー判定が出揃った節目に実施する。

## バックログ

| 優先度 | Phase | 計画ファイル | 状態 |
|---|---|---|---|
| 1 | 3.0 | [docs/plans/phase3.0-spec-v0.3.md](docs/plans/phase3.0-spec-v0.3.md) | 完了 |
| 2 | 3.1 | [docs/plans/phase3.1-diff-engine.md](docs/plans/phase3.1-diff-engine.md) | 完了 |
| 3 | 3.2 | [docs/plans/phase3.2-spell-diff.md](docs/plans/phase3.2-spell-diff.md) | 完了（意匠判定待ち） |
| 4 | 3.3 | [docs/plans/phase3.3-ci-integration.md](docs/plans/phase3.3-ci-integration.md) | 完了 |
| 5 | 3.4 | [docs/plans/phase3.4-dataflow-ir.md](docs/plans/phase3.4-dataflow-ir.md) | 完了 |
| 6 | 3.5 | [docs/plans/phase3.5-belka-style.md](docs/plans/phase3.5-belka-style.md) | 完了（意匠判定待ち） |
| 7 | 4.0.5 | [docs/plans/phase4.0.5-frontend-foundation.md](docs/plans/phase4.0.5-frontend-foundation.md) | **完了**（M1〜M5 全判定合格、2026-06-11） |
| 8 | 4.0 | [docs/plans/phase4.0-source-paired-view.md](docs/plans/phase4.0-source-paired-view.md) | 完了（サーバ側 API まで。UI は 4.0.5 に移管） |
| 9 | 4.0.7 | [docs/plans/phase4.0.7-svg-to-vue-schema.md](docs/plans/phase4.0.7-svg-to-vue-schema.md) | **完了**（判定合格 2026-06-11。v-html 撤去、Schema + コンポーネントツリー） |
| 10 | 4.0.9 | [docs/plans/phase4.0.9-ir-json-export-and-vue-builder.md](docs/plans/phase4.0.9-ir-json-export-and-vue-builder.md) | **完了**（判定合格 2026-06-11。SVG パーサ削除、IR 直結） |
| 11 | 4.0.6 | [docs/plans/phase4.0.6-circle-affordances.md](docs/plans/phase4.0.6-circle-affordances.md) | **前半完了**（凡例パネル、判定合格 2026-06-11）/ 後半 (入口・回転サイン + 補助陣ラベル) は **4.3 の後** |
| 12 | 4.1 | [docs/plans/phase4.1-pinned-focus-view.md](docs/plans/phase4.1-pinned-focus-view.md) | **完了**（判定合格 2026-06-12。追加要望4連鎖も実装: 召喚印インスペクタ / 呼び出し式 / ホバー2層 / 補助リング条件） |
| 13 | 4.2 | [docs/plans/phase4.2-proximity-model.md](docs/plans/phase4.2-proximity-model.md) | **完了**（判定合格 2026-06-12。呼び出し関係 + 3層リング。call_graph 新設） |
| 14 | 4.3 | [docs/plans/phase4.3-composite-still-render.md](docs/plans/phase4.3-composite-still-render.md) | **完了**（判定合格 2026-06-12。SVG 出力を Vue SSR に一本化、Rust SVG レンダラ削除 [break] -1879行） |
| 15 | 4.3.5 | [docs/plans/phase4.3.5-junction-symbols.md](docs/plans/phase4.3.5-junction-symbols.md) | 未着手（イメージ感のみ。**4.3 後** — 分岐・ループ記号を補助リング中央から結節点バッジへ。オーナー要望 2026-06-12） |
| 15.5 | 4.3.7 | [docs/plans/phase4.3.7-diff-on-web.md](docs/plans/phase4.3.7-diff-on-web.md) | **完了**（判定合格 2026-06-12 — 実機で「Diff出てる!すごーい」。?diff=<rev> + live diff） |
| 16 | 4.4 | [docs/plans/phase4.4-call-jump.md](docs/plans/phase4.4-call-jump.md) | **完了**（2026-06-12。チップに呼び出しの向きマーク + 記号⇆チップ双方向リンク強調。クリック遷移は 4.1 インスペクタで先取り済み） |
| 16.5 | 4.4.5 | [docs/plans/phase4.4.5-entrypoint-on-web.md](docs/plans/phase4.4.5-entrypoint-on-web.md) | **完了**（2026-06-12。ヘッダのドロップダウンで切替 + ?file= 同期 + /files API。素材送付済み） |
| 17 | 4.5 | [docs/plans/phase4.5-workspace-overview.md](docs/plans/phase4.5-workspace-overview.md) | **M1 完了**（2026-06-12。俯瞰トグル + ファイルカード + ズームイン + ?scope=。M2+ 候補は計画 memo） |
| 18 | 4.6 | [docs/plans/phase4.6-theme-and-diff-overlay.md](docs/plans/phase4.6-theme-and-diff-overlay.md) | 未着手（イメージ感のみ） |
| 19 | 4.7 | [docs/plans/phase4.7-mana-circuit-animation.md](docs/plans/phase4.7-mana-circuit-animation.md) | 未着手（イメージ感のみ。**魔力回路アニメーション** — 実行順を流れる点P、非同期で分裂。オーナー要望 2026-06-12） |

依存関係:
- 3.0 (仕様化) は全ての前提。3.1 → 3.2 → 3.3 が Spell Diff の系譜
- 3.4 → 3.5 がベルカ式の系譜 (3.1〜3.3 と独立して進められる)
- 3.4 は EdgeLayerData の破壊的再設計 (spec v0.2 §4.3 の既定方針) を含む
- **4.0.5 が Phase 4 全体の前提**。Vue 3 + Vite+ + UnoCSS + Bun 基盤を立ち上げ、Phase 2.x の inline HTML/JS を Vue 化する。4.0 以降は本基盤の上に構築
- **4.0.5 → 4.0.7 → 4.0.9 が「Vue 1本化」のコア系譜**。境界スキーマ `MagicCircleSchema` を 4.0.5 M2 で先置き → 4.0.7 で SVG パーサで埋める (案1) → 4.0.9 で IR ビルダに差し替え (案2)。Vue コンポーネント群は無修正で流用
- **4.0.9 完了時点で Rust SVG レンダラに deprecate マーク**。**4.3 で削除** (`[break]`)
- **4.1 → 4.2 が「ピン中心ビュー」のコア系譜**。4.0.9 前提で書き直し済 (`MagicCircleSchema` + `<Transition>` で宣言的に書く)
- **4.3 は Vue SSR + Bun に全面改稿**。`magia render` / `magia diff` / `magia ci` の SVG 出力を Vue SSR 経路に統一、Rust SVG レンダラ削除 (`[break]`)。Phase 3.1〜3.3 / 3.5 も同経路に移行
- **4.0.6 (読み方アフォーダンス) はオーナー要望 (2026-06-11、M3 判定時)**。凡例パネルは 4.0.5 のみに依存し先行可。**入口サイン・補助陣ラベル (SVG 描画系) は 4.3 (Vue SSR 一本化) の後に実施** (オーナー確定 — Rust レンダラには足さない)
- **4.3.5 (結節点シンボル) はオーナー要望 (2026-06-12、4.1 サイクル中)**。4.3 後に 4.0.6 後半と同タイミング群で実施
- **4.4 (呼び出しジャンプ) は 4.1 + 4.2 + Phase 3.4 データフロー IR に依存**
- **4.4.5 (エントリポイント切替) は 4.5 から先行切り出し** (オーナー要望 2026-06-12)。単独実施可、4.5 はこれを土台にできる
- **4.5 (ワークスペース俯瞰) は 4.0〜4.4 完了後に詳細精緻化**
- **4.6 (テーマ + Spell Diff overlay) は Phase 3.2 / 3.5 完了済成果物を 4.x 上に重ねる**。うち **web 上の diff overlay は 4.3.7 として先行切り出し済** (4.3 M4 で前提が揃ったため。4.6 にはテーマ・ベルカ同時表示・URL 軸統合が残る)
- 4.4〜4.6 は計画書時点では **イメージ感のみ**、実装着手時に内容を精緻化する (オーナー方針 2026-06-11)
- **4.7 (魔力回路アニメーション) はオーナー要望 (2026-06-12)**。土台 (実行順走査・幾何 IR・ConcurrencyInfo・Vue 一本化) は全て完成済み。**4.0.6 後半 (入口サイン = 点Pの出発点) と強いシナジー** — 同サイクルか連続サイクル推奨
- notes の Phase 4 (多言語アダプタ) は **Phase 5 系に繰り下げ**

---

## アーカイブ

| Phase | 計画ファイル | 状態 |
|---|---|---|
| 1.0 | [docs/plans/phase1.0-workspace-bootstrap.md](docs/plans/phase1.0-workspace-bootstrap.md) | 完了 |
| 1.1 | [docs/plans/phase1.1-ir-skeleton.md](docs/plans/phase1.1-ir-skeleton.md) | 完了 |
| 1.2 | [docs/plans/phase1.2-syn-to-ir.md](docs/plans/phase1.2-syn-to-ir.md) | 完了 |
| 1.3 | [docs/plans/phase1.3-aux-rings.md](docs/plans/phase1.3-aux-rings.md) | 完了 |
| 1.4 | [docs/plans/phase1.4-summon-effects.md](docs/plans/phase1.4-summon-effects.md) | 完了 |
| 1.5 | [docs/plans/phase1.5-layout-engine.md](docs/plans/phase1.5-layout-engine.md) | 完了 |
| 1.6 | [docs/plans/phase1.6-svg-renderer-midchilda.md](docs/plans/phase1.6-svg-renderer-midchilda.md) | 完了（意匠は部分合格・1.8 で再判定） |
| 1.7 | [docs/plans/phase1.7-cli-integration.md](docs/plans/phase1.7-cli-integration.md) | 完了 |
| 1.8 | [docs/plans/phase1.8-layout-collision-avoidance.md](docs/plans/phase1.8-layout-collision-avoidance.md) | 完了（意匠再判定待ち） |
| 2.0 | [docs/plans/phase2.0-spec-v0.2.md](docs/plans/phase2.0-spec-v0.2.md) | 完了 |
| 2.1 | [docs/plans/phase2.1-dev-server.md](docs/plans/phase2.1-dev-server.md) | 完了 |
| 2.2 | [docs/plans/phase2.2-layer-toggle.md](docs/plans/phase2.2-layer-toggle.md) | 完了 |
| 2.3 | [docs/plans/phase2.3-filter-dsl.md](docs/plans/phase2.3-filter-dsl.md) | 完了 |
| 2.4 | [docs/plans/phase2.4-transcript.md](docs/plans/phase2.4-transcript.md) | 完了 |
