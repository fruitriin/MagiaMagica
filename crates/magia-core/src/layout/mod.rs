//! 決定論的レイアウトエンジン (Phase 1.5, spec §6.1.4)。
//!
//! `MagiaGraph` を入力に、各 `Sigil` の2次元座標を spec §6.1.4 の優先順位で決める:
//!
//! 1. MainRing を画面中央 `(0, 0)` に固定
//! 2. AuxRing を親リング上の制御フロー位置 (`AuxRingRole.anchor_operation`) に基づく
//!    極座標で配置 (3時起点・反時計回り、Mystical 原典に従う)
//! 3. SummonGlyph を親リングから一定距離の放射状に等間隔配置
//! 4. 線の交差を最小化する局所最適化 (オプション、決定論的)
//!
//! 乱数は使わない。同じ IR からは常に同一の結果が出る (Phase 2 差分表示の前提)。
//! 座標系は数学系 (+x 右、+y 上、反時計回りが正)。SVG への変換は M6 レンダラの責務で、
//! 単純な y 反転 (`y_svg = -y_math`) でよい: 数学系の反時計回り (角度増加で上へ) は
//! 反転後も画面上で「3時起点・反時計回り」(右 → 上 → 左 → 下) に見える
//! (spec §6.1.2 と整合。座標の符号としての向きは反転するが、視覚的な回転方向は保たれる)。
//! `cross_module_edges` は Phase 1 (単一関数 = 単一モジュール) では配置に使わない。

pub mod constants;
mod crossing;

use std::collections::{BTreeMap, VecDeque};
use std::f64::consts::TAU;

use kurbo::{Point, Rect, Vec2};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::ir::{MagiaGraph, Module, Sigil, SigilId, SigilKind};
use crate::layout::constants::{
    AUX_RING_RADIUS, CANVAS_MARGIN, CROSSING_OPT_MAX_PASSES, CROSSING_OPT_ROTATION_STEP_RAD,
    GLYPH_GAP, MAIN_RING_RADIUS, RING_GAP, SIBLING_FAN_STEP_RAD, SUMMON_GLYPH_RADIUS,
};
use crate::layout::crossing::{Segment, count_crossings};

/// レイアウトの調整オプション。
#[derive(Debug, Clone, Copy)]
pub struct LayoutOptions {
    /// 交差最小化の局所最適化を適用するか (テスト容易性のため OFF にできる)。
    pub minimize_crossings: bool,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            minimize_crossings: true,
        }
    }
}

/// レイアウト結果。
///
/// `positions` は `BTreeMap` で持つ: M6 レンダラが順次走査して SVG 要素を出力する
/// 際にも決定論的な順序になる (`HashMap` だと出力順が揺れる)。
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutResult {
    /// Sigil 中心の座標。グラフ内の全 Sigil 分のエントリを持つ。
    pub positions: BTreeMap<SigilId, Point>,
    /// 全 Sigil を包含する bounding box + マージン。
    pub canvas: Rect,
}

/// デフォルトオプション (交差最小化 ON) でレイアウトする。
#[must_use = "レイアウト結果はレンダラに渡されるべき"]
pub fn layout(graph: &MagiaGraph) -> LayoutResult {
    layout_with(graph, LayoutOptions::default())
}

/// オプション指定つきでレイアウトする。
#[must_use = "レイアウト結果はレンダラに渡されるべき"]
pub fn layout_with(graph: &MagiaGraph, options: LayoutOptions) -> LayoutResult {
    let mut placed: BTreeMap<SigilId, PlacedSigil> = BTreeMap::new();
    for module in &graph.modules {
        layout_module(module, options, &mut placed);
    }
    let canvas = canvas_rect(&placed);
    let positions = placed.iter().map(|(id, p)| (*id, p.center)).collect();
    LayoutResult { positions, canvas }
}

/// 配置確定後の Sigil 1個分。中心と半径を1レコードで持ち、
/// 「positions にあるが半径が無い」という不整合状態を作らない。
#[derive(Debug, Clone, Copy)]
struct PlacedSigil {
    center: Point,
    radius: f64,
}

