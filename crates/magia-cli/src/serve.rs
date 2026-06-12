//! dev-server (Phase 2.1〜2.4, Phase 3.5 式トグル, Phase 4.0 ソース連動ビュー)。
//!
//! `magia serve <FILE>` でファイル全体を監視し、ブラウザに
//! 「ソース | 魔法陣」の左右ペインを配信する (spec §7 / Phase 4.0 計画)。
//!
//! 状態モデル (Phase 4.0):
//! - サーバは**直近の正常スナップショット** (ソース全文 + 関数索引) と
//!   現在の解析エラーだけを持つ。どの関数を見ているかは**クライアントの状態**
//!   (URL `?fn=`) であり、サーバは保持しない
//! - `/state` = ファイルメタ + 関数一覧 + エラー。`/spell/<fn>` = 関数単位の
//!   (svg 両式 + SH 済みソース + 書き起こし) をオンデマンドでレンダリング
//! - 構文エラー中は直近の正常スナップショットから配信し続ける (会話を切らない原則)
//!
//! 構成は同期スレッドモデル: tiny_http の受信ループ + リクエストごとのスレッド
//! (SSE 接続はスレッドを1本占有するが、ローカル開発ツールの同時接続数では問題ない)。

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use notify::{RecursiveMode, Watcher};

use magia_core::layout::layout;
use magia_core::proximity::{NeighborSeed, classify_neighbors};
use magia_core::render::ir_export::{NeighborMeta, SpanIr, SpellIr, focus_layout, spell_ir};
use magia_rust::{FunctionEntry, function_index_with_calls, parse_function, workspace_index};

use crate::srcview::highlight_rust;

/// エディタの「テンポラリ書き込み → rename」保存で複数イベントが連射されるため、
/// 最後のイベントからこの時間だけ静まるのを待ってから1回だけ再解析する。
const DEBOUNCE: Duration = Duration::from_millis(120);

/// 直近の正常スナップショット。構文エラー中の配信はここから行う。
/// `file` を含めて1ロックでスワップする — 切替 (Phase 4.4.5) 中に
/// 「新しい一覧 + 旧ファイル名」のような不整合な応答を返さない。
#[derive(Default)]
struct GoodSnapshot {
    /// このスナップショットの由来ファイル (表示名・diff の gitio 解決に使う)。
    file: PathBuf,
    source: String,
    functions: Vec<FunctionEntry>,
    /// 関数間の呼び出しエッジ (近接度の入力)。reload 時に1回だけ構築する
    /// (リクエストごとの再パースを避ける — Phase 4.2)。
    call_edges: Vec<(String, String)>,
}

/// 現在の解析エラー (正常時は None)。
struct ServeError {
    message: String,
    /// 構文エラーの行 (1始まり)。ソースペインの案内に使う。
    line: Option<usize>,
}

/// ブラウザに配る共有状態。
struct Shared {
    /// watcher スレッドへの制御送信 (ファイル切替、Phase 4.4.5)。
    /// 起動順 (`run`: reload → spawn_watcher → HTTP bind) の都合で遅延設定。
    watch_tx: Mutex<Option<Sender<WatchMsg>>>,
    good: Mutex<GoodSnapshot>,
    error: Mutex<Option<ServeError>>,
    /// 更新世代。クライアントはこの値の変化で再取得する。
    version: AtomicU64,
    /// SSE 接続中クライアントへの通知チャネル。
    clients: Mutex<Vec<Sender<u64>>>,
}

