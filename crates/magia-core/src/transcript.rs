//! 呪文書き起こし — Incantation Transcript (Phase 2.4, spec v0.2 §15)。
//!
//! 図形ベース表現のアクセシビリティ対応として、SVG と**同じ IR** から構造化テキスト
//! を生成する (同一 IR の射影であることが内容の一致を保証する)。
//! 出力は装飾なしのプレーンテキスト・決定論的 (スクリーンリーダー前提)。
//!
//! 文面は要素ごとの関数に分離してあり、Phase 3 のレイヤー切替対応では
//! 文の差し込みで拡張する。

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::filter::EffectCategory;
use crate::ir::{AuxRingKind, MagiaGraph, Module, Sigil, SigilKind};

/// グラフ全体の書き起こしを生成する。Phase 1〜2 は単一関数 = 単一モジュール。
#[must_use = "書き起こしは出力先に渡されるべき"]
pub fn transcribe(graph: &MagiaGraph) -> String {
    let mut out = String::new();
    for module in &graph.modules {
        transcribe_module(&mut out, module);
    }
    if out.is_empty() {
        out.push_str("描画対象がありません。\n");
    }
    out
}

fn transcribe_module(out: &mut String, module: &Module) {
    let Some(main) = module.sigils.iter().find(|s| s.kind == SigilKind::MainRing) else {
        return;
    };
    let _ = writeln!(out, "関数 {}:", module.name);
    let _ = writeln!(out, "  {}", main_ring_sentence(main));
    if let Some(sentence) = aux_rings_sentence(module) {
        let _ = writeln!(out, "  {sentence}");
    }
    if let Some(sentence) = calls_sentence(module) {
        let _ = writeln!(out, "  {sentence}");
    }
    let _ = writeln!(out, "  {}", returns_sentence(module, main));
    if let Some(sentence) = async_sentence(main) {
        let _ = writeln!(out, "  {sentence}");
    }
    let _ = writeln!(out, "  {}", metrics_sentence(module));
}

/// メインリングの規模 (notes §9.1 の「中規模 (12操作)」相当)。
fn main_ring_sentence(main: &Sigil) -> String {
    let operations = main.content.len();
    // 閾値は notes §9.1 の例 (12操作 = 中規模) からの逆算で 5〜14 を中規模とする。
    let scale = match operations {
        0..=4 => "小規模",
        5..=14 => "中規模",
        _ => "大規模",
    };
    format!("メインリングは{scale} ({operations}操作)。")
}

/// 補助リングの数と種別 (入れ子を含む全数)。
fn aux_rings_sentence(module: &Module) -> Option<String> {
    let mut branches = 0usize;
    let mut loops = 0usize;
    for sigil in &module.sigils {
        if sigil.kind != SigilKind::AuxRing {
            continue;
        }
        let kind = sigil
            .layers
            .control_flow
            .as_ref()
            .and_then(|c| c.role.as_ref())
            .map(|role| role.kind);
        match kind {
            Some(AuxRingKind::LoopBody(_)) => loops += 1,
            // 分岐系 (if/else/match アーム)。role 欠落も分岐扱いに倒す (防御)。
            _ => branches += 1,
        }
    }
    let total = branches + loops;
    if total == 0 {
        return None;
    }
    Some(format!(
        "補助リングを{total}個持つ (分岐{branches}、ループ{loops}。入れ子を含む)。"
    ))
}

/// 外部呼び出しの集計 (同一呼び出し先をまとめ、カテゴリと回数を付す)。
fn calls_sentence(module: &Module) -> Option<String> {
    // BTreeMap で呼び出し先名の辞書順 = 決定論的な列挙順。
    let mut calls: BTreeMap<&str, (EffectCategory, usize)> = BTreeMap::new();
    for sigil in &module.sigils {
        if !matches!(sigil.kind, SigilKind::SummonGlyph | SigilKind::GateGlyph) {
            continue;
        }
        let Some(op) = sigil.content.first() else {
            continue;
        };
        let Some(target) = op.payload.call_target.as_deref() else {
            continue;
        };
        let category = EffectCategory::of(&op.effects);
        let entry = calls.entry(target).or_insert((EffectCategory::Pure, 0));
        // 同一呼び出し先でカテゴリが揺れた場合は**危険側を採用**する
        // (聞き手に安全側の誤解を与えないため)。
        if category.danger_rank() > entry.0.danger_rank() {
            entry.0 = category;
        }
        entry.1 += 1;
    }
    if calls.is_empty() {
        return None;
    }
    let parts: Vec<String> = calls
        .iter()
        .map(|(target, (category, count))| {
            format!("{target} ({}、{count}回)", category_label(*category))
        })
        .collect();
    Some(format!("外部呼び出し: {}。", parts.join("、")))
}

fn category_label(category: EffectCategory) -> &'static str {
    match category {
        EffectCategory::Pure => "純粋",
        EffectCategory::Io => "IO副作用",
        EffectCategory::Network => "ネットワーク副作用",
        EffectCategory::Db => "DB副作用",
        EffectCategory::Filesystem => "ファイルシステム副作用",
        EffectCategory::Unsafe => "unsafe",
    }
}