/// リング1個の配置情報。`outward` は親から見た放射方向 (子の配置基準になる)。
#[derive(Debug, Clone, Copy)]
struct RingPlacement {
    center: Point,
    outward: f64,
    radius: f64,
}

fn layout_module(
    module: &Module,
    options: LayoutOptions,
    placed: &mut BTreeMap<SigilId, PlacedSigil>,
) {
    let adjacency = Adjacency::build(module);
    let mut placements: BTreeMap<SigilId, RingPlacement> = BTreeMap::new();

    if let Some(main) = module.sigils.iter().find(|s| s.kind == SigilKind::MainRing) {
        place_main_ring(main, &mut placements);
        place_aux_rings(&adjacency, main.id, &mut placements);
    }
    let mut fans = place_summon_glyphs(&adjacency, &placements);
    if options.minimize_crossings {
        let fixed = ring_segments(&adjacency, &placements);
        minimize_glyph_crossings(&mut fans, &fixed);
    }

    for (id, placement) in &placements {
        placed.insert(
            *id,
            PlacedSigil {
                center: placement.center,
                radius: placement.radius,
            },
        );
    }
    for fan in &fans {
        for (index, glyph) in fan.glyph_ids.iter().enumerate() {
            placed.insert(
                *glyph,
                PlacedSigil {
                    center: fan.position_of(index),
                    radius: SUMMON_GLYPH_RADIUS,
                },
            );
        }
    }
    // 不変条件が壊れた IR (MainRing 欠落・到達不能 Sigil) への防御:
    // 未配置の Sigil は原点に置き、結果が常に全 Sigil を覆う規約を守る。
    for sigil in &module.sigils {
        placed.entry(sigil.id).or_insert_with(|| PlacedSigil {
            center: Point::ZERO,
            radius: sigil_radius(sigil.kind),
        });
    }
}

/// 段階1: MainRing を画面中央 `(0, 0)` に固定する (spec §6.1.4 優先順位1)。
fn place_main_ring(main: &Sigil, placements: &mut BTreeMap<SigilId, RingPlacement>) {
    placements.insert(
        main.id,
        RingPlacement {
            center: Point::ZERO,
            // 3時方向 (+x) を基準にする (Mystical 原典の開始点)。
            outward: 0.0,
            radius: MAIN_RING_RADIUS,
        },
    );
}

/// 段階2: AuxRing を親リング上の制御フロー位置に基づく極座標で配置する (優先順位2)。
///
/// 角度は「親の outward」「`anchor_operation / content_len` の全周均等割」
/// 「`ordinal` の扇状ステップ (if 連鎖・match アームの兄弟ずらし)」の和。
/// 入れ子は BFS で展開し、親の outward を基準に外側へ伸びる傾向を作る。
fn place_aux_rings(
    adjacency: &Adjacency,
    root: SigilId,
    placements: &mut BTreeMap<SigilId, RingPlacement>,
) {
    let mut queue: VecDeque<SigilId> = VecDeque::from([root]);
    while let Some(parent_id) = queue.pop_front() {
        let parent = placements[&parent_id];
        let content_len = adjacency
            .sigil(parent_id)
            .map_or(1, |s| s.content.len().max(1));
        for child in adjacency.children_of_kind(parent_id, SigilKind::AuxRing) {
            if placements.contains_key(&child.id) {
                continue; // 多重 Edge への防御 (1子1Edge が正常系)
            }
            let (anchor, ordinal) = child
                .layers
                .control_flow
                .as_ref()
                .and_then(|c| c.role.as_ref())
                .map_or((0, 0), |role| (role.anchor_operation, role.ordinal));
            let angle = parent.outward
                + TAU * f64::from(anchor) / usize_to_f64(content_len)
                + f64::from(ordinal) * SIBLING_FAN_STEP_RAD;
            let distance = parent.radius + RING_GAP + AUX_RING_RADIUS;
            let center = parent.center + Vec2::new(angle.cos(), angle.sin()) * distance;
            placements.insert(
                child.id,
                RingPlacement {
                    center,
                    outward: angle,
                    radius: AUX_RING_RADIUS,
                },
            );
            queue.push_back(child.id);
        }
    }
}

