//! IR スケルトンの統合テスト。
//!
//! Phase 1.1 受け入れ基準:
//! - 空 `MagiaGraph` の生成
//! - JSON round-trip (空)
//! - Phase 1 のレイヤーのみ埋めた `MagiaGraph` の round-trip

use magia_core::ir::{
    AuxRingKind, AuxRingRole, Cardinality, ConcurrencyInfo, ControlFlowInfo, Edge, EdgeKind,
    EdgeLayerData, EffectSet, LayerData, LoopKind, MagiaGraph, Module, ModuleId, Operation,
    OperationKind, OperationPayload, ProjectMetadata, Sigil, SigilId, SigilKind, SourceSpan,
    TypeInfo,
};

#[test]
fn default_graph_is_empty() {
    let graph = MagiaGraph::default();
    assert!(graph.modules.is_empty());
    assert!(graph.cross_module_edges.is_empty());
    assert_eq!(graph.metadata, ProjectMetadata::default());
}

#[test]
fn empty_graph_round_trips_through_json() {
    let graph = MagiaGraph::default();
    let json = serde_json::to_string(&graph).expect("serialize empty graph");
    let decoded: MagiaGraph = serde_json::from_str(&json).expect("deserialize empty graph");
    assert_eq!(graph, decoded);
}

#[test]
fn phase1_populated_graph_round_trips() {
    let graph = sample_phase1_graph();
    let json = serde_json::to_string(&graph).expect("serialize populated graph");
    let decoded: MagiaGraph = serde_json::from_str(&json).expect("deserialize populated graph");
    assert_eq!(graph, decoded);
}

#[test]
fn same_value_serializes_twice_identically() {
    // 同一 IR を2回シリアライズしたら同じ文字列が出る (struct 定義順の出力に依存)。
    // 将来 `HashMap` フィールドを LayerData 等に追加した場合は、キー順を固定する
    // 仕組み (`BTreeMap` への置換等) を別途検討すること。
    let graph = sample_phase1_graph();
    let json_a = serde_json::to_string(&graph).unwrap();
    let json_b = serde_json::to_string(&graph).unwrap();
    assert_eq!(json_a, json_b);
}

#[test]
fn deserialize_minimal_json_uses_defaults() {
    // 空 JSON object から MagiaGraph を復元できる (= 後方互換性の布石)。
    let decoded: MagiaGraph = serde_json::from_str("{}").expect("deserialize minimal");
    assert_eq!(decoded, MagiaGraph::default());
}

fn sample_phase1_graph() -> MagiaGraph {
    let main_ring = Sigil {
        id: SigilId(0),
        kind: SigilKind::MainRing,
        content: vec![Operation {
            kind: OperationKind::Call,
            effects: EffectSet {
                io: true,
                ..EffectSet::default()
            },
            payload: OperationPayload {
                source_excerpt: Some("println!(\"hello\")".to_string()),
                call_target: Some("std::println".to_string()),
                early_return: false,
            },
        }],
        layers: LayerData {
            control_flow: Some(ControlFlowInfo {
                branch_count: 0,
                loop_count: 0,
                early_return_count: 0,
                role: None,
            }),
            type_info: Some(TypeInfo {
                signature: Some("fn hello()".to_string()),
                returns_result: false,
                returns_option: false,
            }),
            concurrency: Some(ConcurrencyInfo {
                is_async: false,
                await_points: 0,
            }),
            ..LayerData::default()
        },
        source_location: SourceSpan {
            file: "src/lib.rs".to_string(),
            start_line: 1,
            end_line: 3,
            ..SourceSpan::default()
        },
        cardinality: Cardinality {
            weight: 1.0,
            density: None,
        },
    };

    let aux_ring = Sigil {
        id: SigilId(1),
        kind: SigilKind::AuxRing,
        layers: LayerData {
            // Phase 1.3: AuxRing は親リングとの接続情報 (role) を持つ。
            control_flow: Some(ControlFlowInfo {
                role: Some(AuxRingRole {
                    kind: AuxRingKind::LoopBody(LoopKind::For),
                    anchor_operation: 0,
                    ordinal: 0,
                    label: None,
                }),
                ..ControlFlowInfo::default()
            }),
            ..LayerData::default()
        },
        ..Sigil::default()
    };

    let module = Module {
        id: ModuleId(0),
        name: "demo".to_string(),
        sigils: vec![main_ring, aux_ring],
        edges: vec![Edge {
            source: SigilId(0),
            target: SigilId(1),
            kind: EdgeKind::ControlFlow,
            cardinality: 1.0,
            layers: EdgeLayerData::default(),
        }],
    };

    MagiaGraph {
        modules: vec![module],
        cross_module_edges: Vec::new(),
        metadata: ProjectMetadata {
            name: "demo".to_string(),
            version: Some("0.1.0".to_string()),
            root_path: None,
        },
    }
}
