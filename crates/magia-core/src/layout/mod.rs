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
use std::f64::consts::{PI, TAU};

use kurbo::{Point, Rect, Vec2};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::ir::{MagiaGraph, Module, Sigil, SigilId, SigilKind};
use crate::layout::constants::{
    AUX_RING_RADIUS, CANVAS_MARGIN, CROSSING_OPT_MAX_PASSES, CROSSING_OPT_ROTATION_STEP_RAD,
    GLYPH_GAP, GLYPH_MARGIN, MAIN_RING_RADIUS, RING_GAP, RING_MARGIN, SUMMON_GLYPH_RADIUS,
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
        relax_ring_overlaps(&mut placements);
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
        for (glyph, point) in fan.glyph_ids.iter().zip(fan.positions()) {
            placed.insert(
                *glyph,
                PlacedSigil {
                    center: point,
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
/// 希望角は「親の outward」+「`anchor_operation / content_len` の全周均等割」+
/// 「同一 anchor 兄弟の中央寄せ扇 (必要角度幅から動的に算出)」。
/// 希望角どうしが近すぎる場合は貪欲パスで最小間隔を強制し (Phase 1.8)、
/// 1軌道に収まらない数の子は距離を1段伸ばした第2軌道以降に送る。
/// 全て閉形式 + SigilId タイブレークで決定論的 (spec §6.1.4)。
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
        let children: Vec<&Sigil> = adjacency
            .children_of_kind(parent_id, SigilKind::AuxRing)
            .into_iter()
            .filter(|child| !placements.contains_key(&child.id)) // 多重 Edge への防御
            .collect();
        if children.is_empty() {
            continue;
        }

        let first_orbit = parent.radius + RING_GAP + AUX_RING_RADIUS;
        let sibling_step = min_angular_step(AUX_RING_RADIUS + RING_MARGIN, first_orbit);

        // anchor ごとの兄弟数 (中央寄せ扇の幅計算に使う)。
        let mut group_sizes: BTreeMap<u32, usize> = BTreeMap::new();
        for child in &children {
            *group_sizes.entry(child_role(child).0).or_insert(0) += 1;
        }

        // 希望角を計算し、(希望角, SigilId) で安定ソート。
        // MainRing 直下は全周に展開するが、入れ子 (親が AuxRing) は祖父母側へ
        // 戻らないよう **outward を中心とした半円** に展開を制限する (Phase 1.8)。
        let parent_is_main = adjacency
            .sigil(parent_id)
            .is_some_and(|s| s.kind == SigilKind::MainRing);
        let mut desired: Vec<(f64, SigilId)> = children
            .iter()
            .map(|child| {
                let (anchor, ordinal) = child_role(child);
                let siblings = usize_to_f64(group_sizes[&anchor]);
                let base = if parent_is_main {
                    // spec §6.1.4: Operation 位置の全周均等割 (3時起点)。
                    parent.outward + TAU * f64::from(anchor) / usize_to_f64(content_len)
                } else {
                    // 入れ子: anchor 位置をビン中央 ((anchor+0.5)/len) で半円に写像。
                    // 単独子 (len=1) はちょうど outward 正面になる。
                    let fraction = (f64::from(anchor) + 0.5) / usize_to_f64(content_len) - 0.5;
                    parent.outward + fraction * PI
                };
                let offset = (f64::from(ordinal) - (siblings - 1.0) / 2.0) * sibling_step;
                (base + offset, child.id)
            })
            .collect();
        desired.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.cmp(&b.1))
        });

        // 軌道ごとに容量分を取り、角度を解決して配置する。
        let mut start = 0usize;
        let mut orbit = 0usize;
        while start < desired.len() {
            let distance = first_orbit + usize_to_f64(orbit) * (2.0 * AUX_RING_RADIUS + RING_GAP);
            let step = min_angular_step(AUX_RING_RADIUS + RING_MARGIN, distance);
            let end = (start + orbit_capacity(step)).min(desired.len());
            let batch = &desired[start..end];
            let batch_desired: Vec<f64> = batch.iter().map(|(angle, _)| *angle).collect();
            let angles = resolve_angles(&batch_desired, step);
            for ((_, id), angle) in batch.iter().zip(angles) {
                let center = parent.center + Vec2::new(angle.cos(), angle.sin()) * distance;
                placements.insert(
                    *id,
                    RingPlacement {
                        center,
                        outward: angle,
                        radius: AUX_RING_RADIUS,
                    },
                );
                queue.push_back(*id);
            }
            start = end;
            orbit += 1;
        }
    }
}

