//! レイアウトエンジンの統合テスト (Phase 1.5 受け入れ基準)。

use magia_core::ir::{
    AuxRingKind, AuxRingRole, ControlFlowInfo, Edge, EdgeKind, LayerData, MagiaGraph, Module,
    ModuleId, Operation, Sigil, SigilId, SigilKind,
};
use magia_core::layout::constants::{
    AUX_RING_RADIUS, GLYPH_GAP, MAIN_RING_RADIUS, RING_GAP, SUMMON_GLYPH_RADIUS,
};
use magia_core::layout::{LayoutOptions, layout, layout_with};

fn ring(id: u32, kind: SigilKind, ops: usize) -> Sigil {
    Sigil {
        id: SigilId(id),
        kind,
        content: vec![Operation::default(); ops],
        ..Sigil::default()
    }
}

fn aux_ring(id: u32, ops: usize, anchor: u32, ordinal: u32) -> Sigil {
    let mut sigil = ring(id, SigilKind::AuxRing, ops);
    sigil.layers = LayerData {
        control_flow: Some(ControlFlowInfo {
            role: Some(AuxRingRole {
                kind: AuxRingKind::IfBranch,
                anchor_operation: anchor,
                ordinal,
                label: None,
                guard_location: None,
            }),
            ..ControlFlowInfo::default()
        }),
        ..LayerData::default()
    };
    sigil
}

fn edge(source: u32, target: u32) -> Edge {
    Edge {
        source: SigilId(source),
        target: SigilId(target),
        kind: EdgeKind::ControlFlow,
        cardinality: 1.0,
        layers: magia_core::ir::EdgeLayerData::default(),
    }
}

fn graph_of(sigils: Vec<Sigil>, edges: Vec<Edge>) -> MagiaGraph {
    MagiaGraph {
        modules: vec![Module {
            id: ModuleId(0),
            name: "test".to_string(),
            sigils,
            edges,
        }],
        ..MagiaGraph::default()
    }
}

/// MainRing(0) + AuxRing(1,2) + 入れ子 AuxRing(3) + glyph(4,5,6) の複合グラフ。
fn complex_graph() -> MagiaGraph {
    graph_of(
        vec![
            ring(0, SigilKind::MainRing, 4),
            aux_ring(1, 2, 0, 0),
            aux_ring(2, 1, 0, 1),
            aux_ring(3, 1, 1, 0), // 1 の子 (入れ子)
            ring(4, SigilKind::SummonGlyph, 1),
            ring(5, SigilKind::SummonGlyph, 1),
            ring(6, SigilKind::SummonGlyph, 1),
        ],
        vec![
            edge(0, 1),
            edge(0, 2),
            edge(1, 3),
            edge(0, 4),
            edge(0, 5),
            edge(1, 6),
        ],
    )
}

fn distance(a: kurbo::Point, b: kurbo::Point) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

#[test]
fn layout_is_deterministic_over_ten_runs() {
    let graph = complex_graph();
    let first = layout(&graph);
    for _ in 0..9 {
        assert_eq!(layout(&graph), first, "同じ IR からは常に同一の結果");
    }
}

#[test]
fn main_ring_is_at_origin() {
    let graph = complex_graph();
    let result = layout(&graph);
    let main = result.positions[&SigilId(0)];
    assert_eq!((main.x, main.y), (0.0, 0.0));
}

#[test]
fn aux_ring_sits_on_expected_radius() {
    let graph = complex_graph();
    let result = layout(&graph);
    let expected = MAIN_RING_RADIUS + RING_GAP + AUX_RING_RADIUS;
    for aux in [SigilId(1), SigilId(2)] {
        let d = distance(result.positions[&aux], result.positions[&SigilId(0)]);
        assert!(
            (d - expected).abs() < 1e-9,
            "AuxRing {aux:?} は MainRing から想定半径 {expected} 上 (実測 {d})"
        );
    }
}

#[test]
fn nested_aux_ring_is_positioned_from_its_parent() {
    let graph = complex_graph();
    let result = layout(&graph);
    let expected = AUX_RING_RADIUS + RING_GAP + AUX_RING_RADIUS;
    let d = distance(result.positions[&SigilId(3)], result.positions[&SigilId(1)]);
    assert!(
        (d - expected).abs() < 1e-9,
        "入れ子 AuxRing は親 AuxRing 基準"
    );
}