/// Mutex の poisoning から回復してロックを取る。
/// dev-server は一部スレッドのパニック後も「壊れたまま応答し続ける」より
/// 「最後の正常状態で応答し続ける」方が望ましい (クラッシュループの回避)。
fn lock_or_recover<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl Shared {
    fn new() -> Self {
        Self {
            watch_tx: Mutex::new(None),
            good: Mutex::new(GoodSnapshot::default()),
            error: Mutex::new(None),
            version: AtomicU64::new(0),
            clients: Mutex::new(Vec::new()),
        }
    }

    /// ファイルを読み直してスナップショットを更新し、SSE クライアントへ通知する。
    fn reload(&self, file: &Path) {
        match load_snapshot(file) {
            Ok(snapshot) => {
                *lock_or_recover(&self.good) = snapshot;
                *lock_or_recover(&self.error) = None;
            }
            Err(error) => {
                // スナップショットは触らない: 直前の正常な内容を配信し続ける (spec §7)。
                *lock_or_recover(&self.error) = Some(error);
            }
        }
        let version = self.version.fetch_add(1, Ordering::SeqCst) + 1;
        lock_or_recover(&self.clients).retain(|client| client.send(version).is_ok());
    }

    /// `/state`: ファイルメタ + 関数一覧 + エラー (魔法陣・ソースは `/spell/<fn>` 側)。
    fn state_json(&self) -> String {
        let good = lock_or_recover(&self.good);
        let functions: Vec<_> = good
            .functions
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "qualified": entry.qualified,
                    "name": entry.name,
                    "impl_context": entry.impl_context,
                    "signature": entry.signature,
                    "start_line": entry.start_line,
                    "end_line": entry.end_line,
                })
            })
            .collect();
        let error = lock_or_recover(&self.error);
        serde_json::json!({
            "version": self.version.load(Ordering::SeqCst),
            "file": good.file.display().to_string(),
            "functions": functions,
            "error": error.as_ref().map(|e| serde_json::json!({
                "message": e.message,
                "line": e.line,
            })),
        })
        .to_string()
    }

    /// `/spell/<fn>`: 関数単位のレンダリング (オンデマンド、サーバに選択状態を持たない)。
    /// 不明な関数は `None` (404 は呼び出し側の責務)。
    fn spell_json(
        &self,
        qualified: &str,
        with_neighbors: bool,
        diff_rev: Option<&str>,
    ) -> Option<Result<String, String>> {
        // レンダリング (parse + layout + render ×2 + syntect) は重いので、
        // ロックは複製を取るまでに留める — reload (保存) を /spell がブロックしない。
        let (source, entry, all, call_edges, file) = {
            let good = lock_or_recover(&self.good);
            let entry = good
                .functions
                .iter()
                .find(|entry| entry.qualified == qualified)?
                .clone();
            let (all, call_edges) = if with_neighbors {
                (good.functions.clone(), good.call_edges.clone())
            } else {
                (Vec::new(), Vec::new())
            };
            // file はスナップショットと同一ロックで取る — diff の gitio 解決が
            // 「good と別ファイル」を見る不整合を防ぐ (レビュー W-2)。
            (
                good.source.clone(),
                entry,
                all,
                call_edges,
                good.file.clone(),
            )
        };
        Some(render_spell(
            &source,
            &entry,
            with_neighbors.then_some(NeighborsInput {
                functions: &all,
                call_edges: &call_edges,
            }),
            diff_rev.map(|rev| DiffInput { rev, file: &file }),
        ))
    }
}

/// ワークスペース (cwd) 配下の .rs ファイルを列挙する (Phase 4.4.5 の切替候補)。
/// target / 隠しディレクトリ / node_modules は除外。cwd 相対パスのソート済み。
fn list_rs_files() -> Vec<String> {
    let mut files = Vec::new();
    let mut stack = vec![PathBuf::from(".")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.is_dir() {
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                stack.push(path);
            } else if name.ends_with(".rs") {
                // "./" プレフィックスを落として URL・表示に使いやすい形へ。
                let display = path
                    .strip_prefix(".")
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                files.push(display);
            }
        }
    }
    files.sort();
    files
}

/// ワークスペース俯瞰 (Phase 4.5 M1): 全 .rs の関数一覧 + ファイル横断の呼び出し
/// エッジ (M2 前段)。パース失敗 (構文エラー・マクロ限界) は俯瞰を壊さず error
/// フラグでスキップする。読めないファイル (権限・削除レース) も同じ error 扱い。
/// オンデマンド構築 (キャッシュなし — ローカル dev の俯瞰を開く頻度では十分)。
fn workspace_json() -> String {
    let sources: Vec<(String, String)> = list_rs_files()
        .into_iter()
        .map(|path| {
            // 読めない場合は空ソース (パース成功・関数0) でなく構文エラーに倒して
            // error フラグを立てる ("(" は対応括弧がなく syn::File として必ず不正)。
            let source = std::fs::read_to_string(&path).unwrap_or_else(|_| "(".to_string());
            (path, source)
        })
        .collect();
    // local_edges (FileIndex に含まれる同ファイル内エッジ) は M2 本実装
    // (ミニ魔法陣タイル等) まで JSON に載せない — 俯瞰のペイロードを薄く保つ。
    let (files, cross_edges) = workspace_index(&sources);
    let files: Vec<serde_json::Value> = files
        .iter()
        .map(|file| {
            let dir = Path::new(&file.path)
                .parent()
                .map_or_else(String::new, |p| p.display().to_string());
            let functions: Vec<_> = file
                .entries
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "qualified": f.qualified,
                        "signature": f.signature,
                    })
                })
                .collect();
            if file.error {
                serde_json::json!({ "path": file.path, "dir": dir, "functions": [], "error": true })
            } else {
                serde_json::json!({ "path": file.path, "dir": dir, "functions": functions })
            }
        })
        .collect();
    let cross_edges: Vec<serde_json::Value> = cross_edges
        .iter()
        .map(|e| {
            serde_json::json!({
                "from_file": e.from_file,
                "from": e.from,
                "to_file": e.to_file,
                "to": e.to,
            })
        })
        .collect();
    serde_json::json!({ "files": files, "cross_edges": cross_edges }).to_string()
}