/// 親リングから放射状に配置される SummonGlyph の一群。
///
/// glyph の幾何は「基準角 + 回転 + 全周均等割」で決まり、交差最小化は
/// `rotation` だけを動かす (個々の glyph の入れ替えは放射状配置では幾何を変えない)。
/// 1個だけのファンも交差回避のために outward 方向から回転しうる (意図された挙動。
/// 交差相手がなければ rotation は 0 のまま outward 方向に残る)。
struct GlyphFan {
    parent_center: Point,
    distance: f64,
    base_angle: f64,
    rotation: f64,
    /// SigilId 昇順 (= ソース出現順)。
    glyph_ids: Vec<SigilId>,
}

impl GlyphFan {
    fn position_of(&self, index: usize) -> Point {
        let angle = self.base_angle
            + self.rotation
            + TAU * usize_to_f64(index) / usize_to_f64(self.glyph_ids.len().max(1));
        self.parent_center + Vec2::new(angle.cos(), angle.sin()) * self.distance
    }

    fn segments(&self) -> impl Iterator<Item = Segment> + '_ {
        (0..self.glyph_ids.len()).map(|i| (self.parent_center, self.position_of(i)))
    }
}

/// 段階3: SummonGlyph を最も関連の深いリング (Edge の親) から一定距離に配置する
/// (優先順位3)。同一親の複数 glyph は親中心を通る放射状に全周等間隔。
/// 1個だけなら親の outward 方向 (真っ直ぐ外向き)。
fn place_summon_glyphs(
    adjacency: &Adjacency,
    placements: &BTreeMap<SigilId, RingPlacement>,
) -> Vec<GlyphFan> {
    let mut fans = Vec::new();
    // BTreeMap の走査 = 親 SigilId 昇順なので決定論的。
    for (parent_id, placement) in placements {
        let glyphs = adjacency.children_of_kind(*parent_id, SigilKind::SummonGlyph);
        if glyphs.is_empty() {
            continue;
        }
        fans.push(GlyphFan {
            parent_center: placement.center,
            distance: placement.radius + GLYPH_GAP + SUMMON_GLYPH_RADIUS,
            base_angle: placement.outward,
            rotation: 0.0,
            glyph_ids: glyphs.iter().map(|s| s.id).collect(),
        });
    }
    fans
}

/// 段階4: 交差最小化の局所最適化 (優先順位4、オプション)。
///
/// 放射状配置では同一ファン内の glyph 入れ替えが幾何を変えないため、計画書の
/// 「隣接 SummonGlyph の角度入れ替え」の代わりに**ファン全体の回転**を貪欲に試す。
/// 探索順 (ファンは親 SigilId 順)・ステップ・パス上限とも固定で決定論的。
fn minimize_glyph_crossings(fans: &mut [GlyphFan], fixed_segments: &[Segment]) {
    let mut best = total_crossings(fans, fixed_segments);
    if best == 0 {
        return;
    }
    for _ in 0..CROSSING_OPT_MAX_PASSES {
        let mut improved = false;
        for i in 0..fans.len() {
            for delta in [
                CROSSING_OPT_ROTATION_STEP_RAD,
                -CROSSING_OPT_ROTATION_STEP_RAD,
            ] {
                fans[i].rotation += delta;
                let candidate = total_crossings(fans, fixed_segments);
                if candidate < best {
                    best = candidate;
                    improved = true;
                } else {
                    fans[i].rotation -= delta;
                }
            }
        }
        if !improved || best == 0 {
            break;
        }
    }
}

