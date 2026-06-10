//! MagiaMagica CLI entry point.
//!
//! Phase 1.0 では no-op。サブコマンドは Phase 1.7 で `clap` ベースで実装する。

fn main() {
    println!("{}", greeting());
}

fn greeting() -> String {
    format!(
        "magia {} — Phase 1.0 bootstrap. Subcommands arrive in Phase 1.7.",
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_mentions_phase() {
        assert!(greeting().contains("Phase 1.0"));
    }
}