/// 監視ファイルの切替 (Phase 4.4.5)。検証して watcher スレッドへ送る。
/// 成功時は採用した cwd 相対パス (クライアントが URL に載せる正規形) を返す。
fn switch_file(shared: &Shared, requested: &str) -> Result<String, String> {
    let path = Path::new(requested);
    if path.extension().and_then(|e| e.to_str()) != Some("rs") {
        return Err("対象は .rs ファイルのみです".to_string());
    }
    if path.is_absolute() {
        return Err("ワークスペース相対パスで指定してください".to_string());
    }
    // canonicalize でシンボリックリンク・`..` を解決してから境界を検証する
    // (任意パス読み出しにしない — 計画のセキュリティ境界)。
    let root = std::fs::canonicalize(".").map_err(|e| format!("cwd を解決できません: {e}"))?;
    let canonical = std::fs::canonicalize(path)
        .map_err(|_| format!("ファイルが見つかりません: {requested}"))?;
    if !canonical.starts_with(&root) {
        return Err("ワークスペースの外は監視できません".to_string());
    }
    if !canonical.is_file() {
        return Err(format!("ファイルではありません: {requested}"));
    }
    let relative = canonical
        .strip_prefix(&root)
        .map_err(|_| "相対パスへ変換できません".to_string())?
        .to_path_buf();
    let tx = lock_or_recover(&shared.watch_tx);
    let Some(tx) = tx.as_ref() else {
        return Err("watcher が初期化されていません".to_string());
    };
    tx.send(WatchMsg::Switch(relative.clone()))
        .map_err(|_| "watcher スレッドが停止しています".to_string())?;
    Ok(relative.display().to_string())
}

/// 差分強調の入力 (`?diff=<REV>` のときだけ渡る — Phase 4.3.7)。
struct DiffInput<'a> {
    /// 比較基準の git リビジョン (`HEAD~1` / `main` 等)。
    rev: &'a str,
    /// 監視対象ファイル (gitio がリポジトリ相対パスへ解決する)。
    file: &'a Path,
}

/// ピン中心ビューの周辺計算入力 (`?with=neighbors` のときだけ渡る)。
struct NeighborsInput<'a> {
    functions: &'a [FunctionEntry],
    /// 関数間の呼び出しエッジ (caller, callee) — 近接度の「呼び出し関係」成分。
    call_edges: &'a [(String, String)],
}

/// ファイルを読み、関数索引つきのスナップショットを作る。
fn load_snapshot(file: &Path) -> Result<GoodSnapshot, ServeError> {
    let source = std::fs::read_to_string(file).map_err(|e| ServeError {
        message: format!("ファイルを読み込めません: {}: {e}", file.display()),
        line: None,
    })?;
    let (functions, call_edges) = function_index_with_calls(&source).map_err(|e| ServeError {
        line: syntax_error_line(&e),
        message: e.to_string(),
    })?;
    Ok(GoodSnapshot {
        file: file.to_path_buf(),
        source,
        functions,
        call_edges,
    })
}

/// 構文エラーの行番号 (ソースペインの案内に使う)。
fn syntax_error_line(error: &magia_rust::Error) -> Option<usize> {
    match error {
        magia_rust::Error::Syntax(syntax) => Some(syntax.span().start().line),
        magia_rust::Error::FunctionNotFound { .. } => None,
    }
}

