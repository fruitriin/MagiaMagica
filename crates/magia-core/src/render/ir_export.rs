//! 配置済み IR の JSON エクスポート (Phase 4.0.9, spec v0.3 §16)。
//!
//! Vue クライアント (web/) が `MagicCircleSchema` を組むための意味論 + 配置データ。
//! レイアウト (座標・半径・操作ドット配置・シグネチャ円弧) は全て Rust 側で確定し、
//! Vue は描画専任 (POSD 分担)。記号 (分岐 Y 字・ループ三角・早期リターン矢) の
//! 頂点計算は「描画」なので Vue 側 — ここでは種別と向きだけを出す。
//!
//! `id` は JSON 内の相互参照 (edge の from/to) 専用。SigilId はパースごとに
//! 変わりうるため、URL 等への永続化に使ってはならない (Phase 3.2 の情報隠蔽方針)。

use std::f64::consts::{PI, TAU};

use kurbo::{Arc, Shape, Vec2};
use serde::Serialize;

use crate::filter::EffectCategory;
use crate::ir::{AuxRingKind, EdgeKind, MagiaGraph, Module, Sigil, SigilKind};
use crate::layout::constants::{OPERATION_DOT_INSET, OPERATION_DOT_RADIUS};
use crate::layout::{LayoutResult, sigil_radius};
use crate::render::midchilda::{
    aux_kind, early_return_count, normalize_path_numbers, outward_direction, screen_position,
    signature_arc_radius, usize_to_f64,
};
use crate::render::palette;

/// 関数1つ分の配置済み IR (ミッドチルダ式の意味論)。
#[derive(Serialize)]
pub struct SpellIr {
    /// SVG viewBox: [minX, minY, width, height]。
    pub view_box: [f64; 4],
    pub rings: Vec<RingIr>,
    pub glyphs: Vec<GlyphIr>,
    pub edges: Vec<EdgeIr>,
    pub signature: Option<SignatureIr>,
    /// Result / Option 戻り値の分岐線の起点 (メインリング 9 時)。なければ省略。
    pub return_branch: Option<[f64; 2]>,
}

