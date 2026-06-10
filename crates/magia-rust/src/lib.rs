//! MagiaMagica Rust language adapter.
//!
//! Phase 1.0 はワークスペース立ち上げのみ。syn による AST → IR 変換は Phase 1.2 で実装する。

/// クレートの識別子。Phase 1.2 以降で `parse_function` などに置き換える。
#[must_use]
pub fn crate_name() -> &'static str {
    "magia-rust"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_is_magia_rust() {
        assert_eq!(crate_name(), "magia-rust");
    }
}