/// 関数1つ分の応答 JSON を組み立てる。Err は内部不整合 (500 相当)。
///
/// `source_html` は Phase 4.0.5 (Vue のソースペイン) が使う先行実装で、
/// 現行 UI (薄い継ぎ目 JS) はまだ表示しない — API 契約として先に固定しておく。
fn render_spell(
    source: &str,
    entry: &FunctionEntry,
    neighbors_from: Option<NeighborsInput<'_>>,
    diff_from: Option<DiffInput<'_>>,
) -> Result<String, String> {
    let graph = parse_function(source, &entry.qualified).map_err(|e| e.to_string())?;
    let placed = layout(&graph);
    let snippet = source_lines(source, entry.start_line, entry.end_line);
    // Phase 4.0.9/4.3: 両式とも配置済み IR (JSON) で返し、Vue が描画する。
    // SVG 文字列は出さない (Rust SVG レンダラは Phase 4.3 M5 で削除済み)。
    //
    // diff 文脈 (?diff=<REV>、Phase 4.3.7): before を git から引いて差分強調つき
    // IR (viewBox ゴースト拡張込み) に差し替える。rev 不正・git 外・before に
    // 関数が無い (新規関数) は応答を壊さず diff_note (案内文) に畳む —
    // 「会話を切らない」原則。保存のたびに再計算されるため live diff になる。
    let mut diff_overlay: Option<Vec<magia_core::render::ir_export::DiffMarkIr>> = None;
    let mut diff_report: Option<String> = None;
    let mut diff_note: Option<String> = None;
    let spell = match diff_from {
        Some(input) => match diff_before_graph(&input, &entry.qualified) {
            Ok(before_graph) => {
                let spell_diff = magia_core::diff::diff(&before_graph, &graph);
                diff_report = Some(spell_diff.to_report(&entry.qualified));
                let (ir, marks) = magia_core::render::ir_export::diff_spell_ir(
                    &before_graph,
                    &graph,
                    &spell_diff,
                );
                diff_overlay = Some(marks);
                ir
            }
            Err(note) => {
                diff_note = Some(note);
                spell_ir(&graph, &placed)
            }
        },
        None => spell_ir(&graph, &placed),
    };
    // Phase 4.1/4.2: ピン中心ビューの周辺配置。近接度は同impl/呼び出し関係/
    // 同ファイルの連続距離 (proximity.rs)、リング離散化は focus_layout 側。
    let focus_layout = neighbors_from.map(|input| {
        let all = input.functions;
        let seed = |e: &FunctionEntry| NeighborSeed {
            qualified: e.qualified.clone(),
            impl_context: e.impl_context.clone(),
        };
        let classified = classify_neighbors(
            &seed(entry),
            &all.iter().map(seed).collect::<Vec<_>>(),
            input.call_edges,
        );
        let with_meta: Vec<_> = classified
            .into_iter()
            .filter_map(|neighbor| {
                let meta = all.iter().find(|e| e.qualified == neighbor.qualified)?;
                Some((
                    neighbor,
                    NeighborMeta {
                        name: meta.name.clone(),
                        signature: meta.signature.clone(),
                    },
                ))
            })
            .collect();
        focus_layout(
            spell.view_box,
            &entry.qualified,
            &with_meta,
            input.call_edges,
        )
    });
    let (call_excerpts, op_excerpts, ring_excerpts) = excerpt_maps(source, &spell);
    let ir = serde_json::to_value(spell).map_err(|e| e.to_string())?;
    // ベルカ式も配置済み IR (Phase 4.3 — Vue の BelkaCircle が描く)。
    let belka = serde_json::to_value(magia_core::render::belka::belka_ir(&graph))
        .map_err(|e| e.to_string())?;
    let mut response = serde_json::json!({
        "qualified": entry.qualified,
        "signature": entry.signature,
        "ir": ir,
        "belka_ir": belka,
        "call_excerpts": call_excerpts,
        "op_excerpts": op_excerpts,
        "ring_excerpts": ring_excerpts,
        "source_html": highlight_rust(&snippet),
        "transcript": magia_core::transcript::transcribe(&graph),
        "start_line": entry.start_line,
    });
    if let Some(layout) = focus_layout {
        response["focus_layout"] = serde_json::to_value(layout).map_err(|e| e.to_string())?;
    }
    if let Some(marks) = diff_overlay {
        response["diff_overlay"] = serde_json::to_value(marks).map_err(|e| e.to_string())?;
    }
    if let Some(report) = diff_report {
        response["diff_report"] = serde_json::Value::String(report);
    }
    if let Some(note) = diff_note {
        response["diff_note"] = serde_json::Value::String(note);
    }
    Ok(response.to_string())
}

type ExcerptMap = serde_json::Map<String, serde_json::Value>;

/// 原文断片マップ群 (ホバープレビュー・インスペクタ用、Phase 4.1) を構築する。
/// 返り値は **`(call, op, ring)` の順** (3要素とも同型なので取り違えに注意):
/// - call: 召喚印の呼び出し式 (glyph id)。レシーバ・引数込みの式全体
/// - op: 操作ドットの文・ヘッダ (`<ring_id>-<出現順>` = Vue の Operation.irKey)
/// - ring: 補助リングのガード・ヘッダ (ring id)
fn excerpt_maps(source: &str, spell: &SpellIr) -> (ExcerptMap, ExcerptMap, ExcerptMap) {
    // 壊れた span で空になった断片は載せない (クライアントは欠落として扱う)。
    let excerpt_html = |span: &SpanIr| {
        let excerpt = span_excerpt(source, span);
        (!excerpt.trim().is_empty()).then(|| serde_json::Value::String(highlight_rust(&excerpt)))
    };
    let call = spell
        .glyphs
        .iter()
        .filter_map(|glyph| {
            let span = glyph.source_span.as_ref()?;
            Some((glyph.id.to_string(), excerpt_html(span)?))
        })
        .collect();
    let op = spell
        .rings
        .iter()
        .flat_map(|ring| {
            ring.operations
                .iter()
                .enumerate()
                .map(move |(index, op)| (ring.id, index, op))
        })
        .filter_map(|(ring_id, index, op)| {
            let span = op.source_span.as_ref()?;
            Some((format!("{ring_id}-{index}"), excerpt_html(span)?))
        })
        .collect();
    let ring = spell
        .rings
        .iter()
        .filter_map(|ring| {
            let span = ring.guard_span.as_ref()?;
            Some((ring.id.to_string(), excerpt_html(span)?))
        })
        .collect();
    (call, op, ring)
}