/// 早期リターン経路と戻り値型。
fn returns_sentence(module: &Module, main: &Sigil) -> String {
    let early_returns: u32 = module
        .sigils
        .iter()
        .filter_map(|s| s.layers.control_flow.as_ref())
        .map(|c| c.early_return_count)
        .sum();
    let mut sentence = if early_returns == 0 {
        "早期リターンなし。".to_string()
    } else {
        format!("早期リターンが{early_returns}経路。")
    };
    if let Some(type_info) = &main.layers.type_info {
        // 両フラグ true は parse 上発生しないが、万一の並存時は Result を優先する
        // (Result<Option<T>> 等で外側の型が支配的なため)。
        if type_info.returns_result {
            sentence.push_str("Result型を返す。");
        } else if type_info.returns_option {
            sentence.push_str("Option型を返す。");
        }
    }
    sentence
}

/// async / await (該当するときのみ)。
fn async_sentence(main: &Sigil) -> Option<String> {
    let concurrency = main.layers.concurrency.as_ref()?;
    if !concurrency.is_async {
        return None;
    }
    Some(format!(
        "async 関数 (await {}点)。",
        concurrency.await_points
    ))
}

/// Phase 1 メトリクス (notes §9.1 の「複雑度4、副作用カテゴリ2種」相当)。
///
/// 複雑度は循環的複雑度の近似: 1 + 全リングの分岐数 + ループ数。
/// 副作用カテゴリ数は記号 (Operation・召喚記号) に現れた純粋以外のカテゴリの種類数。
fn metrics_sentence(module: &Module) -> String {
    let mut complexity: u32 = 1;
    let mut categories: Vec<EffectCategory> = Vec::new();
    for sigil in &module.sigils {
        if let Some(info) = sigil.layers.control_flow.as_ref() {
            complexity += info.branch_count + info.loop_count;
        }
        for op in &sigil.content {
            let category = EffectCategory::of(&op.effects);
            if category != EffectCategory::Pure && !categories.contains(&category) {
                categories.push(category);
            }
        }
    }
    format!(
        "Phase 1 メトリクス: 複雑度{complexity}、副作用カテゴリ{}種。",
        categories.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{
        ConcurrencyInfo, EffectSet, LayerData, MagiaGraph, ModuleId, Operation, OperationPayload,
        TypeInfo,
    };

    fn glyph(target: &str, effects: EffectSet) -> Sigil {
        Sigil {
            kind: SigilKind::SummonGlyph,
            content: vec![Operation {
                effects,
                payload: OperationPayload {
                    call_target: Some(target.to_string()),
                    ..OperationPayload::default()
                },
                ..Operation::default()
            }],
            ..Sigil::default()
        }
    }

    #[test]
    fn same_target_with_conflicting_categories_takes_dangerous_side() {
        let io = EffectSet {
            io: true,
            ..EffectSet::default()
        };
        let network = EffectSet {
            network: true,
            ..EffectSet::default()
        };
        let graph = MagiaGraph {
            modules: vec![Module {
                id: ModuleId(0),
                name: "demo".to_string(),
                sigils: vec![
                    Sigil {
                        kind: SigilKind::MainRing,
                        ..Sigil::default()
                    },
                    glyph("conn::send", io),
                    glyph("conn::send", network),
                ],
                edges: Vec::new(),
            }],
            ..MagiaGraph::default()
        };
        let text = transcribe(&graph);
        assert!(
            text.contains("conn::send (ネットワーク副作用、2回)"),
            "危険側 (network) のカテゴリで集計される: {text}"
        );
    }

    #[test]
    fn empty_graph_has_placeholder() {
        assert_eq!(
            transcribe(&MagiaGraph::default()),
            "描画対象がありません。\n"
        );
    }

    #[test]
    fn small_async_result_function_is_fully_described() {
        let main = Sigil {
            kind: SigilKind::MainRing,
            content: vec![crate::ir::Operation::default(); 3],
            layers: LayerData {
                type_info: Some(TypeInfo {
                    signature: None,
                    returns_result: true,
                    returns_option: false,
                }),
                concurrency: Some(ConcurrencyInfo {
                    is_async: true,
                    await_points: 2,
                }),
                ..LayerData::default()
            },
            ..Sigil::default()
        };
        let graph = MagiaGraph {
            modules: vec![Module {
                id: ModuleId(0),
                name: "demo".to_string(),
                sigils: vec![main],
                edges: Vec::new(),
            }],
            ..MagiaGraph::default()
        };
        let text = transcribe(&graph);
        assert!(text.starts_with("関数 demo:\n"));
        assert!(text.contains("メインリングは小規模 (3操作)。"));
        assert!(text.contains("Result型を返す。"));
        assert!(text.contains("async 関数 (await 2点)。"));
        assert!(text.contains("複雑度1"));
    }
}
