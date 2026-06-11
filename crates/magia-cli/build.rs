//! web/dist (Vue SPA) を rust-embed で同梱するための前段ビルド (Phase 4.0.5 M5)。
//!
//! 方針: `web/dist/index.html` が無ければ Bun でフロントエンドをビルドする。
//! Bun が無い環境では「手順つきのエラー」で止める (黙って UI 抜きのバイナリを
//! 作らない — 配布物は常に全機能で完結させる)。前提は CLAUDE.repo.md に記載。

use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("cargo が設定する");
    let web_dir = Path::new(&manifest_dir).join("../../web");
    let dist_index = web_dir.join("dist/index.html");

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

    let fresh = match (dist_mtime(&dist_index), src_mtime(&web_dir)) {
        (Some(dist), Some(src)) => dist >= src,
        (Some(_), None) => true,
        _ => false,
    };
    if fresh {
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
            Command::new("bun")
                .args(["run", "build"])
                .current_dir(&web_dir)
                .status()
        })
        .map(|s| s.success());

    match ok {
        Ok(true) => {}
        Ok(false) => panic!("web/ のフロントエンドビルド (bun run build) が失敗しました"),
        Err(e) => panic!(
            "web/dist がなく、Bun も実行できません ({e})。\n\
             MagiaMagica のビルドには Bun が必要です (Node.js 不採用、CLAUDE.repo.md 参照):\n\
             1. https://bun.sh からインストール\n\
             2. cd web && bun install && bun run build\n\
             3. cargo build を再実行"
        ),
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