/// diff 基準 (`?diff=<REV>`) の関数グラフを git から引く。
/// 失敗は案内文 (`diff_note`) として返す — UI を壊さない。
fn diff_before_graph(
    input: &DiffInput<'_>,
    qualified: &str,
) -> Result<magia_core::ir::MagiaGraph, String> {
    // before はリクエストのたびに git へ問い合わせる (キャッシュしない)。
    // rev のファイル内容は不変だが、保存 → SSE → このパスの再実行が
    // 「作業ツリーの変更が即ハローに現れる」live diff の成立条件 — after 側の
    // 再計算と対で動くため、ここだけ最適化すると将来 rev 切替時の整合を壊しやすい。
    let before_source = crate::gitio::show_file_at(input.rev, input.file)
        .map_err(|e| format!("リビジョン {} を読めません: {e:#}", input.rev))?;
    parse_function(&before_source, qualified).map_err(|e| match e {
        magia_rust::Error::FunctionNotFound { .. } => format!(
            "リビジョン {} にこの関数はありません (新規関数のため差分はありません)",
            input.rev
        ),
        other @ magia_rust::Error::Syntax(_) => {
            format!("リビジョン {} の解析に失敗: {other}", input.rev)
        }
    })
}

/// span の範囲の原文を切り出す (行・列とも 1-based・文字単位、
/// `end_column` は exclusive — `SpanIr` の規約)。呼び出し式・操作ドットの
/// プレビューに使う。式は行の途中から始まるため 1行目は列頭を落とし、
/// 2行目以降は共通の先頭空白を除いて左端を揃える
/// (ポップオーバー内で元ファイルの深いインデントを引きずらない)。
fn span_excerpt(source: &str, span: &SpanIr) -> String {
    let start_line = span.start_line as usize;
    let end_line = (span.end_line as usize).max(start_line);
    let lines: Vec<&str> = source
        .lines()
        .skip(start_line.saturating_sub(1))
        .take(end_line - start_line + 1)
        .collect();
    let last = lines.len().saturating_sub(1);
    let clipped: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            // 列は proc_macro2 と同じ文字単位 — バイト境界で切らない (UTF-8 防御)
            let chars: Vec<char> = line.chars().collect();
            let from = if i == 0 {
                (span.start_column as usize)
                    .saturating_sub(1)
                    .min(chars.len())
            } else {
                0
            };
            let to = if i == last {
                (span.end_column as usize)
                    .saturating_sub(1)
                    .clamp(from, chars.len())
            } else {
                chars.len()
            };
            chars[from..to].iter().collect()
        })
        .collect();
    let indent = clipped
        .iter()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);
    let mut snippet = clipped
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 {
                line
            } else {
                line.chars().skip(indent).collect()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    snippet.push('\n');
    snippet
}

/// 1始まり・両端含みの行範囲を切り出す (関数スニペット)。
fn source_lines(source: &str, start_line: usize, end_line: usize) -> String {
    // span が正常なら end >= start。壊れた入力でも最低1行は返す防御。
    let end_line = end_line.max(start_line);
    let mut snippet: String = source
        .lines()
        .skip(start_line.saturating_sub(1))
        .take(end_line.saturating_sub(start_line) + 1)
        .collect::<Vec<_>>()
        .join("\n");
    snippet.push('\n');
    snippet
}

/// dev-server を起動する。Ctrl-C で終了するまで戻らない。
pub(crate) fn run(file: &Path, port: u16) -> Result<()> {
    let file = file.to_path_buf();
    let shared = Arc::new(Shared::new());

    // 初回読み込み (エラーでもサーバは起動し、画面にエラーを出す)。
    shared.reload(&file);

    spawn_watcher(Arc::clone(&shared), file.clone())?;

    let server = tiny_http::Server::http(("127.0.0.1", port))
        .map_err(|e| anyhow::anyhow!("サーバを起動できません: {e}"))?;
    let actual_port = match server.server_addr() {
        tiny_http::ListenAddr::IP(addr) => addr.port(),
        // 本実装は IP しか bind しないため到達しない (防御)。
        tiny_http::ListenAddr::Unix(_) => port,
    };
    println!(
        "serving http://127.0.0.1:{actual_port}/  ({})",
        file.display()
    );
    // パイプ接続時はブロックバッファリングされるため明示 flush (テストが URL 行を待つ)。
    std::io::stdout().flush().ok();

    for request in server.incoming_requests() {
        let shared = Arc::clone(&shared);
        std::thread::spawn(move || handle_request(request, &shared));
    }
    Ok(())
}

/// ファイル監視スレッドを起動する。
///
/// エディタの rename 保存 (テンポラリ書き込み → 置換) でも検知できるよう、
/// ファイルそのものではなく**親ディレクトリ**を非再帰で監視し、
/// イベントのパスをファイル名で照合する。
/// watcher スレッドへのメッセージ: FS イベントとファイル切替 (Phase 4.4.5) を
/// 1本の channel に統合する (std mpsc には select が無いため)。
enum WatchMsg {
    Fs(notify::Result<notify::Event>),
    /// 監視対象の切替 (検証済みパス — `switch_file` だけが送る)。
    Switch(PathBuf),
}

