//! SVG レンダラの統合テスト (Phase 1.6 受け入れ基準のうち IR 直組みで検証できるもの)。
//! parse_function を通した E2E ゴールデンテストは magia-rust 側 (render_golden.rs)。

use magia_core::ir::{
    ConcurrencyInfo, Edge, EdgeKind, EdgeLayerData, LayerData, MagiaGraph, Module, ModuleId,
    Operation, Sigil, SigilId, SigilKind, TypeInfo,
};
use magia_core::layout::layout;
use magia_core::render::{RenderStyle, render};

fn sample_graph() -> MagiaGraph {
    let main = Sigil {
        id: SigilId(0),
        kind: SigilKind::MainRing,
        content: vec![Operation::default(); 3],
        layers: LayerData {
            type_info: Some(TypeInfo {
                signature: Some("async fn fetch(url: &str) -> Result<String, Error>".to_string()),
                returns_result: true,
                returns_option: false,
            }),
            concurrency: Some(ConcurrencyInfo {
                is_async: true,
                await_points: 1,
            }),
            ..LayerData::default()
        },
        ..Sigil::default()
    };
    let glyph = Sigil {
        id: SigilId(1),
        kind: SigilKind::SummonGlyph,
        content: vec![Operation::default()],
        ..Sigil::default()
    };
    MagiaGraph {
        modules: vec![Module {
            id: ModuleId(0),
            name: "demo".to_string(),
            sigils: vec![main, glyph],
            edges: vec![Edge {
                source: SigilId(0),
                target: SigilId(1),
                kind: EdgeKind::ControlFlow,
                cardinality: 1.0,
                layers: EdgeLayerData::default(),
            }],
        }],
        ..MagiaGraph::default()
    }
}

#[test]
fn render_is_deterministic() {
    let graph = sample_graph();
    let placed = layout(&graph);
    let first = render(&graph, &placed, RenderStyle::MidchildaConcentric);
    let second = render(&graph, &placed, RenderStyle::MidchildaConcentric);
    assert_eq!(first, second, "同じ入力からは完全一致の文字列");
}

#[test]
fn three_layers_exist_with_content() {
    let graph = sample_graph();
    let placed = layout(&graph);
    let svg = render(&graph, &placed, RenderStyle::MidchildaConcentric);
    for layer in ["layer-control-flow", "layer-effects", "layer-type-info"] {
        let marker = format!(r#"<g class="{layer}">"#);
        assert!(svg.contains(&marker), "{layer} の <g> が存在する");
    }
    // 各レイヤーに対応要素が入っている。
    assert!(svg.contains("main-ring"));
    assert!(svg.contains("summon-glyph"));
    assert!(svg.contains("signature"));
}

#[test]
fn async_fn_renders_double_ring() {
    let graph = sample_graph();
    let placed = layout(&graph);
    let svg = render(&graph, &placed, RenderStyle::MidchildaConcentric);
    assert!(svg.contains(r#"class="main-ring-async""#));
}

#[test]
fn result_return_renders_branch_lines() {
    let graph = sample_graph();
    let placed = layout(&graph);
    let svg = render(&graph, &placed, RenderStyle::MidchildaConcentric);
    assert!(svg.contains(r#"class="return-path-ok""#));
    assert!(svg.contains(r#"class="return-path-err""#));
}

#[test]
fn signature_is_xml_escaped() {
    let graph = sample_graph();
    let placed = layout(&graph);
    let svg = render(&graph, &placed, RenderStyle::MidchildaConcentric);
    assert!(svg.contains("Result&lt;String, Error&gt;"));
    assert!(svg.contains("&amp;str"));
    assert!(!svg.contains("Result<String"));
}

#[test]
fn svg_root_has_viewbox_from_canvas() {
    let graph = sample_graph();
    let placed = layout(&graph);
    let svg = render(&graph, &placed, RenderStyle::MidchildaConcentric);
    assert!(svg.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox=""#));
    assert!(svg.trim_end().ends_with("</svg>"));
}

#[test]
#[should_panic(expected = "not implemented in Phase 1")]
fn belka_style_is_stubbed() {
    let graph = sample_graph();
    let placed = layout(&graph);
    let _ = render(&graph, &placed, RenderStyle::Belka);
}
