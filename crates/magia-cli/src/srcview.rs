//! ソースペイン用のシンタックスハイライト (Phase 4.0)。
//!
//! サーバ側で syntect を一度だけ走らせ、SH 済み HTML を /spell 応答に同梱する
//! (クライアント側ハイライタを増やさず素 JS 継続、計画の設計判断)。
//! テーマは GitHub light 相当の `InspiredGitHub` 固定 (切替は Phase 4.6)。

use std::sync::OnceLock;

use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// Rust スニペットを SH 付き HTML (`<pre style="...">` 1ブロック) に変換する。
///
/// 構文セット・テーマの読み込みは初回のみ (OnceLock)。ハイライトに失敗した場合は
/// エスケープ済みの素のコードへフォールバックする (表示を欠けさせない)。
pub(crate) fn highlight_rust(snippet: &str) -> String {
    static ASSETS: OnceLock<(SyntaxSet, Theme)> = OnceLock::new();
    let (syntaxes, theme) = ASSETS.get_or_init(|| {
        let syntaxes = SyntaxSet::load_defaults_newlines();
        let theme = ThemeSet::load_defaults()
            .themes
            .remove("InspiredGitHub")
            .expect("InspiredGitHub は default-themes に含まれる");
        (syntaxes, theme)
    });
    let syntax = syntaxes
        .find_syntax_by_extension("rs")
        .expect("Rust 構文は default-syntaxes に含まれる");
    highlighted_html_for_string(snippet, syntaxes, syntax, theme)
        .unwrap_or_else(|_| format!("<pre>{}</pre>", escape_html(snippet)))
}

/// HTML テキストノード用の最小エスケープ (フォールバック経路のみで使う)。
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_rust_snippet_as_html() {
        let html = highlight_rust("fn main() { println!(\"hi\"); }\n");
        assert!(html.contains("<pre"));
        assert!(html.contains("main"));
        assert!(
            !html.contains("<script"),
            "コードはエスケープされて埋め込まれる"
        );
    }

    #[test]
    fn is_deterministic() {
        let a = highlight_rust("let x = 1;\n");
        let b = highlight_rust("let x = 1;\n");
        assert_eq!(a, b);
    }
}