/// ファイルの親ディレクトリ (notify の監視単位)。
fn parent_dir(file: &Path) -> PathBuf {
    file.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn spawn_watcher(shared: Arc<Shared>, file: PathBuf) -> Result<()> {
    let parent = parent_dir(&file);
    let watched_name = file
        .file_name()
        .context("監視対象のファイル名を特定できません")?
        .to_os_string();

    let (tx, rx) = channel::<WatchMsg>();
    let fs_tx = tx.clone();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = fs_tx.send(WatchMsg::Fs(event));
    })
    .context("ファイルウォッチャを作成できません")?;
    watcher
        .watch(&parent, RecursiveMode::NonRecursive)
        .with_context(|| format!("監視を開始できません: {}", parent.display()))?;
    *lock_or_recover(&shared.watch_tx) = Some(tx);

    std::thread::spawn(move || {
        // watcher はこのスレッドが握り続ける (drop すると監視が止まる)。
        let mut state = WatcherState {
            watcher,
            current: file,
            parent,
            name: watched_name,
        };
        while let Ok(message) = rx.recv() {
            match message {
                WatchMsg::Fs(event) => {
                    if !event_touches(&event, &state.name) {
                        continue;
                    }
                    // デバウンス: 静まるまで追加 FS イベントを飲み込む。
                    // Switch を飲んだら退避して reload 後に処理する
                    // (黙って捨てると「切替したのに旧ファイル監視のまま」— レビュー W-1)。
                    let mut pending_switch = None;
                    loop {
                        match rx.recv_timeout(DEBOUNCE) {
                            Ok(WatchMsg::Fs(_)) => {}
                            Ok(WatchMsg::Switch(next)) => {
                                pending_switch = Some(next);
                                break;
                            }
                            Err(_) => break,
                        }
                    }
                    shared.reload(&state.current);
                    if let Some(next) = pending_switch {
                        state.switch(&shared, next);
                    }
                }
                WatchMsg::Switch(next) => state.switch(&shared, next),
            }
        }
    });
    Ok(())
}

/// watcher スレッドの状態 (監視対象と notify ハンドル)。
struct WatcherState {
    watcher: notify::RecommendedWatcher,
    current: PathBuf,
    parent: PathBuf,
    name: std::ffi::OsString,
}

impl WatcherState {
    /// 監視対象を切り替える。親ディレクトリが変わるときは watch を張り替え、
    /// 失敗したら旧監視へ戻して中止する (壊れた状態にしない)。
    /// `Shared` 側のファイル名公開は reload (スナップショットの一括スワップ) に
    /// 一本化する — 一覧とファイル名が別ロックでずれない (レビュー W-2)。
    fn switch(&mut self, shared: &Shared, next: PathBuf) {
        let Some(name) = next.file_name() else {
            return; // 検証済みパスのみ届く想定 (防御)
        };
        let next_parent = parent_dir(&next);
        if next_parent != self.parent {
            // 旧監視の解除失敗は致命でない (新監視が機能すればよい)。
            let _ = self.watcher.unwatch(&self.parent);
            if self
                .watcher
                .watch(&next_parent, RecursiveMode::NonRecursive)
                .is_err()
            {
                let _ = self
                    .watcher
                    .watch(&self.parent, RecursiveMode::NonRecursive);
                return;
            }
            self.parent = next_parent;
        }
        self.name = name.to_os_string();
        self.current = next;
        shared.reload(&self.current);
    }
}

fn event_touches(event: &notify::Result<notify::Event>, watched_name: &std::ffi::OsStr) -> bool {
    match event {
        Ok(event) => event
            .paths
            .iter()
            .any(|path| path.file_name() == Some(watched_name)),
        // 監視エラーは「変化したかもしれない」側に倒して再解析する。
        Err(_) => true,
    }
}

// ===== HTTP =====

/// Vue SPA (web/dist)。build.rs が bun でビルドして同梱する (Phase 4.0.5 M5)。
/// 開発時は vite+ dev (HMR) を使い、この埋め込みは本番配信専用。
#[derive(rust_embed::Embed)]
#[folder = "../../web/dist"]
struct WebDist;

