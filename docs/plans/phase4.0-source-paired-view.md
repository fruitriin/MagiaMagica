# Phase 4.0 — ソース連動ビュー (魔法陣とソースの並置)

## 出典

- オーナー要望 (2026-06-11): 「魔法陣とソースコードを1つのウィンドウで並列で見たい。ソースビューで関数を切り替えたら魔法陣も切り替わる」
- 関連: notes §1 (dev-server) で土台は完成済 (Phase 2.1〜2.4)。本計画はそこに「コードとの会話」のもう片側を足す

> notes のロードマップ上は Phase 4 = 多言語アダプタだが、フロントエンド充実はそれと並列の別ストリームとして本計画群 (Phase 4.x-fe) を立てる。多言語アダプタは Phase 5 系に繰り下げる想定。

## 目的

dev-server に「ソースペイン」を加え、**現在描画中の関数のソース**を魔法陣の隣に常時表示する。
同ファイル内の他関数 (impl メソッド含む) は脇に目次として並べ、クリックで魔法陣とソースを同時切替する。

「魔法陣に映る範囲が UI の中心」という原則 (オーナー指定) を守る:
- ソースペインに映るのは**ファイル全体ではなく現関数のソース**
- 同ファイルの他関数は二次情報として目次に出す。ファイル全体ナビ・モジュール階層は本計画では扱わない

## スコープ

### やること

- **CLI 破壊的変更**: `magia serve <FILE>` で起動 (--fn 廃止)。最初に表示する関数は URL `?fn=<name>` または「ファイル先頭の関数」既定
- **FunctionIndex 抽出**: `magia-core` にファイル全体スキャン API を追加。各関数の (name, impl/trait 文脈, 行範囲) を返す
- **ソースプリレンダ**: `syntect` で関数本体を SH 付き HTML スニペットに事前変換。サーバ側で生成し /state に同梱
- **HTTP/SSE 拡張**:
  - `GET /state` を「ファイルメタ + 関数一覧 + 現在選択の魔法陣+ソース」を返す形に拡張
  - `GET /spell/<fn>` で関数単位の (svg, source_html, signature) を返す
  - `?fn=<name>` で初期選択を指定。クライアントは history.replaceState で同期
- **フロント (素 JS 継続)**:
  - 左ペイン = 関数シグネチャ + SH 済みソース、右ペイン = 魔法陣 SVG
  - 上部 or 左上に関数一覧 (impl ブロック単位でグループ化、現選択をハイライト)
  - クリックで `?fn=<name>` 更新 → /spell/<fn> 取得 → 両ペイン差し替え
  - ファイル保存時の SSE 通知では FunctionIndex を比較し、削除関数の選択リセット / 新規関数の追加を反映
- 解析エラー時はソースペインに該当関数のエラー行をハイライト、魔法陣ペインは直前の図を保持 (会話を切らない原則継続)
- 統合テスト: FunctionIndex 抽出 / `magia serve` 起動 → /state /spell/<fn> 取得 / 関数追加削除の SSE 反映 / `?fn=` 初期化 / 未知 fn → 404

### やらないこと (別計画)

- 複数ファイル/モジュール階層ナビ (Phase 4.1 で検討)
- 呼び出し先・呼び出し元ジャンプ (Phase 3.4 のデータフロー IR を使う別 Phase: Phase 4.2 候補)
- ソースペインからの編集 (本計画は read-only)
- Vite/React 等のフロントエンドビルド (素 JS 継続。複雑さが閾値超えたら別 Phase で検討)
- ベルカ式との同時表示 (Phase 3.5 で実装される `?style=` トグルとは独立。本計画では style 1つだけ表示)

## 設計上の判断

### CLI 互換性 — `--fn` を廃止する

CLAUDE.repo.md の「v1.0 前は劣った設計を温存しない」方針に従い、`--fn` は廃止する。
- ソース連動ビューの世界では「最初に映す関数」はクライアント側の状態 (URL `?fn=`) であり CLI 引数ではない
- 残すと CLI / URL の二重ソース・オブ・トゥルースになり、後者が勝つので CLI の意義が消える
- 既存ユーザは現状オーナー1人。移行コストは無視できる

### FunctionIndex は magia-core に置く

`magia serve` だけでなく将来の `magia diff` (Phase 3.2) や `magia ci` (Phase 3.3) でも「ファイル全体の関数一覧」は使う。
core に集約し、CLI 各サブコマンドは index 経由で関数を引く設計に統一する。

### ソースプリレンダはサーバ側で

クライアント側で highlight.js / Prism を動かすより、サーバ側で `syntect` を一度だけ走らせる方が:
- バンドルサイズが減る (素 JS 継続の前提を守れる)
- IR と同じ Rust エコシステムで完結する (syn と同じ世界)
- ファイル監視の再パース時にまとめて再生成できる

`syntect` の Rust シンタックス + GitHub light テーマを既定。テーマは Phase 4.x で切替可能化。

### ペインレイアウト — 上下ではなく左右

