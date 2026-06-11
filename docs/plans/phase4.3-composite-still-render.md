# Phase 4.3 — 静止画レンダ (Vue SSR + Bun に 1本化)

## 出典

- オーナー方針 (2026-06-11): 動的 Vue レンダラでレンダリングしたものを Vue SSR でファイル保存。Playwright スクショではなく **SVG ファイル直保存**。Rust SVG レンダラは廃止して **Vue 1本化**。`magia diff` / `magia ci` も SSR 経路に統一。Bun に最初から乗せる (Node.js は不採用)
- 親計画: Phase 4.0.9 (IR JSON エクスポート) 完了後に着手。同じ Vue コンポーネントが動的 UI と静止画の両方を担う
- 影響: Phase 3.1〜3.3 (Spell Diff + CI 統合) と 3.5 (ベルカ式) の SVG 出力経路を一斉移行

## 目的

`magia render` / `magia diff` / `magia ci` の **SVG 出力経路を Vue SSR + Bun に一本化** する。
Rust の SVG レンダラ (`midchilda.rs` / `belka.rs` の SVG 出力部分) を廃止し、**動的 UI と静止画で同じ Vue コンポーネントを使う** 状態にする。

POSD 観点で言えば「重複は負債」の正面突破。Vue 1本化により:
- 動的 UI と静止画で見た目が**コードレベルで一致**
- レイアウトや意匠の変更は Vue コンポーネント1箇所だけで完結
- Phase 4.6 のテーマ・diff overlay が静止画にも自動反映

## スコープ

### やること

- **Vue SSR スクリプト**:
  - `web/src/render/ssr.ts` — `@vue/server-renderer` で `<MagicCircle>` をレンダ、SVG 文字列を抽出
  - 入力: 標準入力で IR JSON、出力: 標準出力で SVG 文字列
  - 必要なオプション (focus, neighbors all/inner/none) は環境変数 or stdin の JSON ヘッダで渡す
- **Bun bundling**:
  - `bun build --compile --target=bun-darwin-arm64 ./web/src/render/ssr.ts --outfile magia-render` で single-file executable
  - macOS arm64 / linux x64 / windows x64 の cross-build (Bun 公式 cross-compile)
  - 出力バイナリは `target/release/magia-render` 等に配置
- **Rust 側統合**:
  - `magia-cli/src/render.rs`: 内部で `magia-render` を `std::process::Command` で spawn、stdin に IR JSON を流し込み、stdout を受け取る
  - `magia-render` のパス解決: 1) `MAGIA_RENDER_PATH` 環境変数 2) `magia` バイナリと同ディレクトリ 3) PATH 検索
  - サブプロセス失敗時のエラー伝達 (Vue 側のスタックを含める)
- **CLI 拡張**:
  - `magia render <FILE> --focus <fn> [--neighbors all|inner|none] [--out PATH]`
  - `--out` 未指定は stdout
  - `--format svg` (既定) / `--format png` (将来オプション、Bun の resvg-js などで SVG → PNG)
- **`magia diff` / `magia ci` の経路統一**:
  - Phase 3.1〜3.3 で書いた SVG 出力呼び出しを全て `magia-render` 経由に書き換え
  - Phase 3.2 の overlay 差分 (金ハロー・シアン・灰破線) は IR レベルで diff フラグを持たせ、Vue コンポーネントが反映
  - Phase 3.5 のベルカ式は IR JSON に `style: midchilda | belka` フィールドを追加、Vue 側で分岐
- **Rust SVG レンダラの削除** [break]:
  - `magia-core::render::midchilda` / `belka` の SVG 出力関数を削除
  - レイアウト計算部分 (座標決定) は IR 側に残す (Vue SSR が使う)
  - 関連テスト (golden SVG 比較) は Vue SSR 経路に書き換え
- **CI 統合**:
  - GitHub Actions に Bun セットアップ追加 (`oven-sh/setup-bun@v2`)
  - `bun build --compile` を CI で実行、生成バイナリを使って Rust 統合テストを回す
  - PR コメント (Phase 3.3) は Bun 経由で SVG 生成

