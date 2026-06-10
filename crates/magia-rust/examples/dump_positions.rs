//! レイアウト座標のダンプ (デバッグ・回帰ベースライン採取用)。
//! 実行: `cargo run -p magia-rust --example dump_positions -- <FILE> <FN>`

use magia_core::layout::layout;
use magia_rust::parse_function;

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(file), Some(fn_name)) = (args.next(), args.next()) else {
        eprintln!("usage: dump_positions <FILE> <FN>");
        std::process::exit(2);
    };
    let source = std::fs::read_to_string(&file).expect("read file");
    let graph = parse_function(&source, &fn_name).expect("parse");
    let result = layout(&graph);
    for (id, point) in &result.positions {
        println!("{} {:.2} {:.2}", id.0, point.x, point.y);
    }
}