fn handle_request(request: tiny_http::Request, shared: &Shared) {
    let url = request.url().to_string();
    // ルーティングはパス部分のみで行う (`?fn=...` 等の状態クエリが付いても返す)。
    // split は常に1要素以上返すため unwrap_or は不達 (防御のみ)。
    let path = url.split('?').next().unwrap_or("/").to_string();
    let result = match (request.method(), path.as_str()) {
        (tiny_http::Method::Get, "/") => request.respond(embedded_response("index.html")),
        (tiny_http::Method::Get, "/state") => request.respond(
            tiny_http::Response::from_string(shared.state_json())
                .with_header(header("Content-Type", "application/json; charset=utf-8")),
        ),
        (tiny_http::Method::Get, spell) if spell.starts_with("/spell/") => {
            let qualified = percent_decode(&spell["/spell/".len()..]);
            // ?with=neighbors でピン中心ビューの周辺配置 (focus_layout) を併載する
            // (Phase 4.1)。なしは従来契約のまま (静的レンダ・テスト互換)。
            let with_neighbors = url
                .split_once('?')
                .is_some_and(|(_, query)| query.split('&').any(|kv| kv == "with=neighbors"));
            // ?diff=<REV> で差分強調 (Phase 4.3.7)。空値は指定なしと同じ。
            let diff_rev = url
                .split_once('?')
                .and_then(|(_, query)| {
                    query
                        .split('&')
                        .find_map(|kv| kv.strip_prefix("diff="))
                        .map(percent_decode)
                })
                .filter(|rev| !rev.is_empty())
                // `-` 始まりは git のオプションとして解釈されうる (--format=%H 等の
                // オプション注入)。rev として正当な名前に `-` 始まりは無いので弾く。
                .filter(|rev| !rev.starts_with('-'));
            request.respond(spell_response(
                shared,
                &qualified,
                with_neighbors,
                diff_rev.as_deref(),
            ))
        }
        // Phase 4.5 M1: ワークスペース俯瞰 (全ファイルの関数一覧)。
        (tiny_http::Method::Get, "/workspace") => request.respond(
            tiny_http::Response::from_string(workspace_json())
                .with_header(header("Content-Type", "application/json; charset=utf-8")),
        ),
        // Phase 4.4.5: 監視ファイルの一覧と切替。
        (tiny_http::Method::Get, "/files") => request.respond(
            tiny_http::Response::from_string(
                serde_json::json!({ "files": list_rs_files() }).to_string(),
            )
            .with_header(header("Content-Type", "application/json; charset=utf-8")),
        ),
        (tiny_http::Method::Post, "/file") => {
            let mut request = request;
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            let requested = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| v["path"].as_str().map(str::to_string));
            let json_header = header("Content-Type", "application/json; charset=utf-8");
            let response = match requested {
                Some(path) => match switch_file(shared, &path) {
                    Ok(file) => tiny_http::Response::from_string(
                        serde_json::json!({ "file": file }).to_string(),
                    )
                    .with_header(json_header),
                    Err(message) => tiny_http::Response::from_string(
                        serde_json::json!({ "error": message }).to_string(),
                    )
                    .with_status_code(400)
                    .with_header(json_header),
                },
                None => tiny_http::Response::from_string(
                    serde_json::json!({ "error": "JSON ボディに path がありません" }).to_string(),
                )
                .with_status_code(400)
                .with_header(json_header),
            };
            request.respond(response)
        }
        (tiny_http::Method::Get, "/events") => {
            let (tx, rx) = channel::<u64>();
            {
                // 新規接続のたびに切断済みクライアントを掃除する (publish が無い間の
                // Sender リーク防止)。現世代を送って生存判定を兼ねる (受信側は
                // 冪等な refresh をするだけなので余分な1イベントは無害)。
                // 既知の制約: 切断済み Sender は次の reload か新規接続まで残る
                // (ローカル開発ツールの接続数では許容)。
                let version = shared.version.load(Ordering::SeqCst);
                let mut clients = lock_or_recover(&shared.clients);
                clients.retain(|client| client.send(version).is_ok());
                clients.push(tx);
            }
            // SSE は tiny_http の Response 経路を使わない。チャンク転送経路は
            // chunked_transfer::Encoder (8KB) と BufWriter (1KB) の二重バッファが
            // flush されず、イベントがクライアントに届かないため (Phase 4.0.5 M2 で
            // 発見した Phase 2.1 からの潜在バグ)。生 writer に自前でヘッダを書き、
            // イベントごとに flush する。
            stream_sse(request.into_writer(), &rx);
            return;
        }
        // API 以外は SPA の静的ファイル (/assets/*.js, /favicon.svg, ...)。
        // 注: SPA のルートは現状 `/` 1本のため index.html フォールバックは持たない。
        // Vue Router にパスベースのビューを足すときは 404 → index.html のフォールバックが要る。
        (tiny_http::Method::Get, file) => {
            request.respond(embedded_response(file.trim_start_matches('/')))
        }
        _ => request.respond(tiny_http::Response::from_string("not found").with_status_code(404)),
    };
    // クライアント切断は正常系 (SSE はブラウザを閉じれば必ずここに来る)。
    let _ = result;
}

/// 埋め込み済み SPA ファイルの応答 (200 / 404)。
fn embedded_response(file: &str) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    match WebDist::get(file) {
        Some(content) => {
            // 拡張子 → Content-Type の最小マップ (dist に現れる種類だけで足りる)。
            let mime = match file.rsplit('.').next() {
                Some("html") => "text/html; charset=utf-8",
                Some("js") => "text/javascript; charset=utf-8",
                Some("css") => "text/css; charset=utf-8",
                Some("svg") => "image/svg+xml",
                Some("png") => "image/png",
                Some("json") => "application/json; charset=utf-8",
                _ => "application/octet-stream",
            };
            tiny_http::Response::from_data(content.data.into_owned())
                .with_header(header("Content-Type", mime))
        }
        None => tiny_http::Response::from_data(b"not found".to_vec()).with_status_code(404),
    }
}

