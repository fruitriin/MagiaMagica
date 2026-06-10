# 同期スレッドモデルの最小 dev-server パターン

> Phase 2.1 (magia serve) で確立。「保存したらブラウザが自動更新される」live-reload を
> async ランタイムなしで実装する定型。

## 発見した知見

- **tiny_http + SSE で live-reload が完結する**: tokio/axum 一式を入れなくてよい。
  SSE は `mpsc::Receiver` を包む `Read` アダプタ (`recv()` でブロックし
  `data: <n>\n\n` を返す) で実装でき、tiny_http の長さ不定 `Response` が
  チャンク転送してくれる。WebSocket よりも部品が圧倒的に少ない
  (spec が「WebSocket または SSE」を許すならまず SSE を検討する)
- **ファイル監視は親ディレクトリを非再帰監視 + ファイル名照合**:
  エディタの「テンポラリ書き込み → rename」保存は、ファイルを直接監視すると
  inode が変わって取り逃がす。デバウンスは「最後のイベントから 120ms 静まるまで
  `recv_timeout` で飲み込む」だけで足りる
- **エラー中も直前の正常出力を保持する状態設計**: `svg` と `error` を別フィールドに
  持ち、解析エラー時は `error` だけ差し替える。保存のたびに画面が白紙にならず、
  「コードとの会話」が切れない
- **統合テストの定型**:
  - `--port 0` (空きポート自動割当) で実バイナリを起動し、stdout の起動 URL 行から
    ポートを抽出する。`println!` はパイプ接続だとブロックバッファされるため
    **起動行の直後に明示 flush** が必須
  - HTTP クライアント依存を増やさず、生 `TcpStream` に `GET ... HTTP/1.0` を書いて
    `read_to_string` で全文を読む (HTTP/1.0 なのでサーバが接続を閉じる)
  - 子プロセスは `Drop` で `kill()` + `wait()` するガード構造体に包む
    (テスト失敗時のプロセスリークを防ぐ)
  - 状態変化は `/state` の `version` フィールドをポーリングして待つ (上限つき)
- **ブラウザ実機確認を AI 自身が行える**: `.claude/launch.json` にサーバ定義を書き、
  `preview_start` → `preview_screenshot` で表示を画像確認できる
  (Bash で起動したサーバとポートが衝突するので先に止めること)。
  `preview_eval` で DOM 操作 (checkbox の `dispatchEvent`) → computedStyle 検証まで
  E2E でできる。**リロード・URL 直叩きの動線も preview で必ず踏む**こと
- **ルーティングはパスのみで照合する** (Phase 2.2 で踏んだバグ): `request.url()` は
  クエリ込み (`/?layers=...`) なので、`url == "/"` の完全一致だと「状態を URL クエリに
  保存する機能」がリロードで 404 になる。`url.split('?').next()` で
  パスを切り出してから照合する。このバグは単体テストでは出ず、preview の
  リロード検証で発見された
- **クエリ同期 UI の定型**: 状態は `URLSearchParams` ↔ JS 変数の readQuery/writeQuery
  (`history.replaceState`) で双方向同期し、SSE の innerHTML 差し替え後に毎回 apply する
  (差し替えで `<g>` のインラインスタイルが消えるため)

## プロジェクトへの適用

- `crates/magia-cli/src/serve.rs` — `magia serve <FILE> --fn <NAME> [--port]`
- ルートは `/` (静的 HTML) / `/state` (JSON: svg, error, version) / `/events` (SSE) の3つ。
  HTML は完全静的にして初回も `/state` フェッチで描く (テンプレート埋め込み不要)
- Phase 2.2 のレイヤーパレットはこの HTML に載せる

## 注意点・制約

- SSE 接続はスレッドを1本占有する。ローカル開発ツールでは問題ないが、
  多接続を想定するなら async 化を再検討する
- tiny_http の `server_addr()` は `ListenAddr` enum (IP/Unix)。IP しか bind しないなら
  Unix 側は防御的に処理する
- `notify` の監視エラーは「変化したかもしれない」側に倒して再レンダリングする

## 参照

- `crates/magia-cli/src/serve.rs`
- `crates/magia-cli/tests/serve_integration.rs`
- `project-docs/magia/spec-v0.2.md` §7
