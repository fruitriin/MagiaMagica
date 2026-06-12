//! IR 差分エンジン — Spell Diff の中核 (Phase 3.1, spec v0.3 §9.2)。
//!
//! 同一関数の2つの `MagiaGraph` (before / after) を比較する。
//! **`SigilId` はリビジョン間で安定しない**ため、対応付けは ID でなく構造マッチング:
//! リングは (種別, anchor_operation, ordinal)、召喚記号は (call_target, 所属リング) を
//! キーに、同キー内はソース出現順の貪欲対応で決定論的に解消し、余りを追加/削除とする。
//! 同一入力からは空 diff。diff 自体も決定論的。

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::filter::EffectCategory;
use crate::ir::{AuxRingKind, EdgeKind, LoopKind, MagiaGraph, Module, Sigil, SigilId, SigilKind};
use crate::metrics::{Metrics, measure};

/// 差分の結果 (spec v0.3 §9.2 の契約)。
///
/// `SigilId` はリビジョン間で不安定なため外部契約 (JSON / レポート) には出さないが、
/// **同一リビジョンのレイアウトと突き合わせる**用途 (Phase 3.2 の overlay 描画) のために
/// 各ノードは出自側の ID を保持する: added は after 側、removed は before 側。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpellDiff {
    /// 追加されたノード (after 側の ID と人間可読経路、決定論的順序)。
    pub added: Vec<NodeRef>,
    /// 削除されたノード (before 側の ID と経路)。
    pub removed: Vec<NodeRef>,
    /// 変更されたノードの経路と変更内容。
    pub changed: Vec<NodeChange>,
    /// メトリクス変化。
    pub metrics: MetricsDelta,
}

/// 差分に現れたノード1件への参照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeRef {
    /// 人間可読の経路 (例: "main > if分岐(anchor 1, #0) > 召喚 audit")。
    pub path: String,
    /// 出自リビジョン内での ID (added: after / removed: before)。
    pub sigil: SigilId,
}

/// 変更されたノード1件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeChange {
    pub path: String,
    /// 変更内容の日本語記述 (例: "操作数 3 → 5")。
    pub details: Vec<String>,
    /// before リビジョン内での ID。
    pub before: SigilId,
    /// after リビジョン内での ID (overlay はこちらの位置に強調を描く)。
    pub after: SigilId,
}

/// メトリクスの before / after (差分は表示時に計算する)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MetricsDelta {
    pub before: Metrics,
    pub after: Metrics,
}

impl SpellDiff {
    /// 構造の変化 (追加/削除/変更ノード) が1つも無いか。
    /// メトリクス変化は判定に含めない — ノード差分が空でメトリクスだけ動くことは
    /// 原理上ないが、判定の根拠を構造側に限定しておく。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// 日本語のテキストレポート (変化のない項目は省略する)。
    #[must_use]
    pub fn to_report(&self, function_name: &str) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "SpellDiff (関数 {function_name}):");
        if self.is_empty() {
            let _ = writeln!(out, "  構造の変化なし。");
        }
        for node in &self.added {
            let _ = writeln!(out, "  追加: {}", node.path);
        }
        for node in &self.removed {
            let _ = writeln!(out, "  削除: {}", node.path);
        }
        for change in &self.changed {
            let _ = writeln!(
                out,
                "  変更: {} — {}",
                change.path,
                change.details.join("; ")
            );
        }
        let metrics_lines = metric_changes(&self.metrics);
        if !metrics_lines.is_empty() {
            let _ = writeln!(out, "  メトリクス変化: {}", metrics_lines.join("、"));
        }
        out
    }
}

