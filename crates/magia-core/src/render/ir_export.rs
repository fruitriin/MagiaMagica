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
}

/// 召喚印 (外部呼び出し)。GateGlyph も描画上は同形なのでまとめる。
#[derive(Serialize)]
pub struct GlyphIr {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub effect: EffectCategory,
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
                    ir.glyphs.push(GlyphIr {
                        id: sigil.id.0,
                        x: nz(center.x),
                        y: nz(center.y),
                        radius,
                        effect,
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
            }
        })
        .collect();
    let is_async = sigil
        .layers
        .concurrency
        .as_ref()
        .is_some_and(|c| c.is_async);
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
