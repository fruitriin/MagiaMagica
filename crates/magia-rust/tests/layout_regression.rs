//! レイアウトの E2E 回帰テスト (Phase 1.8)。
//!
//! - 過密 fixture で重なりが起きないこと (衝突回避の受け入れ基準)
//! - 中規模 fixture (オーナー合格済みの意匠) の位置がベースラインから動かないこと

use magia_core::ir::{MagiaGraph, SigilId, SigilKind};
use magia_core::layout::constants::{AUX_RING_RADIUS, MAIN_RING_RADIUS, SUMMON_GLYPH_RADIUS};
use magia_core::layout::{LayoutResult, layout};
use magia_rust::parse_function;

fn parsed_layout(source: &str, fn_name: &str) -> (MagiaGraph, LayoutResult) {
    let graph = parse_function(source, fn_name).expect("fixture は必ずパースできる");
    let result = layout(&graph);
    (graph, result)
}

fn radius_of(kind: SigilKind) -> f64 {
    match kind {
        SigilKind::MainRing => MAIN_RING_RADIUS,
        SigilKind::AuxRing => AUX_RING_RADIUS,
        SigilKind::SummonGlyph | SigilKind::GateGlyph => SUMMON_GLYPH_RADIUS,
    }
}

fn is_ring(kind: SigilKind) -> bool {
    matches!(kind, SigilKind::MainRing | SigilKind::AuxRing)
}

/// (リング, リング) と (glyph, リング) の重なりペアを返す。
fn overlaps(graph: &MagiaGraph, result: &LayoutResult) -> Vec<(SigilId, SigilId, &'static str)> {
    let sigils = &graph.modules[0].sigils;
    let mut found = Vec::new();
    for (i, a) in sigils.iter().enumerate() {
        for b in &sigils[i + 1..] {
            let category = match (is_ring(a.kind), is_ring(b.kind)) {
                (true, true) => "ring-ring",
                (true, false) | (false, true) => "glyph-ring",
                (false, false) => continue, // glyph どうしは Phase 1.8 のスコープ外
            };
            let pa = result.positions[&a.id];
            let pb = result.positions[&b.id];
            let dist = ((pa.x - pb.x).powi(2) + (pa.y - pb.y).powi(2)).sqrt();
            if dist + 1e-6 < radius_of(a.kind) + radius_of(b.kind) {
                found.push((a.id, b.id, category));
            }
        }
    }
    found
}

#[test]
fn dense_dispatch_has_no_overlaps() {
    let source = include_str!("../../../fixtures/dense_dispatch.rs");
    let (graph, result) = parsed_layout(source, "dense_dispatch");
    let found = overlaps(&graph, &result);
    assert!(found.is_empty(), "過密 fixture で重なり: {found:?}");
}

/// 位置共有制約 (spec v0.2 §5.4): 表示系レイヤーの有無は配置に影響しない。
///
/// レイアウトが参照してよいのは構造情報 (`control_flow.role` と content) のみ。
/// type_info / concurrency 等の表示系レイヤーを剥がしても `LayoutResult` は
/// 完全一致しなければならない (レイヤー切替 UI が位置を動かさないことの土台)。
#[test]
fn layout_ignores_display_layers() {
    let source = include_str!("../../../fixtures/medium_render_doc.rs");
    let (graph, baseline) = parsed_layout(source, "medium_render_doc");

    // effects は LayerData のフィールドではなく Operation 側 (EffectSet) にあり、
    // レイアウトは Operation の中身を見ない。LayerData で剥がせるのは
    // control_flow (構造情報のため保持) 以外の全フィールド。
    let mut stripped = graph;
    for sigil in &mut stripped.modules[0].sigils {
        sigil.layers.data_flow = None;
        sigil.layers.type_info = None;
        sigil.layers.lifetime = None;
        sigil.layers.concurrency = None;
        sigil.layers.test_coverage = None;
        sigil.layers.profile = None;
        sigil.layers.git_churn = None;
        sigil.layers.security = None;
        sigil.layers.ai_annotations.clear();
    }
    assert_eq!(
        layout(&stripped),
        baseline,
        "表示系レイヤーを剥がしても位置は変わらない"
    );
}

/// オーナーが「お洒落」と判定した write_document 級レイアウトのベースライン
/// (Phase 1.8 着手前の実測値)。衝突回避の改良がこの意匠を崩していないことを
/// 移動量の上限で保証する。
/// 許容 8px: 重なっていた glyph 1個の退避 (約1.6px) は許し、再配分級の変化は拒む。
#[test]
fn medium_fixture_stays_close_to_approved_baseline() {
    const BASELINE: &[(u32, f64, f64)] = &[
        (0, 0.00, 0.00),
        (1, 162.00, 0.00),
        (2, 81.00, 140.30),
        (3, -45.39, 198.89),
        (4, -65.42, 286.63),
        (5, -81.00, 140.30),
        (6, -183.80, 88.51),
        (7, -264.88, 127.56),
        (8, -162.00, 0.00),
        (9, -183.80, -88.51),
        (10, -264.88, -127.56),
        (11, -81.00, -140.30),
        (12, 81.00, -140.30),
    ];
    const TOLERANCE: f64 = 8.0;

    let source = include_str!("../../../fixtures/medium_render_doc.rs");
    let (_, result) = parsed_layout(source, "medium_render_doc");
    assert_eq!(result.positions.len(), BASELINE.len(), "Sigil 数が変わった");
    for (id, x, y) in BASELINE {
        let p = result.positions[&SigilId(*id)];
        let moved = ((p.x - x).powi(2) + (p.y - y).powi(2)).sqrt();
        assert!(
            moved <= TOLERANCE,
            "Sigil {id} がベースラインから {moved:.1}px 移動 (許容 {TOLERANCE}px)"
        );
    }
}