#[test]
fn glyphs_zero_one_five_do_not_break() {
    for count in [0usize, 1, 5] {
        let mut sigils = vec![ring(0, SigilKind::MainRing, 2)];
        let mut edges = Vec::new();
        for i in 0..count {
            let id = u32::try_from(i).unwrap() + 1;
            sigils.push(ring(id, SigilKind::SummonGlyph, 1));
            edges.push(edge(0, id));
        }
        let graph = graph_of(sigils, edges);
        // 最適化 ON で実行する。距離は rotation に依らず一定なので、
        // 以下の距離アサーションは最適化が動いても成立する (方向は検証しない)。
        let result = layout(&graph);
        // 全 Sigil に位置がある。
        assert_eq!(result.positions.len(), count + 1);
        // glyph は親リングから一定距離。
        let expected = MAIN_RING_RADIUS + GLYPH_GAP + SUMMON_GLYPH_RADIUS;
        let origin = result.positions[&SigilId(0)];
        for i in 0..count {
            let id = SigilId(u32::try_from(i).unwrap() + 1);
            let d = distance(result.positions[&id], origin);
            assert!((d - expected).abs() < 1e-9);
        }
        // 複数 glyph は相異なる位置 (放射状等間隔)。
        if count >= 2 {
            let mut points: Vec<_> = (1..=count)
                .map(|i| result.positions[&SigilId(u32::try_from(i).unwrap())])
                .collect();
            points.sort_by(|a, b| (a.x, a.y).partial_cmp(&(b.x, b.y)).unwrap());
            points.dedup_by(|a, b| distance(*a, *b) < 1e-9);
            assert_eq!(points.len(), count, "glyph は重ならない");
        }
    }
}

#[test]
fn canvas_contains_all_sigil_circles() {
    let graph = complex_graph();
    let result = layout(&graph);
    for point in result.positions.values() {
        assert!(
            result.canvas.contains(*point),
            "{point:?} が canvas {:?} の外",
            result.canvas
        );
    }
    // マージン分の余裕がある (中心点ぴったりが境界ではない)。
    assert!(result.canvas.width() > 2.0 * MAIN_RING_RADIUS);
}

#[test]
fn optimization_can_be_toggled_and_both_are_deterministic() {
    let graph = complex_graph();
    let on = layout_with(
        &graph,
        LayoutOptions {
            minimize_crossings: true,
        },
    );
    let off = layout_with(
        &graph,
        LayoutOptions {
            minimize_crossings: false,
        },
    );
    assert_eq!(
        on,
        layout_with(
            &graph,
            LayoutOptions {
                minimize_crossings: true
            }
        )
    );
    assert_eq!(
        off,
        layout_with(
            &graph,
            LayoutOptions {
                minimize_crossings: false
            }
        )
    );
    // OFF でも全 Sigil が配置される。
    assert_eq!(off.positions.len(), graph.modules[0].sigils.len());
}

#[test]
fn empty_graph_yields_empty_layout() {
    let result = layout(&MagiaGraph::default());
    assert!(result.positions.is_empty());
    assert_eq!(result.canvas, kurbo::Rect::ZERO);
}

#[test]
fn module_without_main_ring_falls_back_to_origin() {
    // 防御的ケース: MainRing 欠落でも positions は全 Sigil を覆う。
    let graph = graph_of(vec![ring(0, SigilKind::AuxRing, 1)], Vec::new());
    let result = layout(&graph);
    assert_eq!(result.positions[&SigilId(0)], kurbo::Point::ZERO);
}

// ===== Phase 1.8: 衝突回避 =====

fn radius_of(kind: SigilKind) -> f64 {
    match kind {
        SigilKind::MainRing => MAIN_RING_RADIUS,
        SigilKind::AuxRing => AUX_RING_RADIUS,
        SigilKind::SummonGlyph | SigilKind::GateGlyph => SUMMON_GLYPH_RADIUS,
    }
}

type KindFilter = fn(SigilKind) -> bool;

