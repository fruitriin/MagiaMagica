//! 効果カテゴリ判定のヒューリスティック表 (Phase 1.4, spec §5.2 / §6.1.3)。
//!
//! crate 名先頭セグメントの近似判定 (tech-selection §2.1 Phase 1a)。意味解決はしない。
//! 表は**小さく始めて段階拡張**する方針 (計画の設計判断)。実コードを通したときに
//! 違和感が強いものから随時追加する。

use magia_core::ir::EffectSet;

/// パス前方一致 → 効果カテゴリの対応表。
///
/// マッチは「セグメント境界つき前方一致」(`std::io` は `std::io::stdin` に一致するが
/// `std::iox` には一致しない)。先に一致した行が勝つため、`tokio::net` のような
/// 長い前置詞をワイルドな前置詞より上に置くこと。
// TODO: `tokio::io` は未追加 (tokio::net / tokio::fs と非対称)。実コードで
// false negative が目立ったら追加する (「小さく始める」方針による意図的な見送り)。
const PATH_EFFECTS: &[(&str, Category)] = &[
    ("std::io", Category::Io),
    ("std::net", Category::Network),
    ("std::fs", Category::Filesystem),
    ("tokio::net", Category::Network),
    ("tokio::fs", Category::Filesystem),
    ("reqwest", Category::Network),
    ("hyper", Category::Network),
    ("sqlx", Category::Db),
    ("diesel", Category::Db),
    ("rusqlite", Category::Db),
];

/// io 効果と見なすマクロ名の白リスト (`!` なし)。
///
/// `format!` は引数1つなら pure と見るのが妥当だが、Phase 1 は io 側に倒して様子見する
/// (オーナー確定 2026-06-10)。
const IO_MACROS: &[&str] = &[
    "println", "eprintln", "print", "eprint", "dbg", "format", "write", "writeln",
];

#[derive(Clone, Copy)]
enum Category {
    Io,
    Network,
    Db,
    Filesystem,
}

/// 呼び出し先パスから `EffectSet` を推定する。未知のパスは `pure` 扱い
/// (解決失敗のサイレント pure 方針、計画の設計判断)。
pub(crate) fn effects_for_path(path: &str) -> EffectSet {
    for (prefix, category) in PATH_EFFECTS {
        if matches_segment_prefix(path, prefix) {
            return category_set(*category);
        }
    }
    pure_set()
}

/// マクロ名 (`!` なし) から `EffectSet` を推定する。白リスト外は `pure` 扱い。
pub(crate) fn effects_for_macro(name: &str) -> EffectSet {
    if IO_MACROS.contains(&name) {
        category_set(Category::Io)
    } else {
        pure_set()
    }
}

/// セグメント境界を考慮した前方一致。
fn matches_segment_prefix(path: &str, prefix: &str) -> bool {
    path.strip_prefix(prefix)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with("::"))
}

fn category_set(category: Category) -> EffectSet {
    let mut set = EffectSet::default();
    match category {
        Category::Io => set.io = true,
        Category::Network => set.network = true,
        Category::Db => set.db = true,
        Category::Filesystem => set.filesystem = true,
    }
    set
}

fn pure_set() -> EffectSet {
    EffectSet {
        pure: true,
        ..EffectSet::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_fs_is_filesystem() {
        let set = effects_for_path("std::fs::read_to_string");
        assert!(set.filesystem);
        assert!(!set.pure);
    }

    #[test]
    fn reqwest_is_network() {
        assert!(effects_for_path("reqwest::get").network);
    }

    #[test]
    fn tokio_subpaths_split_by_category() {
        assert!(effects_for_path("tokio::net::TcpStream::connect").network);
        assert!(effects_for_path("tokio::fs::read").filesystem);
        // tokio それ自体は表にない → pure (spawn 等は効果なし扱い)。
        assert!(effects_for_path("tokio::spawn").pure);
    }

    #[test]
    fn segment_boundary_is_respected() {
        // `std::io` は `std::iox::...` に一致してはならない。
        assert!(effects_for_path("std::iox::fake").pure);
        assert!(effects_for_path("std::io").io);
    }

    #[test]
    fn unknown_path_is_pure() {
        let set = effects_for_path("my_helper");
        assert!(set.pure);
        assert!(!set.io && !set.network && !set.db && !set.filesystem);
    }

    #[test]
    fn print_macros_are_io() {
        assert!(effects_for_macro("println").io);
        assert!(effects_for_macro("dbg").io);
        assert!(effects_for_macro("format").io, "Phase 1 は io 側に倒す");
    }

    #[test]
    fn unknown_macro_is_pure() {
        assert!(effects_for_macro("vec").pure);
    }
}
