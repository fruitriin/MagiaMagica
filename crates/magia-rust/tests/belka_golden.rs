//! ベルカ式の配置済み IR のテスト (Phase 3.5 起源、Phase 4.3 M5 で IR ベースへ移行)。
//!
//! SVG 文字列の検証は Vue SSR 側 (web の vitest / cli 統合テスト) が担う —
//! ここでは射影 (三極分類) と三角配置の構造・決定論を見る (spec v0.3 §14)。

use magia_core::render::belka::belka_ir;
use magia_rust::parse_function;

const LOOP_ACCUMULATE: &str = include_str!("../../../fixtures/loop_accumulate.rs");

fn belka_json(source: &str, fn_name: &str) -> serde_json::Value {
    let graph = parse_function(source, fn_name).expect("fixture は必ずパースできる");
    serde_json::to_value(belka_ir(&graph)).expect("BelkaIr は serialize できる")
}

#[test]
fn belka_ir_has_three_poles_with_fields() {
    let ir = belka_json(LOOP_ACCUMULATE, "loop_accumulate");
    let poles = ir["poles"].as_array().unwrap();
    assert_eq!(poles.len(), 3);
    let kinds: Vec<&str> = poles.iter().map(|p| p["pole"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["genesis", "transmute", "consume"]);
    for pole in poles {
        // 力場は極円より外まで届く (フロー量に応じた濃淡の土台)。
        assert!(pole["field_radius"].as_f64().unwrap() > pole["radius"].as_f64().unwrap());
        // ラベルは円の外側 (中心から離れた位置)。
        assert!(pole["label_x"].is_number() && pole["label_y"].is_number());
    }
    assert_eq!(ir["view_box"].as_array().unwrap().len(), 4);
    assert!(
        ir["signature"]["text"]
            .as_str()
            .unwrap()
            .contains("loop_accumulate")
    );
}

#[test]
fn belka_flow_follows_execution_order() {
    // loop_accumulate: items (生成) → ループで変換 → 末尾の total が消費へ。
    // 実行順走査により「変換 → 消費」の還流が出る (深さ優先だと取り逃がす形)。
    let ir = belka_json(LOOP_ACCUMULATE, "loop_accumulate");
    let flows = ir["flows"].as_array().unwrap();
    assert!(
        flows.len() >= 2,
        "生成→変換 と 変換→消費 の2本以上: {}",
        flows.len()
    );
    for flow in flows {
        assert!(flow["width"].as_f64().unwrap() >= 1.0);
        assert!(flow["tip_x"].is_number() && flow["tip_y"].is_number());
    }
}

#[test]
fn belka_ir_is_deterministic() {
    let first = belka_json(LOOP_ACCUMULATE, "loop_accumulate");
    for _ in 0..4 {
        assert_eq!(belka_json(LOOP_ACCUMULATE, "loop_accumulate"), first);
    }
}

#[test]
fn reducer_fixture_is_flagged_in_type_info() {
    let graph = parse_function(
        include_str!("../../../fixtures/reduce_brightness.rs"),
        "reduce_brightness",
    )
    .expect("fixture は必ずパースできる");
    let main = &graph.modules[0].sigils[0];
    assert!(
        main.layers
            .type_info
            .as_ref()
            .is_some_and(|t| t.reducer_shape),
        "(A, B) -> A は Reducer 形として検出される"
    );
}
