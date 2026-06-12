# 同期スレッドモデルの最小 dev-server パターン

> Phase 2.1 (magia serve) で確立、Phase 4.0 (メタ + オンデマンドレンダへの再設計) で拡張、
> Phase 4.0.5 (SSE 配信バグの修正) で訂正。
> 「保存したらブラウザが自動更新される」live-reload を async ランタイムなしで実装する定型。

## 発見した知見

- **tiny_http + SSE で live-reload が完結する**: tokio/axum 一式を入れなくてよい。
  WebSocket よりも部品が圧倒的に少ない
  (spec が「WebSocket または SSE」を許すならまず SSE を検討する)
- **【Phase 4.0.5 で訂正】SSE に tiny_http の Response (チャンク転送) 経路を使ってはいけない**:
  当初は `mpsc::Receiver` を包む `Read` アダプタ + 長さ不定 `Response` で実装したが、
  この経路は `chunked_transfer::Encoder` (8KB) と `BufWriter` (1KB、client.rs の
  `with_capacity(1024, ..)`) の**二重バッファがどちらも flush されず** (response.rs
  「does not flush the writer」、`io::copy` は flush しない)、小さいイベントは
  クライアントへ永遠に届かない。Phase 2.1〜4.0 の間この状態で潜在していた
  (live-reload は「動いているように見えて」実際は配信されていなかった)。
  **正解は `request.into_writer()` で生 writer を取り、ステータス行 + ヘッダを自前で
  書き、イベントごとに `flush()` する** (serve.rs の `stream_sse`)。`Connection: close`
  を付けてその接続をイベント専用にする。`request.upgrade()` は `Connection: upgrade`
  ヘッダが付き、中継 proxy (vite dev 等) が WebSocket 扱いしかねないので使わない
- **SSE の統合テストは「ストリームが実際に届くこと」を読むテストにする**: inline HTML に
  `EventSource` の文字列があるかのような静的チェックでは配信バグを捕捉できない。
  生 `TcpStream` + `read_timeout` 付き `read_line` で「ヘッダ → 接続直後イベント →
  ファイル変更後の追加イベント」を実際に読む (タイムアウト = 滞留の再発)。
  serve_integration.rs の `sse_events_stream_immediately` が定型
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

## メタ + オンデマンドレンダへの再設計 (Phase 4.0)

「描画済み SVG を /state で配る」形から「メタ + 関数単位のオンデマンドレンダ」へ
再設計するときの定型:

- **サーバ状態は最小に**: 「直近の正常スナップショット (ソース全文 + 関数索引) +
  エラー (行付き)」だけを持つ。**どの関数を見ているかはクライアント (URL `?fn=`) の
  状態**であり、サーバは保持しない
- `/spell/<fn>` をオンデマンドレンダにすると、キャッシュ無効化・サーバ側の選択状態・
  全関数プリレンダが全部不要になる。これは**レンダラが決定論的だから成り立つ**
  (同じソースからは常に同じ SVG — リクエストごとの再レンダリングが冪等)
- エラー中は直近スナップショットから配信し続ける (会話を切らない原則の一般化:
  SVG 単体でなくスナップショット全体を保持する)
- **統合テストの述語は意味で書く**: 「version が進んだら」を述語にすると、
  truncate→write の中間状態を読んだ reload を最終状態と誤認してフレークする。
  「新関数が索引に現れたら」のような意味述語で待つ
- URL パスセグメント (`/spell/Foo%3A%3Abar`) には最小のパーセントデコードが要る
  (tiny_http はデコードしない)

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
- `crates/magia-cli/src/serve.rs` の `GoodSnapshot` / `spell_json` / `percent_decode`
- `crates/magia-cli/src/srcview.rs` — syntect は default-features を切り
  `default-syntaxes + default-themes + html + regex-fancy` に絞る (onig の C 依存回避)。
  SyntaxSet/Theme は OnceLock で初回のみロード


## live diff (`?diff=<REV>`) の設計 (Phase 4.3.7)

- 失敗 (rev 不正 / git 外 / before に関数なし) は **エラーでなく案内文 (`diff_note`)**
  に畳んで応答を壊さない — エラー中も図を出し続ける「会話を切らない」原則の diff 版
- before (git 内容) は**意図的にキャッシュしない**: 保存 → SSE → 再計算の経路が
  そのまま「書いている変更が即ハローに現れる」live diff になる
- `?diff=` の値は `-` 始まりを弾く (`git show --format=...` 等のオプション注入防御。
  Command::args 経由なのでシェル注入は元々ない)
- e2e は fixture ディレクトリを git init + 初期 commit して実 git で検証する —
  beforeEach の「既知内容に書き戻す」運用と合わせると「HEAD = 初期内容」が固定され、
  テスト内のファイル変更がそのまま差分になる
