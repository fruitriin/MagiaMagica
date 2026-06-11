//! git 連携 (Phase 3.3, spec v0.3 §9.1)。
//!
//! CI ロジックを `magia` コマンド側に寄せるための薄い git ラッパー。
//! ワークフロー YAML からもローカルからも同じコマンドで再現できることが
//! デバッグ可能性の要件 (計画の設計判断)。
//! git バイナリへの依存はこのモジュールに閉じ込める (POSD 情報隠蔽)。

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

/// `file` のリビジョン `rev` 時点の内容を返す。
///
/// パスは現在のインデックス上のリポジトリ相対パスで解決する
/// (リネームを跨ぐ比較は Phase 3 では追わない)。
pub fn show_file_at(rev: &str, file: &Path) -> Result<String> {
    let dir = context_dir(file);
    let relative = repo_relative(file)?;
    show_repo_file(rev, &dir, &relative).with_context(|| {
        format!(
            "リビジョン {rev} に {relative} が見つかりません (新規ファイルの場合は --git は使えません)"
        )
    })
}

/// リポジトリ相対パス `relative` のリビジョン `rev` 時点の内容を返す。
/// `dir` はリポジトリ内の任意のディレクトリでよい。
pub fn show_repo_file(rev: &str, dir: &Path, relative: &str) -> Result<String> {
    git(dir, &["show", &format!("{rev}:{relative}")])
}

/// リビジョン `rev` から作業ツリーまでの間に変更された .rs ファイルを
/// リポジトリルートからの相対パスで返す (決定論的: git の出力順 = パス辞書順)。
pub fn changed_rs_files(rev: &str, cwd: &Path) -> Result<Vec<String>> {
    let listed = git(cwd, &["diff", "--name-only", rev, "--", "*.rs"])?;
    Ok(listed.lines().map(str::to_string).collect())
}

/// リポジトリのルートディレクトリ。
pub fn repo_root(cwd: &Path) -> Result<PathBuf> {
    let root = git(cwd, &["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(root.trim_end()))
}

/// リポジトリルートから見た `file` の相対パス。git 管理外なら案内エラー。
fn repo_relative(file: &Path) -> Result<String> {
    let dir = context_dir(file);
    let file_name = file
        .file_name()
        .with_context(|| format!("ファイル名を取り出せません: {}", file.display()))?;
    let listed = git(
        &dir,
        &[
            "ls-files",
            "--full-name",
            "--",
            &file_name.to_string_lossy(),
        ],
    )?;
    let relative = listed.lines().next().unwrap_or("").trim();
    if relative.is_empty() {
        bail!(
            "{} は git 管理下にありません (git add 済みのファイルのみ --git で比較できます)",
            file.display()
        );
    }
    Ok(relative.to_string())
}

/// git コマンドの実行コンテキストにするディレクトリ (ファイルの親、なければカレント)。
fn context_dir(file: &Path) -> PathBuf {
    file.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// `git -C <dir> <args...>` を実行して stdout を返す。失敗時は stderr を含むエラー。
fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .context("git コマンドを起動できません (git はインストールされていますか?)")?;
    if !output.status.success() {
        bail!(
            "git {} が失敗しました: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    String::from_utf8(output.stdout).context("git の出力が UTF-8 ではありません")
}