/// 段階2.5 (Phase 1.8): 親をまたぐリングどうし (従兄弟など) の残存重なりを解消する
/// 決定論的緩和パス。
///
/// `place_aux_rings` の角度配分は同一親の子しか調整できず、別の親に属するリングが
/// 過密領域で衝突しうる。重なったペアの**大きい SigilId 側**を、相手から離れる方向へ
/// 不足距離 + わずかな過押し (接触ぎりぎりの浮動小数フリッカ防止) ぶん押し出す。
/// MainRing は SigilId 最小 (= 常に動かない側) なので原点固定 (spec §6.1.4 優先順位1)
/// が保たれる。パス数・ペア処理順とも固定で決定論的。重なりが無いグラフでは
/// 完全に no-op (意匠保全)。
///
/// 既知の限界: 押し出しの連鎖が振動する病的な密集ではパス上限到達後に重なりが
/// 残りうる。サポート規模 (match 6アーム級) は `dense_*` テストで重なり 0 を保証し、
/// 上限超過は重なりテストの失敗として検出する方針。
fn relax_ring_overlaps(placements: &mut BTreeMap<SigilId, RingPlacement>) {
    const PASSES: usize = 32;
    /// 接触ぎりぎり (distance ≈ needed) での再判定フリッカを防ぐ過押し量。
    const OVERSHOOT: f64 = 0.5;
    let ids: Vec<SigilId> = placements.keys().copied().collect();
    for _ in 0..PASSES {
        let mut moved = false;
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let a = placements[&ids[i]];
                let b = placements[&ids[j]];
                let delta = b.center - a.center;
                let distance = delta.hypot();
                let needed = a.radius + b.radius + RING_MARGIN;
                if distance >= needed {
                    continue;
                }
                // 同一点に退化した場合は 3時方向へ離す (決定論的な既定方向)。
                let push = if distance < 1e-6 {
                    Vec2::new(needed + OVERSHOOT, 0.0)
                } else {
                    delta * ((needed - distance + OVERSHOOT) / distance)
                };
                let target = placements
                    .get_mut(&ids[j])
                    .expect("ids は placements のキー由来");
                // outward は意図的に更新しない: glyph や入れ子の基準方向は
                // 「本来の制御フロー位置」を保ち、押し出しは位置の微修正に留める。
                target.center += push;
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
}

fn child_role(child: &Sigil) -> (u32, u32) {
    child
        .layers
        .control_flow
        .as_ref()
        .and_then(|c| c.role.as_ref())
        .map_or((0, 0), |role| (role.anchor_operation, role.ordinal))
}

/// 距離 `distance` の軌道上で、半径方向の広がり `half_extent` の要素どうしが
/// 重ならないための最小角度間隔。要素が軌道半径以上の退化ケースは半周に丸める。
fn min_angular_step(half_extent: f64, distance: f64) -> f64 {
    // half_extent は現状正の定数由来だが、asin の定義域を呼び出し側に頼らない。
    2.0 * (half_extent.max(0.0) / distance.max(f64::EPSILON))
        .clamp(0.0, 1.0)
        .asin()
}

/// 1軌道に置ける要素数の上限。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn orbit_capacity(step: f64) -> usize {
    if step <= f64::EPSILON {
        return usize::MAX;
    }
    ((TAU / step).floor() as usize).max(1)
}

/// 希望角 (昇順) に最小間隔を強制する。前から貪欲に押し出し、円周を一周しても
/// 収まらない (先頭と末尾が衝突する) 場合は先頭希望角を起点とした均等割に
/// フォールバックする。間隔が既に十分な要素は一切動かない (意匠保全)。
///
/// 前提条件: `desired` は昇順 (呼び出し元がソート済み)、かつ要素数は
/// `orbit_capacity(step)` 以下 (= 均等割 TAU/n が step 以上になることが保証される)。
fn resolve_angles(desired: &[f64], step: f64) -> Vec<f64> {
    debug_assert!(
        desired.windows(2).all(|w| w[0] <= w[1]),
        "resolve_angles は昇順入力を前提とする"
    );
    let mut out = desired.to_vec();
    for i in 1..out.len() {
        if out[i] < out[i - 1] + step {
            out[i] = out[i - 1] + step;
        }
    }
    if out.len() >= 2 {
        let span = out[out.len() - 1] - out[0];
        if span > TAU - step {
            let count = usize_to_f64(out.len());
            for (i, slot) in out.iter_mut().enumerate() {
                *slot = desired[0] + TAU * usize_to_f64(i) / count;
            }
        }
    }
    out
}