### やらないこと (別計画)

- Playwright ヘッドレス撮影 (本計画では不採用。将来「ホバー中の中間状態」「アニメ途中フレーム」が必要になったらオプション追加)
- PNG/PDF 出力本体 (`--format png` のスケルトンだけ用意、実装は将来)
- Vue SSR で扱えない動的状態 (本計画は最終状態の SVG のみ)
- Tauri 等のデスクトップアプリ化

## 設計上の判断

### Vue SSR を採用、Playwright は採らない (オーナー判定)

| 観点 | Vue SSR | Playwright |
|---|---|---|
| 出力形式 | SVG (テキスト) | PNG / PDF (バイナリ) |
| `git diff` 可能性 | ◎ テキスト差分 | × |
| 起動コスト | ~50ms (Bun) | ~500ms (Chromium) |
| 依存重量 | 30〜50MB (Bun + bundle) | 数百 MB (Chromium) |
| 中間状態の撮影 | × | ◎ |

Phase 4.3 の目的 (README / PR コメント / CI 用静止画) は最終状態の SVG で完結 → **Vue SSR の圧勝**。

### Bun に最初から乗せる (オーナー指示)

Node.js + Bun 両対応の設計は二重保守になる。Bun 1本化で:
- `bun build --compile` の single-file executable がそのまま配布物に
- 起動が高速 (~50ms)
- TypeScript ネイティブ実行で `tsc` 不要
- Vite+ が Bun 互換 (4.0.5 で確認済の前提)

CI も Bun 1本化。

### Rust SVG レンダラの「レイアウト計算部分」だけ残す

Rust の責務は IR + レイアウト計算。SVG 出力は捨てる。
- `magia-core::layout` — 残す (IR に配置済座標を載せる役割)
- `magia-core::render::svg` — 削除 [break]
- `magia-core::render::midchilda` / `belka` — SVG 出力関数を削除、style 定数や IR 加工ロジックは残す (Vue 側が参照する形に整理)

### `magia-render` バイナリの配布形態

3案を検討:

| 案 | 説明 | 採用判断 |
|---|---|---|
| A | `magia-render` を独立バイナリで配布 | ◎ 起動高速、cargo install のみで完結しない (Bun が要る) |
| B | `magia` バイナリの中に `magia-render` を rust-embed で同梱、実行時に一時ディレクトリへ展開 | △ 起動が遅い、ディスク I/O 増 |
| C | `magia` バイナリが Node/Bun ランタイムを spawn し、SSR スクリプトを毎回実行 | × 起動が最も遅い、Bun が PATH に要る |

→ **採用 A**。`cargo install --path crates/magia-cli` の手順を「`bun install` も必要」と書き直し、CLAUDE.repo.md に明記。
配布パッケージ (将来) は `magia + magia-render` をセットでビルドした成果物を Releases に置く。

### IR JSON で diff/ci の意匠を表現する

Phase 3.2 の Spell Diff (金ハロー・シアン・灰破線):
- IR に `diff_status: Added | Removed | Modified | Unchanged` フィールド
- Vue コンポーネント側で `:class` 切替 → UnoCSS で意匠定義

Phase 3.3 の CI コメント:
- `magia ci` が `magia-render` 経由で diff SVG を生成
- PR コメントに `<img src="data:image/svg+xml;base64,...">` で埋め込み

Phase 3.5 のベルカ式:
- IR に `style: midchilda | belka` フィールド
- Vue 側で `<MidchildaCircle>` / `<BelkaCircle>` 分岐

これらは全て **Vue コンポーネント側で意匠を持つ** 設計に統一。Rust 側に意匠定数を二重に持たない。

## 受け入れ基準

