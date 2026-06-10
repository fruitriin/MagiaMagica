//! ファイルシステム効果 (茶) のサンプル。
fn filesystem_read(path: &str) -> String {
    let raw = std::fs::read_to_string(path);
    normalize(raw)
}
