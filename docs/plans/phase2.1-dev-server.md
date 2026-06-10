# Phase 2.1 — dev-server 最小実装 (静止画から会話へ)

## 出典

- `project-docs/magia/phase2plus-notes-v0.1.md` §1 (dev-server 化とリアルタイム可視化)
- `project-docs/magia/spec-v0.1.md` §7 (dev-server とレイアウト)、Phase 2.0 で v0.2 に正式化予定

## 目的

`magia serve <FILE> --fn <NAME>` で常駐サーバを起動し、ブラウザに魔法陣を表示、
**ファイル保存のたびに自動で再描画**されるようにする。notes §1.1 の「コードとの会話」体験の最小形。

## スコープ

### やること

- `magia serve <FILE> --fn <NAME> [--port PORT]` サブコマンド
- HTTP サーバ: SVG をインライン埋め込みした HTML 1枚を配信 (テンプレートは最小)
- ファイル監視: `notify` クレートで対象ファイルの変更を検知 → parse → layout → render を再実行
- 更新通知: WebSocket (または SSE) でブラウザへ再描画を push。フロントは SVG 要素を差し替えるだけ
- 解析エラー時はエラーメッセージを画面に表示し、直前の正常な図を保持する (会話を切らない)
- 統合テスト: サーバ起動 → HTTP GET で SVG を含む HTML が返る / ファイル書き換え → 更新が配信される

### やらないこと (notes §1.2 の3層は Phase 1 規模では不要)

- 永続キャッシュ層 (sled/RocksDB) — 単一関数の再解析は数ミリ秒で済む
- インクリメンタル再計算層 (Salsa・逆依存グラフ) — 多関数対応 (Phase 2 後半以降) で導入判断
- JSON Patch による差分配信 — SVG 全置換で十分。60fps アニメーションは将来
- 複数関数・プロジェクト全体の監視

## 設計上の判断

- web フレームワークは軽量なものを実装時に選定する (`axum` を第一候補、依存の重さ次第で `tiny_http` + `tungstenite` に切替)。**実装時に `cargo add --dry-run` で実値確認** (Feedback の改善アクション)
- 新クレートは作らず `magia-cli` 内のモジュール (`serve.rs`) とする。dev-server が育ったら (Phase 2 後半) `magia-server` クレートへ分離を判断する
- HTML テンプレートは Phase 2.2 (レイヤー UI) の土台になるため、`<g class="layer-*">` を CSS で制御できる構造を最初から保つ

## 受け入れ基準

- [ ] `magia serve fixtures/io_print.rs --fn io_print` でブラウザに魔法陣が表示される
- [ ] ファイルを保存すると 1 秒以内に表示が更新される (手動確認 + 統合テスト)
- [ ] 構文エラー中も画面が白紙にならない (エラー表示 + 直前の図を保持)
- [ ] `cargo test --workspace` / clippy 警告0

## 後続

- Phase 2.2 でこの HTML にレイヤーパレット UI を載せる
