//! 静止画レンダ `magia-render` (Vue SSR の単一実行ファイル、Phase 4.3) の
//! サブプロセス統合。
//!
//! `magia render` の SVG 出力は動的 UI (serve) と同じ Vue コンポーネントツリーで
//! 描く — 意匠の定義を Vue 1箇所に保つための経路 (計画 4.3 の Vue 1本化)。
//! Rust 側は配置済み IR (JSON) を stdin に流し、stdout から SVG を受け取るだけ。

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use magia_core::filter::{FilterSpec, LayerName};
use magia_core::render::ir_export::{DiffMarkIr, SpellIr};

/// `magia-render` への描画リクエストを JSON 文字列に組む。
/// フィルタ (`.magia` の show/hide/effects) は適用結果 (表示レイヤー集合 +
/// カテゴリ列) へ畳んでから渡す — Vue 側に FilterSpec の解釈を持ち込まない。
pub(crate) fn render_request(
    ir: &SpellIr,
    diff_overlay: Option<&[DiffMarkIr]>,
    spec: &FilterSpec,
) -> Result<String> {
    let mut request = serde_json::json!({ "ir": ir });
    if let Some(marks) = diff_overlay {
        request["diff_overlay"] = serde_json::to_value(marks)?;
    }
    let all = [
        LayerName::ControlFlow,
        LayerName::Effects,
        LayerName::TypeInfo,
    ];
    let visible: Vec<&str> = all
        .into_iter()
        .filter(|layer| spec.is_visible(*layer))
        .map(LayerName::as_str)
        .collect();
    if visible.len() < all.len() {
        request["show_layers"] = serde_json::to_value(&visible)?;
    }
    if let Some(categories) = spec.effect_categories() {
        let names: Vec<&str> = categories.iter().map(|c| c.as_str()).collect();
        request["effects"] = serde_json::to_value(&names)?;
    }
    Ok(request.to_string())
}

/// ベルカ式の描画リクエスト (フィルタはレイヤー語彙が合わないため受けない —
/// CLI が `--style belka` + フィルタ併用を明示エラーにする既存規約のまま)。
pub(crate) fn belka_request(belka: &magia_core::render::belka::BelkaIr) -> String {
    serde_json::json!({ "belka": belka }).to_string()
}

/// `magia-render` のパス解決 (CLAUDE.repo.md・CI と同期):
/// 1) `MAGIA_RENDER_PATH` 環境変数 (明示指定 — 存在しなければエラー)
/// 2) `magia` 実行ファイルと同じディレクトリ (配布形態: 2バイナリ同梱)
/// 3) その親ディレクトリ (開発ビルド: `target/debug/magia` と `target/magia-render`)
/// 4) PATH (Command の解決に委ねる)
fn resolve_render_bin() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("MAGIA_RENDER_PATH") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
        bail!(
            "MAGIA_RENDER_PATH が指すファイルがありません: {}",
            path.display()
        );
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let sibling = dir.join("magia-render");
        if sibling.is_file() {
            return Ok(sibling);
        }
        if let Some(parent) = dir.parent() {
            let dev = parent.join("magia-render");
            if dev.is_file() {
                return Ok(dev);
            }
        }
    }
    Ok(PathBuf::from("magia-render"))
}

/// 配置済み IR (JSON リクエスト) を `magia-render` に渡して SVG 文字列を得る。
/// 失敗時は Vue 側の stderr (スタック込み) をエラーに含める。
pub(crate) fn render_via_ssr(request_json: &str) -> Result<String> {
    let bin = resolve_render_bin()?;
    let mut child = Command::new(&bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| {
            format!(
                "magia-render を起動できません ({})。\n\
                 `cd web && bun run build:render` でビルドするか、\
                 MAGIA_RENDER_PATH 環境変数でパスを指定してください",
                bin.display()
            )
        })?;
    {
        // ブロックで drop して EOF を送ってから待つ (wait_with_output も内部で
        // stdin を閉じるが、意図を読み手に明示する — レビュー W1)。
        let mut stdin = child
            .stdin
            .take()
            .expect("piped stdin は spawn 直後に必ず存在する");
        stdin
            .write_all(request_json.as_bytes())
            .context("magia-render への IR 書き込みに失敗しました")?;
    }
    let output = child
        .wait_with_output()
        .context("magia-render の完了待ちに失敗しました")?;
    if !output.status.success() {
        bail!(
            "magia-render が失敗しました ({}):\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let svg = String::from_utf8(output.stdout).context("magia-render の出力が UTF-8 ではない")?;
    Ok(svg.trim_end().to_string())
}
