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

- [x] `magia serve` でブラウザに魔法陣が表示される (preview スクリーンショットで実機確認)
- [x] ファイルを保存すると 1 秒以内に表示が更新される (統合テスト: version 増加 + SVG 変化)
- [x] 構文エラー中も画面が白紙にならない (エラー表示 + 直前の図を保持 + 復帰も検証)
- [x] `cargo test --workspace` (121本) / clippy 警告0

## 後続

- Phase 2.2 でこの HTML にレイヤーパレット UI を載せる

## 実装結果メモ (2026-06-11)

### 依存の実値 (計画の axum 第一候補から変更)

- **tiny_http 0.12 + notify 8.2 + SSE** を採用。tokio/axum 一式を持ち込まずに
  live-reload が完結する (SSE は mpsc::Receiver を包む Read アダプタで実装、
  tiny_http の長さ不定 Response がチャンク転送)。spec §7 の「WebSocket または SSE」の
  範囲内。多接続が必要になったら (Phase 2 後半の多関数対応) async 化を再評価

### 設計判断の確定

- ルートは `/` (完全静的 HTML、初回も /state フェッチで描画) / `/state` (JSON:
  svg・error・version) / `/events` (SSE) の3つ
- 監視は親ディレクトリ非再帰 + ファイル名照合 (エディタの rename 保存対応)、
  120ms デバウンス。監視エラーは「変化したかもしれない」側に倒す
- Mutex は poisoning 回復つきロック (`lock_or_recover`): 一部スレッドの
  パニック後も最後の正常状態で応答し続ける (クラッシュループ回避)
- SSE クライアントの掃除は publish 時 + 新規接続時の二系統 (リーク防止)

### レビュー対応 (Stage 2)

- 修正: Mutex poisoning 回復 (H-1) / SSE Sender リークの接続時掃除 (H-2) /
  テストのエラーメッセージ文言非依存化 (M-1) / fixture のテスト別ディレクトリ分離
  + 毎回クリーン作成 (M-2、並列実行時の notify イベント混入によるフレーク防止)
- 既知の制約 (ドキュメント化): リクエスト/SSE はスレッド占有 (M-4、ローカル
  dev ツールの同時接続数では問題ない)。launch.json は Claude preview 用の定義で
  VS Code の launch.json とは別物 (L-1)
- spec v0.3 への持ち越し: 監視エラー時挙動の明文化 (L-3) → Feedback に記録

### 動作確認

統合テスト3本 (配信 / 変更検知 / エラー保持・復帰) + preview スクリーンショットで
write_document の魔法陣表示を実機確認。`.claude/launch.json` に preview 定義を常設