fn metric_changes(delta: &MetricsDelta) -> Vec<String> {
    let mut lines = Vec::new();
    let pairs: [(&str, u32, u32); 6] = [
        ("複雑度", delta.before.complexity, delta.after.complexity),
        (
            "副作用カテゴリ",
            delta.before.effect_categories,
            delta.after.effect_categories,
        ),
        ("リング数", delta.before.rings, delta.after.rings),
        ("召喚記号数", delta.before.glyphs, delta.after.glyphs),
        (
            "早期リターン",
            delta.before.early_returns,
            delta.after.early_returns,
        ),
        // CI しきい値 (spec v0.3 §9.3) の根拠数値。レポートでも見えるようにする。
        (
            "unsafe 操作",
            delta.before.unsafe_ops,
            delta.after.unsafe_ops,
        ),
    ];
    for (label, before, after) in pairs {
        if before != after {
            lines.push(format!("{label} {before} → {after}"));
        }
    }
    lines
}

/// 2つのグラフを比較する。Phase 1〜3 は単一関数 = 先頭モジュールの比較。
#[must_use = "diff 結果はレポートや CI 判定に使われるべき"]
pub fn diff(before: &MagiaGraph, after: &MagiaGraph) -> SpellDiff {
    let mut result = SpellDiff::default();
    let (Some(before_module), Some(after_module)) = (before.modules.first(), after.modules.first())
    else {
        return result;
    };
    result.metrics = MetricsDelta {
        before: measure(before_module),
        after: measure(after_module),
    };

    let before_tree = Tree::build(before_module);
    let after_tree = Tree::build(after_module);
    match (before_tree, after_tree) {
        (Some(b), Some(a)) => match_nodes(&b, &a, "main", &mut result),
        (Some(b), None) => collect_subtree(&b, "main", &mut result.removed),
        (None, Some(a)) => collect_subtree(&a, "main", &mut result.added),
        (None, None) => {}
    }
    result
}

// ===== 木の構築 =====

struct Tree<'a> {
    sigil: &'a Sigil,
    children: Vec<Tree<'a>>,
}

impl<'a> Tree<'a> {
    /// edges から親子を復元して MainRing を根とする木を作る。
    fn build(module: &'a Module) -> Option<Tree<'a>> {
        let sigils: BTreeMap<SigilId, &Sigil> = module.sigils.iter().map(|s| (s.id, s)).collect();
        let mut children_of: BTreeMap<SigilId, Vec<SigilId>> = BTreeMap::new();
        for edge in &module.edges {
            // 木構造を成すのは係留 Edge (ControlFlow + Chain — Phase 4.8 で鎖後続の
            // glyph は先行 glyph の子になる)。DataFlow Edge (Phase 3.4) はスコープを
            // 跨ぐ別系統の線で、親子関係には使わない。
            if edge.kind != EdgeKind::ControlFlow && edge.kind != EdgeKind::Chain {
                continue;
            }
            children_of
                .entry(edge.source)
                .or_default()
                .push(edge.target);
        }
        let main = module
            .sigils
            .iter()
            .find(|s| s.kind == SigilKind::MainRing)?;
        Some(Self::build_node(main, &sigils, &children_of))
    }

    fn build_node(
        sigil: &'a Sigil,
        sigils: &BTreeMap<SigilId, &'a Sigil>,
        children_of: &BTreeMap<SigilId, Vec<SigilId>>,
    ) -> Tree<'a> {
        let mut children: Vec<Tree<'a>> = children_of
            .get(&sigil.id)
            .into_iter()
            .flatten()
            .filter_map(|id| sigils.get(id))
            .map(|child| Self::build_node(child, sigils, children_of))
            .collect();
        // マッチングと出力順の決定論性: キー順 → SigilId 順 (= ソース出現順)。
        children.sort_by(|a, b| {
            node_key(a.sigil)
                .cmp(&node_key(b.sigil))
                .then(a.sigil.id.cmp(&b.sigil.id))
        });
        Tree { sigil, children }
    }
}

/// 構造マッチングのキー (spec v0.3 §9.2)。`Ord` でマッチングが決定論的になる。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NodeKey {
    /// (種別の判別値, anchor_operation, ordinal)
    Ring(u8, u32, u32),
    /// 召喚記号: 呼び出し先 (所属リングはパスで表現される)
    Glyph(String),
}

