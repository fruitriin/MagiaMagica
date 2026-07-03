//! MagiaMagica WASM デモ版コア (Phase 4.12)。
//!
//! ブラウザ単体で「ソース文字列 → 魔法陣 IR (JSON)」を完結させる薄ラッパー。
//! 静的ホスティング (Vercel / Cloudflare Pages 等) に置くデモ版 (`magia-hobby`) の
//! WASM 側がこれ。フル機能 (ライブ追従・diff・俯瞰・呼び出しジャンプ) は `magia serve`
//! の領分で、本クレートは **list + 素の spell** に絞る (計画 (a) 案)。
//!
//! 設計の核 (計画 Phase 4.12「薄さを保つ罠」):
//! - パイプラインの本体は `magia-core` / `magia-rust` に既にある (`list_functions` /
//!   `parse_function` / `layout` / `spell_ir` / `transcribe` / `belka_ir`)。本クレートは
//!   それらを呼んで JSON 化するだけ — ロジックを再実装しない。
//! - `serve.rs` のリッチ機能組み立て (neighbors / excerpts / diff / syntect ハイライト) は
//!   再利用しない。再利用したくなったら共有レイヤーへ抽出して serve も hobby も薄くする
//!   ((b) 案) — その前にコピーすると drift を生む。
//! - JSON in / JSON out の境界は serve の HTTP 契約 (`/state` の functions, `/spell/` の
//!   `ir` / `transcript` / `belka_ir`) のサブセットに揃える。Vue 側の描画・パーサを無修正で
//!   流用できる。

use magia_core::layout::layout;
use magia_core::render::belka::belka_ir;
use magia_core::render::ir_export::{SpellResponseBase, spell_ir};
use magia_core::transcript::transcribe;
use magia_rust::{FunctionEntry, function_index, parse_function_with_entry};

/// ファイル内の関数一覧 (JSON 文字列) を返す。
///
/// 要素の形は `FunctionEntry::summary_json` — serve `/state` の `functions` と
/// 同一実装の共有契約 (Phase 4.12 レビューで一点化)。
/// パース失敗 (構文エラー等) は `Err` にメッセージを返す — UI 側で案内に畳む。
///
/// # Errors
/// ソースが Rust として解釈できない場合、または JSON 直列化に失敗した場合。
pub fn list_json(source: &str) -> Result<String, String> {
    let entries = function_index(source).map_err(|e| e.to_string())?;
    let functions: Vec<_> = entries.iter().map(FunctionEntry::summary_json).collect();
    serde_json::to_string(&serde_json::json!({ "functions": functions })).map_err(|e| e.to_string())
}

/// 関数1つを魔法陣 IR (JSON 文字列) に変換する。
///
/// `fn_name` は `list_json` が返す `qualified` (`Foo::bar`) か素の名前。形は
/// `SpellResponseBase` — serve `/spell/` の共通部と同一 struct の共有契約で、
/// neighbors / excerpts / diff / source_html はデモでは出さない (計画 (a) 案)。
/// パースは `parse_function_with_entry` の1回だけ (wasm の syn は遅いので効く)。
///
/// # Errors
/// 関数が見つからない・構文エラー・JSON 直列化失敗のいずれか。
pub fn spell_json(source: &str, fn_name: &str) -> Result<String, String> {
    let (entry, graph) = parse_function_with_entry(source, fn_name).map_err(|e| e.to_string())?;
    let placed = layout(&graph);
    let base = SpellResponseBase {
        ir: spell_ir(&graph, &placed),
        belka_ir: belka_ir(&graph),
        transcript: transcribe(&graph),
        qualified: entry.qualified,
        signature: entry.signature,
        start_line: entry.start_line,
    };
    serde_json::to_string(&base).map_err(|e| e.to_string())
}

/// WASM (ブラウザ) 向けエクスポート。`&str` を受け `String` (JSON) を返す薄い境界。
/// エラーは `JsError` に変換して JS の例外として投げる (UI 側で catch して案内に畳む)。
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn list(source: &str) -> Result<String, JsError> {
        super::list_json(source).map_err(|e| JsError::new(&e))
    }

    #[wasm_bindgen]
    pub fn spell(source: &str, fn_name: &str) -> Result<String, JsError> {
        super::spell_json(source, fn_name).map_err(|e| JsError::new(&e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
        fn greet(name: &str) -> String {
            if name.is_empty() {
                return String::from("hello");
            }
            format!("hello, {name}")
        }

        struct Counter { n: u32 }
        impl Counter {
            fn bump(&mut self) {
                self.n += 1;
            }
        }
    "#;

    #[test]
    fn list_json_finds_free_fn_and_method() {
        let json = list_json(SAMPLE).expect("構文は正しい");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let names: Vec<&str> = value["functions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["qualified"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"greet"), "free fn が出る: {names:?}");
        assert!(
            names.contains(&"Counter::bump"),
            "メソッドが出る: {names:?}"
        );
    }

    #[test]
    fn spell_json_has_renderable_ir() {
        let json = spell_json(SAMPLE, "greet").expect("greet は存在する");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["qualified"], "greet");
        // SpellIr は描画に必須の view_box と rings を持つ (serve `/spell/` の ir 契約)。
        assert!(value["ir"]["view_box"].is_array(), "view_box が要る");
        assert!(value["ir"]["rings"].is_array(), "rings が要る");
        assert!(value["transcript"].is_string(), "書き起こしが要る");
        assert!(value["belka_ir"].is_object(), "belka_ir が要る");
    }

    #[test]
    fn spell_json_resolves_qualified_method() {
        let json = spell_json(SAMPLE, "Counter::bump").expect("メソッドも解決できる");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["qualified"], "Counter::bump");
    }

    #[test]
    fn spell_json_resolves_bare_method_name() {
        // 手打ちの共有リンク (`fn=bump`) 相当 — 素名でも qualified に解決される。
        let json = spell_json(SAMPLE, "bump").expect("素名フォールバックが効く");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["qualified"], "Counter::bump");
    }

    #[test]
    fn errors_are_messages_not_panics() {
        assert!(list_json("fn broken( {").is_err(), "構文エラーは Err");
        assert!(
            spell_json(SAMPLE, "nonexistent").is_err(),
            "未知の関数は Err"
        );
    }
}
