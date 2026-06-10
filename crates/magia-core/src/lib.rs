//! MagiaMagica core: IR, analysis, layout, and SVG rendering.
//!
//! Phase 1.0 はワークスペース立ち上げのみ。IR スケルトン以降は Phase 1.1 で実装する。

/// クレートの識別子。Phase 1.1 以降で IR モジュールに置き換える。
#[must_use]
pub fn crate_name() -> &'static str {
    "magia-core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_is_magia_core() {
        assert_eq!(crate_name(), "magia-core");
    }
}