fn node_key(sigil: &Sigil) -> NodeKey {
    match sigil.kind {
        SigilKind::MainRing | SigilKind::AuxRing => {
            let role = sigil
                .layers
                .control_flow
                .as_ref()
                .and_then(|c| c.role.as_ref());
            let kind_disc = role.map_or(0, |r| ring_kind_discriminant(r.kind));
            let (anchor, ordinal) = role.map_or((0, 0), |r| (r.anchor_operation, r.ordinal));
            NodeKey::Ring(kind_disc, anchor, ordinal)
        }
        SigilKind::SummonGlyph | SigilKind::GateGlyph => NodeKey::Glyph(
            sigil
                .content
                .first()
                .and_then(|op| op.payload.call_target.clone())
                .unwrap_or_default(),
        ),
    }
}

fn ring_kind_discriminant(kind: AuxRingKind) -> u8 {
    match kind {
        AuxRingKind::IfBranch => 1,
        AuxRingKind::ElseBranch => 2,
        AuxRingKind::MatchArm => 3,
        AuxRingKind::LoopBody(LoopKind::For) => 4,
        AuxRingKind::LoopBody(LoopKind::While) => 5,
        AuxRingKind::LoopBody(LoopKind::Loop) => 6,
    }
}

/// ノードの人間可読ラベル (経路の1セグメント)。
fn node_label(sigil: &Sigil) -> String {
    match sigil.kind {
        SigilKind::MainRing => "main".to_string(),
        SigilKind::AuxRing => {
            let role = sigil
                .layers
                .control_flow
                .as_ref()
                .and_then(|c| c.role.as_ref());
            match role {
                Some(role) => {
                    let base = match role.kind {
                        AuxRingKind::IfBranch => "if分岐".to_string(),
                        AuxRingKind::ElseBranch => "else分岐".to_string(),
                        AuxRingKind::MatchArm => match &role.label {
                            Some(pattern) => format!("matchアーム「{pattern}」"),
                            None => "matchアーム".to_string(),
                        },
                        AuxRingKind::LoopBody(LoopKind::For) => "forループ本体".to_string(),
                        AuxRingKind::LoopBody(LoopKind::While) => "whileループ本体".to_string(),
                        AuxRingKind::LoopBody(LoopKind::Loop) => "loopループ本体".to_string(),
                    };
                    format!(
                        "{base}(anchor {}, #{})",
                        role.anchor_operation, role.ordinal
                    )
                }
                None => "補助リング".to_string(),
            }
        }
        SigilKind::SummonGlyph | SigilKind::GateGlyph => {
            let target = sigil
                .content
                .first()
                .and_then(|op| op.payload.call_target.as_deref())
                .unwrap_or("(不明)");
            format!("召喚 {target}")
        }
    }
}

// ===== マッチング =====

fn match_nodes(before: &Tree, after: &Tree, path: &str, result: &mut SpellDiff) {
    // 1) 自身の内容比較
    let details = compare_sigils(before.sigil, after.sigil);
    if !details.is_empty() {
        result.changed.push(NodeChange {
            path: path.to_string(),
            details,
            before: before.sigil.id,
            after: after.sigil.id,
        });
    }

    // 2) 子をキーでグループ化し、同キー内は出現順 (構築時ソート済み) で貪欲に対応付ける。
    let before_groups = group_children(before);
    let after_groups = group_children(after);

    let mut keys: Vec<&NodeKey> = before_groups.keys().chain(after_groups.keys()).collect();
    keys.sort();
    keys.dedup();
    for key in keys {
        let empty = Vec::new();
        let befores = before_groups.get(key).unwrap_or(&empty);
        let afters = after_groups.get(key).unwrap_or(&empty);
        let paired = befores.len().min(afters.len());
        for i in 0..paired {
            let child_path = format!("{path} > {}", node_label(afters[i].sigil));
            match_nodes(befores[i], afters[i], &child_path, result);
        }
        for extra in &befores[paired..] {
            collect_subtree(
                extra,
                &format!("{path} > {}", node_label(extra.sigil)),
                &mut result.removed,
            );
        }
        for extra in &afters[paired..] {
            collect_subtree(
                extra,
                &format!("{path} > {}", node_label(extra.sigil)),
                &mut result.added,
            );
        }
    }
}