#[derive(Serialize)]
pub struct RingIr {
    pub id: u32,
    pub role: RingRole,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    /// async fn は内側に二重線を描く (spec §6.1.3)。
    pub is_async: bool,
    /// リング中央の制御記号 (分岐 Y 字 / ループ三角)。メインリングは常に None。
    pub symbol: Option<RingSymbol>,
    /// 早期リターン矢印の向き (単位ベクトル)。経路がなければ None。
    pub early_return: Option<[f64; 2]>,
    /// リング上の操作ドット (3時起点・反時計回りの配置済み座標、spec §6.1.2)。
    pub operations: Vec<OperationIr>,
    /// 補助リングのガード・ヘッダの原文位置 (`if cond` / `pat if guard` /
    /// `for pat in expr`)。メインリング・無条件の腕 (`else`) は None。
    /// serve 層がホバープレビュー用の切り出しに使う (Phase 4.1 追加要望4)。
    pub guard_span: Option<SpanIr>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RingRole {
    Main,
    Aux,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RingSymbol {
    Branch,
    Loop,
}

#[derive(Serialize)]
pub struct OperationIr {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub effect: EffectCategory,
    /// 操作の原文位置 (plain statement = 文全体、制御 = キーワード〜ガード式)。
    /// serve 層がホバープレビュー用の切り出し + ハイライトに使う (Phase 4.1)。
    pub source_span: Option<SpanIr>,
}

/// 召喚印 (外部呼び出し)。GateGlyph も描画上は同形なのでまとめる。
#[derive(Serialize)]
pub struct GlyphIr {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub effect: EffectCategory,
    /// 呼び出し先の名前 (`write_defs` / `.expect` / `writeln!` 等)。
    /// 同ファイル関数への解決 (ピン可能判定) はクライアントが関数一覧と照合する
    /// (Phase 4.1 の召喚印インスペクタ / Phase 4.4 呼び出しジャンプの入力)。
    pub call_target: Option<String>,
    /// 呼び出し式全体 (レシーバ・引数込み) の原文位置。serve 層が原文の
    /// 切り出し + ハイライトに使う (Phase 4.1 インスペクタの呼び出し式表示)。
    pub source_span: Option<SpanIr>,
}

/// 原文上の位置範囲 (行・列とも 1-based・文字単位、`end_column` は最後の文字の
/// 直後 = exclusive — `SourceSpan` の規約をそのまま写す)。列が取れない解析器では
/// span ごと省略するため、ここでは全フィールド必須。
#[derive(Serialize)]
pub struct SpanIr {
    pub start_line: u32,
    pub end_line: u32,
    pub start_column: u32,
    pub end_column: u32,
}

/// `SourceSpan` から完全な位置範囲だけを取り出す (行未確定・列欠落は None)。
fn span_ir(loc: &crate::ir::SourceSpan) -> Option<SpanIr> {
    if loc.start_line == 0 {
        return None;
    }
    Some(SpanIr {
        start_line: loc.start_line,
        end_line: loc.end_line,
        start_column: loc.start_column?,
        end_column: loc.end_column?,
    })
}

/// 制御フローの接続。端点 (リング表面) の計算は from/to の中心 + 半径から
/// 自明に決まるため、描画側 (Vue) が行う。
#[derive(Serialize)]
pub struct EdgeIr {
    pub from: u32,
    pub to: u32,
}

#[derive(Serialize)]
pub struct SignatureIr {
    pub text: String,
    /// 円弧 textPath の path データ (9時 → 頂上 → 3時の上半円、衝突回避済み)。
    pub arc_path: String,
}

/// 負のゼロを正の 0.0 に正規化する (IEEE 754: -0.0 + 0.0 = 0.0)。
/// `screen_position` の y 反転 (`-p.y`) が原点で -0.0 を生み、JSON に "-0.0" が
/// 混入するのを防ぐ (SVG レンダラの `num()` と同じ正規化 — クロス検証テストで固定)。
fn nz(value: f64) -> f64 {
    value + 0.0
}

/// `MagiaGraph` + `LayoutResult` から配置済み IR を構築する。
///
/// 操作ドットの配置・シグネチャ円弧の生成条件は `midchilda.rs` の描画と
/// 同一規則 (`write_effects` / `write_defs` 参照)。4.3 で SVG レンダラを
/// 削除するとき、規則の所有権はこのモジュールへ移る。
#[must_use]
pub fn spell_ir(graph: &MagiaGraph, layout: &LayoutResult) -> SpellIr {
    let canvas = layout.canvas;
    let mut ir = SpellIr {
        view_box: [canvas.x0, -canvas.y1, canvas.width(), canvas.height()],
        rings: Vec::new(),
        glyphs: Vec::new(),
        edges: Vec::new(),
        signature: None,
        return_branch: None,
    };

    for module in &graph.modules {
        for edge in &module.edges {
            if edge.kind != EdgeKind::ControlFlow {
                continue;
            }
            ir.edges.push(EdgeIr {
                from: edge.source.0,
                to: edge.target.0,
            });
        }

        for sigil in &module.sigils {
            let center = screen_position(layout, sigil.id);
            let radius = sigil_radius(sigil.kind);
            match sigil.kind {
                SigilKind::MainRing | SigilKind::AuxRing => {
                    ir.rings
                        .push(ring_ir(module, layout, sigil, center, radius));
                    if sigil.kind == SigilKind::MainRing {
                        // signature / return_branch は上書き代入 — parse_function は
                        // 現状 1 モジュール (= MainRing 1つ) のみ返す前提。複数モジュール
                        // 対応 (Phase 4.5+) ではフィールドをモジュール単位へ移すこと。
                        push_type_info(&mut ir, module, layout, sigil, center, radius);
                    }
                }
                SigilKind::SummonGlyph | SigilKind::GateGlyph => {
                    let effect = sigil
                        .content
                        .first()
                        .map_or(EffectCategory::Pure, |op| palette::category_of(&op.effects));
                    let call_target = sigil
                        .content
                        .first()
                        .and_then(|op| op.payload.call_target.clone());
                    ir.glyphs.push(GlyphIr {
                        id: sigil.id.0,
                        x: nz(center.x),
                        y: nz(center.y),
                        radius,
                        effect,
                        call_target,
                        source_span: span_ir(&sigil.source_location),
                    });
                }
            }
        }
    }
    ir
}

/// リング1つ分 (操作ドットの配置・記号種別・早期リターン向きを含む)。
fn ring_ir(
    module: &Module,
    layout: &LayoutResult,
    sigil: &Sigil,
    center: kurbo::Point,
    radius: f64,
) -> RingIr {
    let is_main = sigil.kind == SigilKind::MainRing;
    let early_return = (early_return_count(sigil) > 0).then(|| {
        if is_main {
            // メインリングの早期リターンは 9 時 (流れの出口側)。
            [-1.0, 0.0]
        } else {
            let direction = outward_direction(module, layout, sigil.id);
            [direction.x, direction.y]
        }
    });
    let symbol = if is_main {
        None
    } else {
        match aux_kind(sigil) {
            Some(AuxRingKind::LoopBody(_)) => Some(RingSymbol::Loop),
            Some(_) => Some(RingSymbol::Branch),
            None => None,
        }
    };
    let track = (radius - OPERATION_DOT_INSET).max(6.0);
    let count = sigil.content.len();
    let operations = sigil
        .content
        .iter()
        .enumerate()
        .map(|(index, op)| {
            let angle = TAU * usize_to_f64(index) / usize_to_f64(count.max(1));
            OperationIr {
                // y 反転済み画面座標で反時計回りに見える向き。
                x: nz(center.x + track * angle.cos()),
                y: nz(center.y - track * angle.sin()),
                radius: OPERATION_DOT_RADIUS,
                effect: palette::category_of(&op.effects),
                source_span: op.payload.source_span.as_ref().and_then(span_ir),
            }
        })
        .collect();
    let is_async = sigil
        .layers
        .concurrency
        .as_ref()
        .is_some_and(|c| c.is_async);
    let guard_span = sigil
        .layers
        .control_flow
        .as_ref()
        .and_then(|info| info.role.as_ref())
        .and_then(|role| role.guard_location.as_ref())
        .and_then(span_ir);
    RingIr {
        id: sigil.id.0,
        role: if is_main {
            RingRole::Main
        } else {
            RingRole::Aux
        },
        x: nz(center.x),
        y: nz(center.y),
        radius,
        is_async,
        symbol,
        early_return,
        operations,
        guard_span,
    }
}

/// type-info 層 (シグネチャ円弧 + 戻り値分岐) の情報をメインリングから拾う。
/// 生成条件は `midchilda::write_defs` / `write_type_info` と同一規則。
fn push_type_info(
    ir: &mut SpellIr,
    module: &Module,
    layout: &LayoutResult,
    sigil: &Sigil,
    center: kurbo::Point,
    radius: f64,
) {
    let Some(type_info) = &sigil.layers.type_info else {
        return;
    };
    if let Some(text) = &type_info.signature {
        let arc_radius = signature_arc_radius(module, layout, sigil.id, center);
        let arc = Arc::new(center, Vec2::new(arc_radius, arc_radius), PI, PI, 0.0);
        ir.signature = Some(SignatureIr {
            text: text.clone(),
            arc_path: normalize_path_numbers(&arc.to_path(0.1).to_svg()),
        });
    }
    if type_info.returns_result || type_info.returns_option {
        ir.return_branch = Some([nz(center.x - radius), nz(center.y)]);
    }
}

// ===== ピン中心ビューのレイアウト (Phase 4.1, spec v0.3 §16 追補) =====

/// ピン中心ビューの全体配置。フォーカス魔法陣 (中央) の周囲に、近接度リングへ
/// 周辺関数チップを等角度で置く。配置は全てここ (Rust) で確定し、Vue は
/// `<g transform>` で描くだけ (POSD 分担 — spell_ir と同じ原則)。
#[derive(Serialize)]
pub struct FocusLayout {
    /// 周辺リングまで含めた全体の viewBox。
    pub view_box: [f64; 4],
    pub neighbors: Vec<NeighborChip>,
}

/// 周辺関数のチップ (縮小盾)。スタブ段階は1種 (円 + 関数名) で、
/// 距離は scale / opacity の差として現れる。3段階の縮小表現は Phase 4.2 の
/// 本実装近接度と合わせて精緻化する。
#[derive(Serialize)]
pub struct NeighborChip {
    pub qualified: String,
    pub name: String,
    pub signature: String,
    /// リング距離 (proximity::Neighbor と同値)。
    pub distance: u8,
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub opacity: f64,
    /// チップ円の半径 (スケール前)。
    pub radius: f64,
}

/// チップ円の基準半径 (スケール前)。
const CHIP_RADIUS: f64 = 44.0;
/// 中央魔法陣の外接円からリングまでのマージン。
const RING_1_MARGIN: f64 = 90.0;
const RING_2_MARGIN: f64 = 190.0;

/// フォーカスの viewBox と周辺リストから配置を計算する。
///
/// 角度は 12 時起点・時計回りの等角度割付。`neighbors` は呼び出し側で
/// (距離, 名前) ソート済み (`proximity::classify_neighbors`) — 同一入力からは
/// 常に同じ配置 (決定論)。
#[must_use]
pub fn focus_layout(
    focus_view_box: [f64; 4],
    neighbors: &[(crate::proximity::Neighbor, NeighborMeta)],
) -> FocusLayout {
    let [min_x, min_y, width, height] = focus_view_box;
    // viewBox の幾何学的中心 = 魔法陣の視覚的中心 (0,0) という前提
    // (layout::canvas は対称に取られる)。4.2 で非対称レイアウトが入るなら
    // 視覚的中心を別途受け取る形に変えること (レビュー I1 の記録)。
    let center_x = min_x + width / 2.0;
    let center_y = min_y + height / 2.0;
    let focus_radius = (width.max(height)) / 2.0;

    let ring_radius = |distance: u8| -> f64 {
        focus_radius
            + match distance {
                1 => RING_1_MARGIN,
                _ => RING_2_MARGIN,
            }
    };
    let ring_scale = |distance: u8| if distance == 1 { 0.55 } else { 0.35 };
    let ring_opacity = |distance: u8| if distance == 1 { 0.85 } else { 0.6 };

    // リングごとの member 数 (等角度割付の分母)。
    let count_in_ring = |distance: u8| {
        neighbors
            .iter()
            .filter(|(n, _)| n.distance == distance)
            .count()
    };

    let mut chips = Vec::with_capacity(neighbors.len());
    let mut index_in_ring = std::collections::BTreeMap::new();
    for (neighbor, meta) in neighbors {
        let total = count_in_ring(neighbor.distance).max(1);
        let index = index_in_ring.entry(neighbor.distance).or_insert(0usize);
        // 12時起点 (-90°)・時計回り。
        let angle = -std::f64::consts::FRAC_PI_2 + TAU * usize_to_f64(*index) / usize_to_f64(total);
        *index += 1;
        let radius = ring_radius(neighbor.distance);
        chips.push(NeighborChip {
            qualified: neighbor.qualified.clone(),
            name: meta.name.clone(),
            signature: meta.signature.clone(),
            distance: neighbor.distance,
            x: nz(center_x + radius * angle.cos()),
            y: nz(center_y + radius * angle.sin()),
            scale: ring_scale(neighbor.distance),
            opacity: ring_opacity(neighbor.distance),
            radius: CHIP_RADIUS,
        });
    }

    // 全体 viewBox: 最遠リング + スケール後チップ + ラベル余白まで広げる。
    let max_extent = if chips.is_empty() {
        focus_radius
    } else {
        focus_radius + RING_2_MARGIN + CHIP_RADIUS * 0.55 + 24.0
    };
    FocusLayout {
        view_box: [
            nz(center_x - max_extent),
            nz(center_y - max_extent),
            nz(max_extent * 2.0),
            nz(max_extent * 2.0),
        ],
        neighbors: chips,
    }
}

/// チップに表示する関数メタ (FunctionEntry への依存を持ち込まない最小写像)。
pub struct NeighborMeta {
    pub name: String,
    pub signature: String,
}
