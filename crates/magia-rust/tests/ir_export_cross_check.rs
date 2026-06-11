//! ir_export (Phase 4.0.9) と SVG レンダラの座標クロス検証。
//!
//! 配置済み IR は「midchilda.rs の描画と同一規則」を契約とする (spec v0.3 §16)。
//! 規則のコピーが将来ずれる事故 (4.3 で SVG レンダラを削除するまでの過渡リスク)
//! を、同一 fixture を両経路で出力して座標一致を検証することで防ぐ。

use magia_core::layout::layout;
use magia_core::render::ir_export::spell_ir;
use magia_core::render::{RenderStyle, render};
use magia_rust::parse_function;

/// `class="<class>"` を持つ要素の cx/cy を取り出す (依存を増やさない素朴パース)。
fn extract_centers(svg: &str, class: &str) -> Vec<(f64, f64)> {
    let attr = |chunk: &str, name: &str| -> f64 {
        let start = chunk.find(&format!("{name}=\"")).expect("属性がある") + name.len() + 2;
        let rest = &chunk[start..];
        let end = rest.find('"').expect("属性が閉じる");
        rest[..end].parse().expect("SVG 数値")
    };
    svg.split(&format!("class=\"{class}\""))
        .skip(1)
        .map(|chunk| {
            let chunk = &chunk[..chunk.find("/>").unwrap_or(chunk.len())];
            (attr(chunk, "cx"), attr(chunk, "cy"))
        })
        .collect()
}

#[test]
fn ir_export_matches_svg_renderer_coordinates() {
    // fixture はワークスペースルートに統一 (Phase 2.4 の教訓)。
    let source = include_str!("../../../fixtures/medium_render_doc.rs");
    let graph = parse_function(source, "medium_render_doc").expect("fixture はパースできる");
    let placed = layout(&graph);

    let svg = render(&graph, &placed, RenderStyle::MidchildaConcentric);
    let ir = spell_ir(&graph, &placed);

    // 座標は SVG が2桁丸め・IR が生値のため、両者を2桁表記へ正規化した
    // ソート済みリストで集合一致を見る (丸め前の僅差でソート順が入れ替わるのを防ぐ)。
    let canonical = |points: Vec<(f64, f64)>| -> Vec<String> {
        let mut keys: Vec<String> = points
            .into_iter()
            .map(|(x, y)| format!("{x:.2},{y:.2}"))
            .collect();
        keys.sort();
        keys
    };

    // 操作ドット: SVG の op-dot (cx, cy) と IR の operations が集合一致する。
    let svg_dots = canonical(extract_centers(&svg, "op-dot"));
    let ir_dots = canonical(
        ir.rings
            .iter()
            .flat_map(|ring| ring.operations.iter().map(|op| (op.x, op.y)))
            .collect(),
    );
    assert_eq!(svg_dots, ir_dots, "操作ドット座標が一致する");

    // リング: main-ring / aux-ring の中心が一致する。
    let mut svg_ring_points = extract_centers(&svg, "main-ring");
    svg_ring_points.extend(extract_centers(&svg, "aux-ring"));
    let svg_rings = canonical(svg_ring_points);
    let ir_rings = canonical(ir.rings.iter().map(|r| (r.x, r.y)).collect());
    assert_eq!(svg_rings, ir_rings, "リング中心が一致する");

    // 召喚印・エッジ・戻り値分岐の存在数も一致する (medium_render_doc は
    // Result 戻り値を持つので return_branch が立つ)。
    let svg_glyphs = svg.matches(r#"class="summon-glyph""#).count();
    assert_eq!(svg_glyphs, ir.glyphs.len(), "召喚印数が一致する");
    let svg_edges = svg.matches(r#"class="edge-control-flow""#).count();
    assert_eq!(svg_edges, ir.edges.len(), "接続線数が一致する");
    assert!(svg.contains(r#"class="return-path-ok""#));
    assert!(ir.return_branch.is_some(), "戻り値分岐の生成条件が一致する");
    assert!(svg.contains(r#"class="signature""#));
    assert!(ir.signature.is_some(), "シグネチャの生成条件が一致する");
}
