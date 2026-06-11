# Phase 4.0 — ソース連動ビュー (魔法陣とソースの並置)

## 出典

- オーナー要望 (2026-06-11): 「魔法陣とソースコードを1つのウィンドウで並列で見たい。ソースビューで関数を切り替えたら魔法陣も切り替わる」
- 関連: notes §1 (dev-server) で土台は完成済 (Phase 2.1〜2.4)。本計画はそこに「コードとの会話」のもう片側を足す

> notes のロードマップ上は Phase 4 = 多言語アダプタだが、フロントエンド充実はそれと並列の別ストリームとして本計画群 (Phase 4.x-fe) を立てる。多言語アダプタは Phase 5 系に繰り下げる想定。

## 2026-06-11 スコープ変更通知 (重要)

本計画着手中に **Phase 4.0.5 (Vue 3 + Vite+ + UnoCSS 基盤)** を立ち上げる方針が固まった。素 JS で新規フロント UI を書くと 4.0.5 で全て二度書きになるため、**本計画のスコープを「サーバ側 API 完成まで」に縮める**。

- **本計画でやる**: FunctionIndex 抽出、syntect ハイライト、`/state` `/spell/<fn>` エンドポイント、`--fn` 廃止、サーバ統合テスト
- **Phase 4.0.5 に移管**: 左右ペアビューの UI 実装、`?fn=` 同期、関数目次クリック切替、preview 実機確認、目視判定
- **既存 inline HTML は維持**: Phase 2.x の機能 (魔法陣・レイヤー toggle・DSL・書き起こし) が動き続ける最低限の継ぎ目だけ確保。新規 UI 機能は載せない

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

### やらないこと (別計画 / スコープ縮小により移管)

- **左右ペアビューの UI 実装、関数目次クリック切替、`?fn=` URL 同期、preview 実機確認、目視判定** → **Phase 4.0.5 に移管** (2026-06-11 スコープ変更)
- 複数ファイル/モジュール階層ナビ (Phase 4.1 で検討)
- 呼び出し先・呼び出し元ジャンプ (Phase 3.4 のデータフロー IR を使う別 Phase: Phase 4.2 候補)
- ソースペインからの編集 (本計画は read-only)
- ~~Vite/React 等のフロントエンドビルド (素 JS 継続)~~ → **Vite+ Vue 3 + UnoCSS で 4.0.5 着手** (方針転換)
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

## 受け入れ基準 (スコープ縮小後・サーバ側のみ)

- [x] `FunctionIndex` がファイル全体の関数 (impl メソッド含む) を qualified 名 (`Foo::bar`) で抽出する
- [x] `syntect` で関数スニペットを HTML ハイライト (InspiredGitHub 等のテーマ固定)
- [x] `GET /state` が関数一覧 + メタを返す
- [x] `GET /spell/<fn>` が個別関数の SVG + source_html + transcript + signature を返す。未知名は 404
- [x] `magia serve <FILE>` で起動 (`--fn` 廃止、`--fn` 指定はエラー)
- [x] 構文エラー時、`/spell/<fn>` は last-good スナップショット + エラー行を返す (UI 表示は 4.0.5)
- [x] ファイル保存時の SSE 更新で関数一覧の差分も配信される
- [x] `cargo test --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` 通過
- [x] **既存 inline HTML 経由で Phase 2.x の機能 (魔法陣・レイヤー toggle・DSL・書き起こし) が引き続き動く** (preview スモークで確認)
- [x] preview 実機確認・目視判定は **4.0.5 M3 で実施** (本フェーズでは要求しない)

## 後続候補

- **Phase 4.1**: ファイル横断ナビ (workspace 内の `*.rs` を再帰スキャン、ファイルツリー追加)
- **Phase 4.2**: 呼び出しジャンプ (Phase 3.4 データフロー IR を流用、関数呼び出し → 呼び出し先関数へワンクリック移動)
- **Phase 4.3**: テーマ切替 (syntect テーマ + 魔法陣カラーパレットを連動)
- **Phase 4.4**: 差分ビュー連携 (Phase 3.2 Spell Diff を「同関数の before/after」と同時に「同ファイルの関数間」両軸で見られるよう拡張)