/// SSE ストリームを生 writer へ書き続ける。クライアント切断 (write/flush エラー)
/// またはサーバ終了 (Sender 全 drop) まで返らない。
/// `Connection: close` でこの接続をイベント専用にする (keep-alive の次リクエストを
/// 同一接続に載せさせない — レスポンスは EOF 終端のストリームのため)。
fn stream_sse(mut writer: Box<dyn Write + Send>, rx: &Receiver<u64>) {
    let send = |writer: &mut Box<dyn Write + Send>, payload: &str| -> std::io::Result<()> {
        writer.write_all(payload.as_bytes())?;
        writer.flush()
    };
    // 接続直後に1イベント流し、EventSource の onmessage → refresh を促す。
    // ペイロードの「0」に意味はない (クライアントは値を見ず refresh するだけ)。
    // 将来 version を差分処理に使う場合は接続直後イベントを実世代に変えること。
    if send(
        &mut writer,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\ndata: 0\n\n",
    )
    .is_err()
    {
        return;
    }
    while let Ok(version) = rx.recv() {
        if send(&mut writer, &format!("data: {version}\n\n")).is_err() {
            return; // クライアント切断 (正常系)
        }
    }
    // サーバ側終了 → writer drop で接続クローズ
}

/// `/spell/<fn>` の応答 (200 / 404 / 500)。
fn spell_response(
    shared: &Shared,
    qualified: &str,
    with_neighbors: bool,
    diff_rev: Option<&str>,
) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    let json_header = header("Content-Type", "application/json; charset=utf-8");
    match shared.spell_json(qualified, with_neighbors, diff_rev) {
        Some(Ok(json)) => tiny_http::Response::from_string(json).with_header(json_header),
        Some(Err(message)) => {
            tiny_http::Response::from_string(serde_json::json!({ "error": message }).to_string())
                .with_status_code(500)
                .with_header(json_header)
        }
        None => tiny_http::Response::from_string(
            serde_json::json!({ "error": format!("関数 `{qualified}` は索引にありません") })
                .to_string(),
        )
        .with_status_code(404)
        .with_header(json_header),
    }
}

fn header(field: &str, value: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(field.as_bytes(), value.as_bytes()).expect("静的ヘッダは常に有効")
}

/// URL パスセグメントの最小パーセントデコード (`Foo%3A%3Abar` → `Foo::bar`)。
/// 不正なエンコードは文字をそのまま通す (防御的、未知名は 404 側で吸収される)。
fn percent_decode(segment: &str) -> String {
    let bytes = segment.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = bytes.get(i + 1..i + 3)
            && let Ok(hex_str) = std::str::from_utf8(hex)
            && let Ok(value) = u8::from_str_radix(hex_str, 16)
        {
            out.push(value);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_decode_handles_qualified_names() {
        assert_eq!(percent_decode("Foo%3A%3Abar"), "Foo::bar");
        assert_eq!(percent_decode("plain_name"), "plain_name");
    }

    #[test]
    fn percent_decode_passes_incomplete_sequences_through() {
        // 不正・不完全なエンコードは文字をそのまま通す (404 側で吸収される)。
        assert_eq!(percent_decode("Foo%3"), "Foo%3");
        assert_eq!(percent_decode("Foo%"), "Foo%");
        assert_eq!(percent_decode("Foo%ZZ"), "Foo%ZZ");
    }

    #[test]
    fn source_lines_clamps_reversed_range() {
        assert_eq!(source_lines("a\nb\nc\n", 2, 1), "b\n");
    }

    fn span(sl: u32, sc: u32, el: u32, ec: u32) -> SpanIr {
        SpanIr {
            start_line: sl,
            end_line: el,
            start_column: sc,
            end_column: ec,
        }
    }

    #[test]
    fn span_excerpt_clips_single_line_by_columns() {
        // `return helper(sum);` の `helper(sum)` 部分 (列 12〜23、end exclusive)
        let source = "fn f() {\n    return helper(sum);\n}\n";
        assert_eq!(span_excerpt(source, &span(2, 12, 2, 23)), "helper(sum)\n");
    }

    #[test]
    fn span_excerpt_dedents_continuation_lines() {
        // メソッドチェーンの継続行は共通インデントを除いて左端を揃える
        let source = "fn f() {\n    let x = sigil\n        .layers\n        .map(|r| r.kind);\n}\n";
        assert_eq!(
            span_excerpt(source, &span(2, 13, 4, 25)),
            "sigil\n.layers\n.map(|r| r.kind)\n"
        );
    }

    #[test]
    fn span_excerpt_clamps_columns_beyond_line_end() {
        // 壊れた span (列が行長を超える) でも panic せず行内に丸める
        let source = "call()\n";
        assert_eq!(span_excerpt(source, &span(1, 1, 1, 99)), "call()\n");
    }

    #[test]
    fn span_excerpt_collapses_reversed_columns_to_empty() {
        // 壊れた span (end < start) は空へフォールバックする (panic しない。
        // クライアントは空の式ブロックを表示しないので欠落として扱われる)
        let source = "call()\n";
        assert_eq!(span_excerpt(source, &span(1, 5, 1, 2)), "\n");
    }
}
