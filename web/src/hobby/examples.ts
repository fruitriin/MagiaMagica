// デモの初回 paint で空フォームでなく魔法陣を出すための prefill 例 (Phase 4.12 M3 先取り)。
// 自己ホスト感のある、制御フロー + 効果 + メソッドチェーンが一目で映える短い関数。

export const PREFILL_SOURCE = `/// 受け取ったログ行から ERROR を数えて要約する。
fn summarize_errors(lines: &[String]) -> String {
    let mut errors = 0;
    for line in lines {
        if line.contains("ERROR") {
            errors += 1;
            eprintln!("found: {line}");
        }
    }
    if errors == 0 {
        return String::from("no errors");
    }
    format!("{errors} error(s) in {} lines", lines.len())
}
`;

/** prefill 時に最初に開く関数 (qualified)。 */
export const PREFILL_FN = "summarize_errors";
