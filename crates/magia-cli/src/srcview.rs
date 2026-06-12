//! ソースペイン用のシンタックスハイライト (Phase 4.0)。
//!
//! サーバ側で syntect を一度だけ走らせ、SH 済み HTML を /spell 応答に同梱する
//! (クライアント側ハイライタを増やさず素 JS 継続、計画の設計判断)。
//! テーマは GitHub light 相当の `InspiredGitHub` 固定 (切替は Phase 4.6)。

use std::sync::OnceLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{
    IncludeBackground, highlighted_html_for_string, styled_line_to_highlighted_html,
};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// 構文セット・テーマの読み込みは初回のみ (OnceLock で全ハイライト経路が共有)。
fn assets() -> &'static (SyntaxSet, Theme) {
    static ASSETS: OnceLock<(SyntaxSet, Theme)> = OnceLock::new();
    ASSETS.get_or_init(|| {
        let syntaxes = SyntaxSet::load_defaults_newlines();
        let theme = ThemeSet::load_defaults()
            .themes
            .remove("InspiredGitHub")
            .expect("InspiredGitHub は default-themes に含まれる");
        (syntaxes, theme)
    })
}

/// Rust スニペットを SH 付き HTML (`<pre style="...">` 1ブロック) に変換する。
///
/// ハイライトに失敗した場合はエスケープ済みの素のコードへフォールバックする
/// (表示を欠けさせない)。
pub(crate) fn highlight_rust(snippet: &str) -> String {
    let (syntaxes, theme) = assets();
    let syntax = syntaxes
        .find_syntax_by_extension("rs")
        .expect("Rust 構文は default-syntaxes に含まれる");
    highlighted_html_for_string(snippet, syntaxes, syntax, theme)
        .unwrap_or_else(|_| format!("<pre>{}</pre>", escape_html(snippet)))
}

/// ファイル全体を**行ごと**の SH 済み HTML 断片に変換する (細部修正 2026-06-12)。
///
/// ソースペインのファイル全体表示が、行番号・フォーカス範囲の強調・行スクロールを
/// クライアント側で行うための形 (1要素 = 1行)。状態は行をまたいで引き継ぐ
/// (複数行コメント・文字列が正しく着色される)。失敗した行はエスケープ素通し。
pub(crate) fn highlight_rust_lines(source: &str) -> Vec<String> {
    let (syntaxes, theme) = assets();
    let syntax = syntaxes
        .find_syntax_by_extension("rs")
        .expect("Rust 構文は default-syntaxes に含まれる");
    let mut highlighter = HighlightLines::new(syntax, theme);
    LinesWithEndings::from(source)
        .map(|line| {
            // パースには改行込みで渡し (構文状態の引き継ぎに必要)、HTML からは
            // 落とす (1行1要素のレイアウトに改行を持ち込まない)。
            highlighter
                .highlight_line(line, syntaxes)
                .ok()
                .and_then(|ranges| {
                    styled_line_to_highlighted_html(&ranges, IncludeBackground::No).ok()
                })
                .unwrap_or_else(|| escape_html(line))
                .replace('\n', "")
        })
        .collect()
}

/// HTML テキストノード用の最小エスケープ (フォールバック経路のみで使う)。
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;") // 属性値への注入も塞ぐ (レビュー W1)
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
    fn highlights_lines_one_element_per_line() {
        let lines = highlight_rust_lines("fn a() {}\n// comment\nlet x = 1;\n");
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("fn"));
        assert!(lines[1].contains("comment"));
        assert!(
            lines.iter().all(|l| !l.contains('\n')),
            "行 HTML に改行は残さない"
        );
    }

    #[test]
    fn is_deterministic() {
        let a = highlight_rust("let x = 1;\n");
        let b = highlight_rust("let x = 1;\n");
        assert_eq!(a, b);
    }
}