- [ ] `magia render <FILE> --focus <fn>` で SVG 文字列が stdout に出る
- [ ] `--out <PATH>` でファイル書き出し
- [ ] 起動 → SVG 生成までの実時間が 200ms 以下 (Bun の起動が利く)
- [ ] 既存 fixtures (Phase 4.0.7 / 4.0.9 で揃えたもの) で 4.0.9 動的 UI と画素単位等価
- [ ] `magia diff` / `magia ci` の出力 SVG が `magia-render` 経由になっている (Rust SVG レンダラを呼ばない)
- [ ] Phase 3.1〜3.3 の既存テスト (golden SVG diff、CI コメント) が Vue SSR 経路で通る
- [ ] Phase 3.5 のベルカ式 SVG が Vue SSR で生成できる
- [ ] **Rust SVG レンダラ削除完了** (`grep -r "midchilda.*svg\|belka.*svg" crates/magia-core/src/render/` がヒットしない)
- [ ] バイナリサイズ: `magia` (Rust) は Phase 1.7 規模を維持、`magia-render` (Bun) は 50MB 以下
- [ ] CLAUDE.repo.md に Bun 前提と `bun build --compile` の手順記載

## 後続候補

- `--format png` 実装 (resvg-js or sharp で SVG → PNG)
- Playwright オプション追加 (中間状態撮影が必要になった場合)
- PDF 出力 (印刷用)
- `magia-render` のクロスコンパイル CI (macOS / Linux / Windows)

## 実装ステップ (粗粒度)

1. **Vue SSR スクリプト** `web/src/render/ssr.ts`: stdin で IR JSON 受領 → `renderToString` → SVG 抽出 → stdout
2. **Bun bundling**: `bun build --compile` 設定。CI で各 OS 向けにビルド
3. **Rust 統合**: `magia-cli/src/render.rs` で `magia-render` を spawn、エラー伝達
4. **`magia diff` / `magia ci` 移行**: Phase 3.1〜3.3 の SVG 出力呼び出しを `magia-render` 経由に書き換え
5. **意匠の Vue 側移植**:
   - Phase 3.2 Spell Diff (`diff_status` フィールド + Vue `:class`)
   - Phase 3.5 ベルカ式 (`style` フィールド + 分岐)
6. **Rust SVG レンダラ削除** [break]: midchilda / belka の SVG 出力関数 + 関連 golden テスト書き換え
7. **CI 統合**: GitHub Actions に Bun セットアップ、bundling、テスト実行
8. **CLAUDE.repo.md 更新**: Bun 前提、起動手順、`magia-render` 配布形態
9. **目視確認**: 既存 fixtures + diff fixture + ベルカ fixture で 4.0.9 動的 UI と並べてオーナー判定
10. **Stage 1 品質ゲート** + コーディング知見記録
11. Stage 2 レビュー + 指摘対応
12. 完了処理

## 想定リスク

- **Vue SSR で再現できない描画**: `<Transition>` などのアニメ系は SSR では最終状態だけが描かれる。Phase 4.3 の目的 (静止画) と整合するが、4.6 で diff overlay などをアニメ風に書いたとき SSR で何が出るか実装時確認
- **Bun の cross-compile 安定性**: `bun build --compile --target=` の各 OS 対応状況を実装時最新確認。1つでも未対応なら該当 OS は Rust SVG レンダラ残置の判断 (本計画の根本見直し)
- **`magia-render` のパス解決**: 環境変数 + 同ディレクトリ + PATH の優先順は CLAUDE.repo.md と CI で同期。テストで明示
- **既存 Phase 3.x の golden 大量更新**: 移行で 100+ の golden SVG が動く可能性。`UPDATE_GOLDENS=1` 環境変数で一括更新、レビューで diff 内容を確認
- **CI 時間増**: Bun bundling + 既存 cargo test の合算で CI が長くなる。並列ジョブ + Bun キャッシュで吸収
- **意匠の二重管理リスク回避**: Rust 側に色定数を残さないよう grep で確認。色は UnoCSS theme + Vue コンポーネントだけが持つ状態にする (Phase 4.0.5 の「`palette.rs` と一致」原則を Vue 側に逆転)