魔法陣は正方形に近い、ソースは縦長になりがち。**横並び**にして:
- 左 30〜45%: ソース (max-width で読みやすい幅にクランプ)
- 右 55〜70%: 魔法陣 (現状の単独表示と同じ感覚)
- 関数一覧は左ペイン上部に collapsible で配置 (impl 単位でグループ)

レスポンシブで狭いウィンドウは縦並びにフォールバック。

### 「中心」原則の遵守

ファイル全体のソースは出さない。出すのは現関数 (impl ブロックなら ⌈impl Foo { ... ⌉ の囲い + 当該 fn のみ) のソース。
関数一覧は「サムネイル的に小さく」「現関数は強調」「他関数はワンクリックで切り替えられる」状態に留める。

これにより、画面の主役 = 魔法陣に映る関数、副次 = 隣接関数の存在感、という階層が UI に出る。

## 受け入れ基準

- [ ] `magia serve <FILE>` でブラウザに「ソース | 魔法陣」が並んで表示される (preview スクショで実機確認)
- [ ] 関数一覧の項目をクリックすると魔法陣とソースが同時に切替わる (URL `?fn=` 同期)
- [ ] `?fn=<name>` で初期表示関数を指定できる。未知の `?fn=` は先頭関数にフォールバック + 注意表示
- [ ] ファイル保存時の SSE 更新で関数一覧と現関数のソース・魔法陣が同期して再描画される
- [ ] 構文エラー中はソースペインにエラー行表示 + 魔法陣ペインは直前の図保持
- [ ] `--fn` フラグは削除されている (`magia serve --fn ...` はエラー)
- [ ] `cargo test --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` 通過
- [ ] 既存 fixtures (medium_render_doc, write_document, dense_dispatch, write_control_flow) の各関数を切り替えて目視確認

## 後続候補

- **Phase 4.1**: ファイル横断ナビ (workspace 内の `*.rs` を再帰スキャン、ファイルツリー追加)
- **Phase 4.2**: 呼び出しジャンプ (Phase 3.4 データフロー IR を流用、関数呼び出し → 呼び出し先関数へワンクリック移動)
- **Phase 4.3**: テーマ切替 (syntect テーマ + 魔法陣カラーパレットを連動)
- **Phase 4.4**: 差分ビュー連携 (Phase 3.2 Spell Diff を「同関数の before/after」と同時に「同ファイルの関数間」両軸で見られるよう拡張)

## 実装ステップ (粒度)

**実装フェーズ**

1. **magia-core**: `FunctionIndex` 型と `extract_functions(file: &str) -> Vec<FunctionEntry>` を追加。entry は (name, impl_context: Option<String>, line_range, signature_text)
2. **magia-core**: `syntect` 依存追加 + `highlight_rust(snippet) -> String (HTML)`。テーマは GitHub light 固定
3. **magia-cli/serve**: ルーティング拡張 — `/state` (関数一覧 + 既定/選択関数), `/spell/<fn>` (個別取得), `/events` 既存 SSE
4. **magia-cli/serve**: `--fn` 削除。`magia serve <FILE>` のみ受け入れる。エラー時の clap message を分かりやすく
5. **フロント (HTML テンプレート)**: 左右ペイン構造、関数一覧 UI、`?fn=` 同期、ペインクラスは `magia-layout-paired` で CSS スコープ
6. **フロント (JS)**: `loadFn(name)` で /spell/<fn> 取得 → 両ペイン差替え。SSE で関数追加削除を反映 (現選択が消えたら先頭にフォールバック + status 表示)
7. **エラーハンドリング**: 構文エラー時の partial レンダ (前回の魔法陣 + 今回のエラー行)
8. **テスト**: FunctionIndex の golden (impl 内 fn / トップレベル fn / 同名 fn 重複)、/spell/<fn> の正常系/404、SSE での index 差分反映、`?fn=` URL 同期
9. **目視素材生成**: 既存 fixtures 3〜4 個で並列ビューの SVG/HTML を撮ってオーナー送付
10. **Stage 1 品質ゲート** (build / clippy / fmt / test / `.claude/tests/run-all.sh`) + コーディング知見記録

**品質検証フェーズ (Stage 2)**

11. レビュー (`addf-code-review-agent`) + コントリビューション検出 (`addf-contribution-agent`) + 指摘対応 → ゲート再実行

**完了処理**

12. 計画 memo、Feedback / TODO 更新、アーカイブ、コミット

## 想定リスク

- **syntect の依存重量**: 初回ビルドが伸びる可能性。`default-features = false` + 必要な syntax set のみで抑える
- **`?fn=` の name 衝突**: 同名 fn が impl 違いで複数あるケース。`impl_context` を含めた `?fn=Foo::bar` 形式にする (トップレベルは `?fn=bar`)
- **ファイル監視 + 関数選択の競合**: 編集中に「現関数の名前が変わった」場合は line_range の連続性で追跡を試み、追えなければ先頭関数にフォールバック (本計画では best effort)