/// 重なっている (中心間距離 < 半径和) Sigil ペアを列挙する。
fn overlapping_pairs(
    graph: &MagiaGraph,
    result: &magia_core::layout::LayoutResult,
    kinds: (KindFilter, KindFilter),
) -> Vec<(SigilId, SigilId)> {
    let sigils = &graph.modules[0].sigils;
    let mut pairs = Vec::new();
    for (i, a) in sigils.iter().enumerate() {
        for b in &sigils[i + 1..] {
            let matches =
                (kinds.0(a.kind) && kinds.1(b.kind)) || (kinds.1(a.kind) && kinds.0(b.kind));
            if !matches {
                continue;
            }
            let pa = result.positions[&a.id];
            let pb = result.positions[&b.id];
            let dist = ((pa.x - pb.x).powi(2) + (pa.y - pb.y).powi(2)).sqrt();
            if dist + 1e-6 < radius_of(a.kind) + radius_of(b.kind) {
                pairs.push((a.id, b.id));
            }
        }
    }
    pairs
}

fn is_ring(kind: SigilKind) -> bool {
    matches!(kind, SigilKind::MainRing | SigilKind::AuxRing)
}

fn is_glyph(kind: SigilKind) -> bool {
    matches!(kind, SigilKind::SummonGlyph | SigilKind::GateGlyph)
}

/// match 6アーム級の過密グラフ: main(content 6) + 同一 anchor のアーム6個
/// (各アームに入れ子1個 + glyph 1個) + main 直下の glyph 4個。
fn dense_graph() -> MagiaGraph {
    let mut sigils = vec![ring(0, SigilKind::MainRing, 6)];
    let mut edges = Vec::new();
    let mut next_id = 1u32;
    for ordinal in 0..6u32 {
        let arm = next_id;
        sigils.push(aux_ring(arm, 2, 0, ordinal));
        edges.push(edge(0, arm));
        next_id += 1;
        // 入れ子
        sigils.push(aux_ring(next_id, 1, 0, 0));
        edges.push(edge(arm, next_id));
        next_id += 1;
        // アームの glyph
        sigils.push(ring(next_id, SigilKind::SummonGlyph, 1));
        edges.push(edge(arm, next_id));
        next_id += 1;
    }
    for _ in 0..4 {
        sigils.push(ring(next_id, SigilKind::SummonGlyph, 1));
        edges.push(edge(0, next_id));
        next_id += 1;
    }
    graph_of(sigils, edges)
}

#[test]
fn dense_graph_has_no_ring_ring_overlaps() {
    let graph = dense_graph();
    let result = layout(&graph);
    let pairs = overlapping_pairs(&graph, &result, (is_ring, is_ring));
    assert!(pairs.is_empty(), "リング重なり: {pairs:?}");
}

#[test]
fn dense_graph_has_no_glyph_ring_overlaps() {
    let graph = dense_graph();
    let result = layout(&graph);
    let pairs = overlapping_pairs(&graph, &result, (is_glyph, is_ring));
    assert!(pairs.is_empty(), "glyph とリングの重なり: {pairs:?}");
}

#[test]
fn dense_graph_layout_is_deterministic() {
    let graph = dense_graph();
    let first = layout(&graph);
    for _ in 0..9 {
        assert_eq!(layout(&graph), first);
    }
}

#[test]
fn overflowing_children_move_to_second_orbit() {
    // 軌道容量を超える数の子は距離を伸ばした第2軌道に送られる。
    let mut sigils = vec![ring(0, SigilKind::MainRing, 14)];
    let mut edges = Vec::new();
    for i in 0..14u32 {
        sigils.push(aux_ring(i + 1, 1, i, 0));
        edges.push(edge(0, i + 1));
    }
    let graph = graph_of(sigils, edges);
    let result = layout(&graph);

    let first_orbit = MAIN_RING_RADIUS + RING_GAP + AUX_RING_RADIUS;
    let origin = result.positions[&SigilId(0)];
    let mut beyond_first = 0;
    for i in 1..=14u32 {
        let p = result.positions[&SigilId(i)];
        let d = ((p.x - origin.x).powi(2) + (p.y - origin.y).powi(2)).sqrt();
        if d > first_orbit + 1.0 {
            beyond_first += 1;
        }
    }
    assert!(beyond_first > 0, "容量超過分が第2軌道に送られる");
    // 重なりも起きない。
    assert!(
        overlapping_pairs(&graph, &result, (is_ring, is_ring)).is_empty(),
        "第2軌道送り後もリング重なりなし"
    );
}