/// 子をマッチングキーでグループ化する。同キー内は構築時ソート (SigilId 順) を保つ。
fn group_children<'t, 'a>(tree: &'t Tree<'a>) -> BTreeMap<NodeKey, Vec<&'t Tree<'a>>> {
    let mut map: BTreeMap<NodeKey, Vec<&Tree>> = BTreeMap::new();
    for child in &tree.children {
        map.entry(node_key(child.sigil)).or_default().push(child);
    }
    map
}

/// 部分木全体を追加/削除リストへ平坦化する。
fn collect_subtree(tree: &Tree, path: &str, sink: &mut Vec<NodeRef>) {
    sink.push(NodeRef {
        path: path.to_string(),
        sigil: tree.sigil.id,
    });
    for child in &tree.children {
        collect_subtree(
            child,
            &format!("{path} > {}", node_label(child.sigil)),
            sink,
        );
    }
}

/// 同一キーで対応付いたノードの内容差分 (日本語の変更記述)。
fn compare_sigils(before: &Sigil, after: &Sigil) -> Vec<String> {
    let mut details = Vec::new();

    if before.content.len() == after.content.len() {
        let signature = |sigil: &Sigil| -> Vec<(crate::ir::OperationKind, EffectCategory, bool)> {
            sigil
                .content
                .iter()
                .map(|op| {
                    (
                        op.kind,
                        EffectCategory::of(&op.effects),
                        op.payload.early_return,
                    )
                })
                .collect()
        };
        if signature(before) != signature(after) {
            details.push("操作列の内容が変化".to_string());
        }
    } else {
        details.push(format!(
            "操作数 {} → {}",
            before.content.len(),
            after.content.len()
        ));
    }

    let counts = |sigil: &Sigil| {
        sigil.layers.control_flow.as_ref().map_or((0, 0, 0), |c| {
            (c.branch_count, c.loop_count, c.early_return_count)
        })
    };
    let (b_branch, b_loop, b_early) = counts(before);
    let (a_branch, a_loop, a_early) = counts(after);
    if b_branch != a_branch {
        details.push(format!("分岐数 {b_branch} → {a_branch}"));
    }
    if b_loop != a_loop {
        details.push(format!("ループ数 {b_loop} → {a_loop}"));
    }
    if b_early != a_early {
        details.push(format!("早期リターン {b_early} → {a_early}"));
    }

    // MainRing は木の根としてのみ対応付くため両側同種だが、前提を明示しておく。
    if before.kind == SigilKind::MainRing && after.kind == SigilKind::MainRing {
        let type_pair = |sigil: &Sigil| {
            sigil
                .layers
                .type_info
                .as_ref()
                .map(|t| (t.signature.clone(), t.returns_result, t.returns_option))
        };
        if type_pair(before) != type_pair(after) {
            details.push("シグネチャ・戻り値型が変化".to_string());
        }
        let concurrency = |sigil: &Sigil| {
            sigil
                .layers
                .concurrency
                .as_ref()
                .map(|c| (c.is_async, c.await_points))
        };
        if concurrency(before) != concurrency(after) {
            details.push("async/await が変化".to_string());
        }
    }

    if matches!(before.kind, SigilKind::SummonGlyph | SigilKind::GateGlyph) {
        let category = |sigil: &Sigil| {
            sigil
                .content
                .first()
                .map(|op| EffectCategory::of(&op.effects))
        };
        let (b, a) = (category(before), category(after));
        if b != a {
            details.push(format!(
                "効果カテゴリ {} → {}",
                b.map_or("なし", EffectCategory::as_str),
                a.map_or("なし", EffectCategory::as_str),
            ));
        }
    }

    details
}