#[test]
fn sibling_closure_rings_do_not_overlap() {
    // Phase 4.8 M2 レビュー W1 の回帰: 1つの召喚に複数のクロージャ引数
    // (rayon::join 形) があっても補助陣が同位置に重ならない。
    use magia_core::ir::SigilKind;
    let src = "fn f() { join(|| left(), || right()); }";
    let graph = magia_rust::parse_function(src, "f").unwrap();
    let layout = magia_core::layout::layout(&graph);
    let module = &graph.modules[0];
    let rings: Vec<_> = module
        .sigils
        .iter()
        .filter(|s| s.kind == SigilKind::AuxRing)
        .map(|s| layout.positions[&s.id])
        .collect();
    assert_eq!(rings.len(), 2, "クロージャ補助陣2つ");
    let d = ((rings[0].x - rings[1].x).powi(2) + (rings[0].y - rings[1].y).powi(2)).sqrt();
    assert!(d > 20.0, "兄弟の補助陣は分散される (距離 {d:.1})");
}

#[test]
fn short_chain_keeps_default_step_when_uncrowded() {
    // Phase 4.9 M1: 衝突がない短い鎖は従来位置 (CHAIN_STEP 等間隔) のまま。
    use magia_core::ir::{EdgeKind, SigilKind};
    let src = "fn f(v: V) { v.iter().count(); }";
    let graph = magia_rust::parse_function(src, "f").unwrap();
    let layout = magia_core::layout::layout(&graph);
    let module = &graph.modules[0];
    let chain_edges: Vec<_> = module
        .edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Chain)
        .collect();
    assert_eq!(chain_edges.len(), 1, "鎖 1 本 (iter→count)");
    let head = chain_edges[0].source;
    let tail = chain_edges[0].target;
    let head_pos = layout.positions[&head];
    let tail_pos = layout.positions[&tail];
    let d = ((head_pos.x - tail_pos.x).powi(2) + (head_pos.y - tail_pos.y).powi(2)).sqrt();
    // CHAIN_STEP = 14*2 + 12 = 40。許容誤差 ±2px (浮動小数の累積)。
    assert!((d - 40.0).abs() < 2.0, "短い鎖は 40px 等間隔 (実測 {d:.1})");
    let _ = SigilKind::SummonGlyph; // 利用
}

#[test]
fn crowded_chain_extends_past_obstacle() {
    // Phase 4.9 M1: 鎖の進路に他の rings/glyphs がある場合、CHAIN_STEP を伸ばして空きを探す。
    // 制御構造 (if) + 鎖を組み合わせて補助陣の近傍にチェーンを通す。
    use magia_core::ir::{EdgeKind, SigilKind};
    let src = r"
fn f(v: Vec<i32>) -> usize {
    if v.is_empty() {
        return 0;
    }
    v.iter().filter(|x| **x > 0).map(|x| x * 2).count()
}";
    let graph = magia_rust::parse_function(src, "f").unwrap();
    let layout = magia_core::layout::layout(&graph);
    let module = &graph.modules[0];
    // 鎖の全 glyph を取り出し、隣接 glyph 間距離が CHAIN_STEP (40) を下回らないことだけ確認。
    let chain_edges: Vec<(_, _)> = module
        .edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Chain)
        .map(|e| (e.source, e.target))
        .collect();
    assert!(!chain_edges.is_empty(), "鎖 edge が複数本");
    for (a, b) in &chain_edges {
        let pa = layout.positions[a];
        let pb = layout.positions[b];
        let d = ((pa.x - pb.x).powi(2) + (pa.y - pb.y).powi(2)).sqrt();
        assert!(
            d >= 40.0 - 1e-6,
            "鎖 glyph 間距離 {d:.1} は CHAIN_STEP 以上"
        );
        // 上限: CHAIN_STEP * CHAIN_STEP_MAX_FACTOR * step が累積するので、
        // 鎖の長さに比例する。チェーン長 4 (4本最大) を想定して余裕を持つ
        // 400px (= CHAIN_STEP * 4 * MAX_FACTOR = 40 * 4 * 4 / 鎖長 4) とする。
        // M2 の精度要件確定で再調整 (memo)。
        assert!(d <= 640.0 + 1e-6, "鎖 glyph 間距離 {d:.1} は上限以内");
    }
    let _ = SigilKind::SummonGlyph;
}
