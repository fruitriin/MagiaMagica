//! 自己ホスティング fixture (Phase 1.6 受け入れ基準 (b))。
//!
//! このリポジトリ自身の代表関数を魔法陣として描画し、「綺麗か」をオーナーが
//! 目視判定するための素材を出力する (ドッグフーディング)。
//!
//! 実行: `cargo run -p magia-rust --example render_self`
//! 出力: `target/self-hosting/<関数名>.svg`

use std::fs;
use std::path::Path;

use magia_core::layout::layout;
use magia_core::render::{RenderStyle, render};
use magia_rust::parse_function;

fn main() {
    // (ラベル, ソース, 関数名)。include_str! でコンパイル時に自身のコードを取り込む。
    let targets: &[(&str, &str, &str)] = &[
        (
            "magia-rust/src/lib.rs",
            include_str!("../src/lib.rs"),
            "parse_function",
        ),
        (
            "magia-rust/src/summon.rs",
            include_str!("../src/summon.rs"),
            "collect_calls_in_stmt",
        ),
        (
            "magia-core/src/layout/mod.rs",
            include_str!("../../magia-core/src/layout/mod.rs"),
            "layout_with",
        ),
        (
            "magia-core/src/layout/crossing.rs",
            include_str!("../../magia-core/src/layout/crossing.rs"),
            "count_crossings",
        ),
    ];

    let out_dir = Path::new("target/self-hosting");
    fs::create_dir_all(out_dir).expect("出力ディレクトリを作成できる");

    for (label, source, fn_name) in targets {
        let graph = match parse_function(source, fn_name) {
            Ok(graph) => graph,
            Err(error) => {
                eprintln!("skip {label}::{fn_name}: {error}");
                continue;
            }
        };
        let placed = layout(&graph);
        let svg = render(&graph, &placed, RenderStyle::MidchildaConcentric);
        let path = out_dir.join(format!("{fn_name}.svg"));
        fs::write(&path, svg).expect("SVG を書き出せる");
        println!("{} ({label})", path.display());
    }
}