## 実装ステップ (スコープ縮小後)

**実装フェーズ — サーバ側のみ**

1. **magia-rust**: `FunctionIndex` 型と `list_functions` 拡張 (qualified 名・impl_context・行範囲・signature)。impl メソッド対応に統一
2. **magia-cli**: `syntect` 依存追加 (default-features 最小) + `highlight_rust(snippet) -> String (HTML)`
3. **magia-cli/serve**: 再設計。Shared = 直近正常スナップショット + エラー (行付き)。`/state` (メタ + 関数一覧)、`/spell/<fn>` (svg 両式 + source_html + transcript + signature)、未知 fn は 404
4. **magia-cli**: `--fn` 削除。`magia serve <FILE>` のみ受け入れる。launch.json 更新
5. **既存 inline HTML 維持**: Phase 2.x 機能が動き続けるよう最低限の継ぎ目だけ。**新規フロント UI は 4.0.5 で Vue 実装**
6. **エラーハンドリング**: API レベルで last-good スナップショット返却 + エラー行情報。UI 表示は 4.0.5
7. **テスト**: FunctionIndex golden (impl 内 / トップレベル / 同名重複)、qualified parse、serve 統合 (`/state` `/spell/<fn>` 404 SSE 反映 `--fn` エラー)
8. **Stage 1 品質ゲート** (build / clippy / fmt / test / `.claude/tests/run-all.sh`) + コーディング知見記録 (サーバ側のみ)

**品質検証フェーズ (Stage 2)**

9. レビュー (`addf-code-review-agent`) + コントリビューション検出 (`addf-contribution-agent`) + 指摘対応 → ゲート再実行

**完了処理**

10. 計画 memo、Feedback / TODO 更新、アーカイブ、コミット → 続けて 4.0.5 に着手

## 実装結果メモ (2026-06-11)

- **レイヤリングの計画差分**: FunctionIndex は計画の magia-core でなく **magia-rust**
  (`index.rs`) に置いた。core は言語非依存 (syn 依存はアダプタ層) の既存原則を優先。
  計画の動機「diff/ci でも使う」は両者とも magia-rust 利用者なので満たされる
- index.rs は「索引と本体を同じ walker で集める」(1関心1visitor の意図的例外)。
  impl メソッドは ImplItemFn → ItemFn 正規化で後段を一本化。qualified 名が一意キー
  [break]、素の名前はソース出現順フォールバック。メソッド内ネスト fn は impl 文脈を
  引き継ぐ (意図的近似、テストで固定)
- serve は「直近正常スナップショット + オンデマンドレンダ」に再設計。/spell/<fn> の
  毎回レンダはレンダラの決定論性が担保 (キャッシュ・選択状態が不要)。
  source_html (syntect 5.3, regex-fancy) は 4.0.5 のための先行 API 契約
- スコープ縮小指示を受け、書きかけた素 JS の新 UI は撤回し、既存 UI + 薄い継ぎ目
  (?fn= 同期・先頭フォールバック) に差し戻した
- 統合テストのフレーク修正: version 述語 → 意味述語 (truncate→write の中間状態対策)
- レビュー (Stage 2): Critical 0 / Warning 3 / Suggestion 3 → 全件対応
  (spell_json のロック早期解放、source_html 未使用の明示、ネスト fn 挙動の固定、
  percent_decode 単体テスト、source_lines 防御)

## 想定リスク

- **syntect の依存重量**: 初回ビルドが伸びる可能性。`default-features = false` + 必要な syntax set のみで抑える
- **`?fn=` の name 衝突**: 同名 fn が impl 違いで複数あるケース。`impl_context` を含めた `?fn=Foo::bar` 形式にする (トップレベルは `?fn=bar`)
- **ファイル監視 + 関数選択の競合**: 編集中に「現関数の名前が変わった」場合は line_range の連続性で追跡を試み、追えなければ先頭関数にフォールバック (本計画では best effort)