/// 親リングから放射状に配置される SummonGlyph の一群。
///
/// 基本配置は「基準角 + 回転 + 全周均等割」(Phase 1.5 と同じ)。Phase 1.8 で
/// **子 AuxRing の占有角度帯 (`zones`) に落ちる glyph だけ**を帯の出口へ退避させ、
/// 衝突しない glyph は一切動かさない (オーナー合格済みの意匠を保全する局所修正)。
/// 交差最小化は `rotation` だけを動かす。
struct GlyphFan {
    parent_center: Point,
    distance: f64,
    base_angle: f64,
    rotation: f64,
    /// SigilId 昇順 (= ソース出現順)。
    glyph_ids: Vec<SigilId>,
    /// 占有角度帯 (基準角相対 [0, TAU)、マージ済み昇順)。orbit-1 の子 AuxRing 由来。
    zones: Vec<(f64, f64)>,
    /// glyph どうしの最小角度間隔。
    glyph_step: f64,
}

impl GlyphFan {
    /// 全 glyph の座標 (glyph_ids と同じ並び)。
    ///
    /// 均等割候補 → 占有帯に落ちた glyph を帯の出口へ押し出し → 押し出しの連鎖で
    /// 近づきすぎた glyph に最小間隔を強制、の順で決める。円周を一周しても収まらない
    /// 場合は候補のまま受理する (重なり回避より無限ループ防止を優先する防御)。
    fn positions(&self) -> Vec<Point> {
        let count = self.glyph_ids.len();
        let count_f = usize_to_f64(count.max(1));
        // (相対候補角, 元 index)。rotation を足して [0, TAU) に正規化し、角度順に処理する。
        let mut order: Vec<(f64, usize)> = (0..count)
            .map(|i| {
                (
                    (TAU * usize_to_f64(i) / count_f + self.rotation).rem_euclid(TAU),
                    i,
                )
            })
            .collect();
        order.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.cmp(&b.1))
        });

        let mut resolved = vec![0.0f64; count];
        let mut prev: Option<f64> = None;
        let mut first: Option<f64> = None;
        for (candidate, index) in &order {
            let mut angle = match prev {
                Some(p) => candidate.max(p + self.glyph_step),
                None => *candidate,
            };
            // 占有帯から出るまで前方へ。帯はマージ済みなので高々 zones 数 + 1 回で抜ける。
            // angle が TAU を超えた後の後続 glyph は prev + glyph_step の連鎖で
            // 詰めて並ぶ (意図された挙動)。過剰になれば下の一周超過判定が拾う。
            for _ in 0..=self.zones.len() {
                match self.zone_exit(angle) {
                    Some(exit) => angle = exit,
                    None => break,
                }
            }
            resolved[*index] = angle;
            first.get_or_insert(angle);
            prev = Some(angle);
        }
        // 押し出しの連鎖が一周を超えた (先頭と末尾が衝突する) 場合は、
        // 帯の外の空き弧へ均等配置し直す。空き弧がほぼ無い極端なケースのみ
        // 候補位置のまま受理する (重なり回避より無限ループ防止を優先する防御)。
        if let (Some(first), Some(last)) = (first, prev)
            && count >= 2
            && last - first > TAU - self.glyph_step
        {
            self.spread_uniform_in_free(&order, &mut resolved);
        }
        resolved
            .iter()
            .map(|relative| {
                let angle = self.base_angle + relative;
                self.parent_center + Vec2::new(angle.cos(), angle.sin()) * self.distance
            })
            .collect()
    }

    /// 占有帯の補集合 (空き弧) に glyph を均等配置する (過密時のフォールバック)。
    /// 空き弧が実質ゼロ (全周占有) なら何もしない (= 候補位置のまま受理)。
    /// `rotation` を offset に含めるのは意図的: 交差最小化がフォールバック配置も
    /// 空き弧の中でスライドできるようにする (同じ rotation には常に同じ結果)。
    fn spread_uniform_in_free(&self, order: &[(f64, usize)], resolved: &mut [f64]) {
        let mut arcs: Vec<(f64, f64)> = Vec::new(); // (開始, 長さ) 基準角相対
        let mut cursor = 0.0;
        for (start, end) in &self.zones {
            if *start > cursor {
                arcs.push((cursor, start - cursor));
            }
            cursor = cursor.max(*end);
        }
        if cursor < TAU {
            arcs.push((cursor, TAU - cursor));
        }
        let total: f64 = arcs.iter().map(|(_, length)| length).sum();
        if total < 1e-3 {
            return;
        }
        let count = usize_to_f64(order.len().max(1));
        for (slot, (_, index)) in order.iter().enumerate() {
            let mut offset = (total * usize_to_f64(slot) / count + self.rotation).rem_euclid(total);
            let mut angle = arcs[arcs.len() - 1].0; // 数値誤差時の防御値 (最後の弧の先頭)
            for (start, length) in &arcs {
                if offset < *length {
                    angle = start + offset;
                    break;
                }
                offset -= length;
            }
            resolved[*index] = angle;
        }
    }

    /// `angle` (基準角相対、TAU 超過可) が占有帯の中なら帯の出口角を返す。
    fn zone_exit(&self, angle: f64) -> Option<f64> {
        let wrapped = angle.rem_euclid(TAU);
        for (start, end) in &self.zones {
            if wrapped >= *start && wrapped < *end {
                return Some(angle + (end - wrapped));
            }
        }
        None
    }

    fn segments(&self) -> Vec<Segment> {
        self.positions()
            .into_iter()
            .map(|p| (self.parent_center, p))
            .collect()
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
        let distance = placement.radius + GLYPH_GAP + SUMMON_GLYPH_RADIUS;
        fans.push(GlyphFan {
            parent_center: placement.center,
            distance,
            base_angle: placement.outward,
            rotation: 0.0,
            glyph_ids: glyphs.iter().map(|s| s.id).collect(),
            zones: occupied_zones(*parent_id, placement, distance, placements),
            glyph_step: min_angular_step(SUMMON_GLYPH_RADIUS + GLYPH_MARGIN, distance),
        });
    }
    fans
}