fn total_crossings(fans: &[GlyphFan], fixed_segments: &[Segment]) -> usize {
    let mut segments: Vec<Segment> = fixed_segments.to_vec();
    for fan in fans {
        segments.extend(fan.segments());
    }
    count_crossings(&segments)
}

/// リング間 (親 → AuxRing) の接続線分。交差カウントの固定要素。
fn ring_segments(
    adjacency: &Adjacency,
    placements: &BTreeMap<SigilId, RingPlacement>,
) -> Vec<Segment> {
    let mut segments = Vec::new();
    for (parent_id, parent) in placements {
        for child in adjacency.children_of_kind(*parent_id, SigilKind::AuxRing) {
            if let Some(child_placement) = placements.get(&child.id) {
                segments.push((parent.center, child_placement.center));
            }
        }
    }
    segments
}

fn canvas_rect(placed: &BTreeMap<SigilId, PlacedSigil>) -> Rect {
    let mut acc: Option<Rect> = None;
    for sigil in placed.values() {
        let item = Rect::new(
            sigil.center.x - sigil.radius,
            sigil.center.y - sigil.radius,
            sigil.center.x + sigil.radius,
            sigil.center.y + sigil.radius,
        );
        acc = Some(acc.map_or(item, |rect| rect.union(item)));
    }
    acc.map_or(Rect::ZERO, |rect| {
        rect.inflate(CANVAS_MARGIN, CANVAS_MARGIN)
    })
}

/// Sigil 種別ごとの描画半径。レンダラ (M6) も同じ値で円を描く。
pub(crate) fn sigil_radius(kind: SigilKind) -> f64 {
    match kind {
        SigilKind::MainRing => MAIN_RING_RADIUS,
        SigilKind::AuxRing => AUX_RING_RADIUS,
        SigilKind::SummonGlyph | SigilKind::GateGlyph => SUMMON_GLYPH_RADIUS,
    }
}

/// usize の小さなカウント (Operation 数・glyph 数) を f64 化する。
/// 2^53 までは正確に表現でき、レイアウト対象のカウントがそれを超えることは
/// 現実にないため、panic 経路を作るより精度劣化 lint の許容を選ぶ。
#[allow(clippy::cast_precision_loss)]
fn usize_to_f64(value: usize) -> f64 {
    value as f64
}

/// petgraph による隣接情報。IR 自体は petgraph に依存させない (tech-selection §2.2)。
struct Adjacency<'a> {
    graph: DiGraph<SigilId, ()>,
    nodes: BTreeMap<SigilId, NodeIndex>,
    sigils: BTreeMap<SigilId, &'a Sigil>,
}

impl<'a> Adjacency<'a> {
    fn build(module: &'a Module) -> Self {
        let mut graph = DiGraph::new();
        let mut nodes = BTreeMap::new();
        let mut sigils = BTreeMap::new();
        for sigil in &module.sigils {
            nodes.insert(sigil.id, graph.add_node(sigil.id));
            sigils.insert(sigil.id, sigil);
        }
        for edge in &module.edges {
            if let (Some(&source), Some(&target)) =
                (nodes.get(&edge.source), nodes.get(&edge.target))
            {
                graph.add_edge(source, target, ());
            }
        }
        Self {
            graph,
            nodes,
            sigils,
        }
    }

    fn sigil(&self, id: SigilId) -> Option<&Sigil> {
        self.sigils.get(&id).copied()
    }

    /// `id` の子のうち指定 kind のものを `SigilId` 昇順で返す。
    /// petgraph の neighbors は挿入逆順のため、昇順ソートで決定論性を保証する。
    fn children_of_kind(&self, id: SigilId, kind: SigilKind) -> Vec<&Sigil> {
        let Some(&node) = self.nodes.get(&id) else {
            return Vec::new();
        };
        let mut children: Vec<&Sigil> = self
            .graph
            .neighbors(node)
            .filter_map(|n| self.sigils.get(&self.graph[n]).copied())
            .filter(|s| s.kind == kind)
            .collect();
        children.sort_by_key(|s| s.id);
        children
    }
}
