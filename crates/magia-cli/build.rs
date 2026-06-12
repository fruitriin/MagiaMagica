//! web/dist (Vue SPA) を rust-embed で同梱するための前段ビルド (Phase 4.0.5 M5) と、
//! 静止画レンダ `magia-render` (Vue SSR の単一実行ファイル) のビルド (Phase 4.3)。
//!
//! 方針: `web/dist/index.html` または `target/magia-render` が古ければ Bun で
//! ビルドする。Bun が無い環境では「手順つきのエラー」で止める (黙って UI 抜きの
//! バイナリを作らない — 配布物は常に全機能で完結させる)。前提は CLAUDE.repo.md に記載。

use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("cargo が設定する");
    let web_dir = Path::new(&manifest_dir).join("../../web");
    let dist_index = web_dir.join("dist/index.html");
    let render_bin = Path::new(&manifest_dir).join("../../target/magia-render");

    // フロントエンドのソース変更で再ビルドする (dist は成果物なので監視しない —
    // 監視すると bun build 自体が再実行ループを起こす)。
    for tracked in [
        "src",
        "index.html",
        "package.json",
        "vite.config.ts",
        "uno.config.ts",
    ] {
        println!(
            "cargo::rerun-if-changed={}",
            web_dir.join(tracked).display()
        );
    }

    // 鮮度判定は rerun-if-changed と二重に見えるが役割が違う: rerun-if-changed は
    // 「build.rs を再実行するか」、mtime 比較は「再実行されたとき bun build を呼ぶか」。
    // CI のように bun build を明示的に先行させた環境では後者がスキップして二重ビルドを防ぐ。
    // src ツリーが読めないのは異常 — 黙って古い成果物を採用しない (センチネル禁止規約)。
    let src = src_mtime(&web_dir)
        .unwrap_or_else(|| panic!("web/src の最終更新時刻を取得できません (web/ の構成を確認)"));
    let fresh = |artifact: &Path| dist_mtime(artifact).is_some_and(|out| out >= src);

    let mut steps: Vec<&[&str]> = Vec::new();
    if !fresh(&dist_index) {
        steps.push(&["run", "build"]);
    }
    // 静止画レンダ (Phase 4.3): vite SSR ビルド + bun build --compile。
    // serve の SPA とは別成果物だが、ソースは同じ web/src を共有する。
    if !fresh(&render_bin) {
        steps.push(&["run", "build:render"]);
    }
    if steps.is_empty() {
        return;
    }

    let ok = Command::new("bun")
        .args(["install", "--frozen-lockfile"])
        .current_dir(&web_dir)
        .status()
        .and_then(|s| {
            if !s.success() {
                return Err(std::io::Error::other("bun install failed"));
            }
            for step in &steps {
                let status = Command::new("bun")
                    .args(*step)
                    .current_dir(&web_dir)
                    .status()?;
                if !status.success() {
                    return Err(std::io::Error::other(format!(
                        "bun {} failed",
                        step.join(" ")
                    )));
                }
            }
            Ok(())
        });

    if let Err(e) = ok {
        panic!(
            "web/ のビルド (bun) に失敗しました ({e})。\n\
             MagiaMagica のビルドには Bun が必要です (Node.js 不採用、CLAUDE.repo.md 参照):\n\
             1. https://bun.sh からインストール\n\
             2. cd web && bun install && bun run build && bun run build:render\n\
             3. cargo build を再実行"
        );
    }
}

fn dist_mtime(dist_index: &Path) -> Option<std::time::SystemTime> {
    dist_index.metadata().and_then(|m| m.modified()).ok()
}

/// web/src 以下と設定ファイルの最新更新時刻 (粗い鮮度判定で十分)。
fn src_mtime(web_dir: &Path) -> Option<std::time::SystemTime> {
    let mut latest = None;
    let mut stack = vec![web_dir.join("src")];
    for name in [
        "index.html",
        "package.json",
        "vite.config.ts",
        "uno.config.ts",
    ] {
        push_mtime(&web_dir.join(name), &mut latest);
    }
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                push_mtime(&path, &mut latest);
            }
        }
    }
    latest
}

fn push_mtime(path: &Path, latest: &mut Option<std::time::SystemTime>) {
    if let Ok(modified) = path.metadata().and_then(|m| m.modified())
        && latest.is_none_or(|current| modified > current)
    {
        *latest = Some(modified);
    }
}