/// glyph 軌道 (半径 `glyph_distance`) と半径方向に重なる**配置済みリング全て**
/// (自分の子・親・叔父リングを含む) の角度帯を、親の outward 相対 [0, TAU) の
/// マージ済み区間として返す。半径方向に離れたリング (第2軌道など) は含めない。
fn occupied_zones(
    parent_id: SigilId,
    parent: &RingPlacement,
    glyph_distance: f64,
    placements: &BTreeMap<SigilId, RingPlacement>,
) -> Vec<(f64, f64)> {
    let mut spans: Vec<(f64, f64)> = Vec::new();
    for (ring_id, ring) in placements {
        if *ring_id == parent_id {
            continue;
        }
        let ring_vec = ring.center - parent.center;
        let ring_distance = ring_vec.hypot();
        if ring_distance < 1e-6 {
            continue;
        }
        let clearance = SUMMON_GLYPH_RADIUS + ring.radius + RING_MARGIN;
        // 半径方向フィルタは円どうしの接触可能性の**厳密な**判定:
        // 軌道上の glyph 中心とリング中心の距離は角度を動かすと [|R-g|, R+g] を取り、
        // その最小値 |R-g| が clearance を超えるならどの角度でも接触しない。
        // 祖父母リング (例: 入れ子の fan から見た MainRing) も意図的に対象に含む。
        if (ring_distance - glyph_distance).abs() > clearance {
            continue; // どの角度でも接触し得ない (第2軌道など)
        }
        // 余弦定理: glyph 軌道上の点がこのリングと clearance を保つ最小角度差。
        let cos_delta = (glyph_distance.powi(2) + ring_distance.powi(2) - clearance.powi(2))
            / (2.0 * glyph_distance * ring_distance);
        let half_width = cos_delta.clamp(-1.0, 1.0).acos();
        let center_angle = (ring_vec.y.atan2(ring_vec.x) - parent.outward).rem_euclid(TAU);
        let start = (center_angle - half_width).rem_euclid(TAU);
        let length = (2.0 * half_width).min(TAU);
        if start + length <= TAU {
            spans.push((start, start + length));
        } else {
            spans.push((start, TAU));
            spans.push((0.0, start + length - TAU));
        }
    }
    merge_spans(spans)
}

fn merge_spans(mut spans: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    spans.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for (start, end) in spans {
        if let Some(last) = merged.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
            continue;
        }
        merged.push((start, end));
    }
    merged
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
