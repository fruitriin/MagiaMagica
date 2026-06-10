//! IO 効果 (青) のサンプル。print 系マクロが召喚記号になる。
fn io_print(name: &str) {
    let greeting = format!("Hello, {name}!");
    println!("{greeting}");
    eprintln!("logged");
}
