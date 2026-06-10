//! ミッドチルダ式 ConcentricRings レンダラ (Phase 1.6, spec §6.1.2 / §6.1.3)。
//!
//! 描画規約:
//! - レイアウトの数学座標を `y_svg = -y_math` で画面座標へ変換する。これにより
//!   spec §6.1.2 の「3時起点・反時計回り」が画面上でもそのまま保たれる
//! - レイヤーは `<g class="layer-control-flow">` (リング・接続線・制御記号)、
//!   `<g class="layer-effects">` (Operation ドット・召喚記号)、
//!   `<g class="layer-type-info">` (シグネチャ円弧・戻り値分岐線) の3層 (spec §6.1.5)。
//!   async の二重線はシグネチャ由来 (`async fn`) だが、リングの輪郭であるため
//!   control-flow 層に置く
//! - 属性順序は固定、数値は小数2桁丸め。同じ入力からは常に同一文字列が出る
//! - 出力は1行1要素。レイヤー `<g>` の内側に `<g>` は入れ子にせず、閉じタグは
//!   `</g>` 単独行とする (magia-cli の `--layers` フィルタが行単位でこの規約に依存)

use std::f64::consts::{PI, TAU};
use std::fmt::Write;

use kurbo::{Arc, Point, Shape, Vec2};

use crate::ir::{AuxRingKind, MagiaGraph, Module, Sigil, SigilId, SigilKind};
use crate::layout::constants::{
    ASYNC_INNER_RING_OFFSET, AUX_RING_STROKE, EDGE_STROKE, MAIN_RING_STROKE, OPERATION_DOT_INSET,
    OPERATION_DOT_RADIUS, RETURN_BRANCH_LENGTH, SIGNATURE_ARC_OFFSET,
};
use crate::layout::{LayoutResult, sigil_radius};
use crate::render::palette;

pub(crate) fn render(graph: &MagiaGraph, layout: &LayoutResult) -> String {
    let mut out = String::new();
    write_document(&mut out, graph, layout).expect("String への SVG 書き込みは失敗しない");
    out
}

fn write_document(out: &mut String, graph: &MagiaGraph, layout: &LayoutResult) -> std::fmt::Result {
    let canvas = layout.canvas;
    writeln!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}">"#,
        num(canvas.x0),
        num(-canvas.y1),
        num(canvas.width()),
        num(canvas.height()),
    )?;
    write_defs(out, graph, layout)?;

    writeln!(out, r#"<g class="layer-control-flow">"#)?;
    for module in &graph.modules {
        write_control_flow(out, module, layout)?;
    }
    writeln!(out, "</g>")?;

    writeln!(out, r#"<g class="layer-effects">"#)?;
    for module in &graph.modules {
        write_effects(out, module, layout)?;
    }
    writeln!(out, "</g>")?;

    writeln!(out, r#"<g class="layer-type-info">"#)?;
    for module in &graph.modules {
        write_type_info(out, module, layout)?;
    }
    writeln!(out, "</g>")?;

    writeln!(out, "</svg>")
}

/// シグネチャ円弧ラベル用の `<path>` を `<defs>` に出す。
/// 左 (9時) から頂上を通って右 (3時) へ向かう上半円。`kurbo::Arc` →
/// `BezPath::to_svg()` で文字列化する (tech-selection §2.3)。
///
/// 不変条件: ここで `<path id="sig-arc-*">` を出す条件 (MainRing かつ signature あり)
/// は `write_type_info` が `<textPath href>` を出す条件と完全一致させること。
/// 片方だけ変更すると参照切れ・浮き path が生じる。
fn write_defs(out: &mut String, graph: &MagiaGraph, layout: &LayoutResult) -> std::fmt::Result {
    writeln!(out, "<defs>")?;
    for module in &graph.modules {
        for sigil in &module.sigils {
            if sigil.kind != SigilKind::MainRing {
                continue;
            }
            let has_signature = sigil
                .layers
                .type_info
                .as_ref()
                .is_some_and(|t| t.signature.is_some());
            if !has_signature {
                continue;
            }
            let center = screen_position(layout, sigil.id);
            let radius = sigil_radius(sigil.kind) + SIGNATURE_ARC_OFFSET;
            // 画面座標で start=π (左) から +π 掃引すると中点が 3π/2 = 真上を通る。
            let arc = Arc::new(center, Vec2::new(radius, radius), PI, PI, 0.0);
            // to_svg() は最短浮動小数表現でノイズ (1.6e-14 等) を含むため、
            // 他の属性と同じ2桁固定に正規化してスナップショットを安定させる。
            let d = normalize_path_numbers(&arc.to_path(0.1).to_svg());
            writeln!(
                out,
                r#"<path id="sig-arc-{}-{}" d="{d}" fill="none"/>"#,
                module.id.0, sigil.id.0
            )?;
        }
    }
    writeln!(out, "</defs>")
}

// ===== layer-control-flow =====

fn write_control_flow(
    out: &mut String,
    module: &Module,
    layout: &LayoutResult,
) -> std::fmt::Result {
    // 接続線を先に描き、リングが上に重なるようにする。
    for edge in &module.edges {
        let from = screen_position(layout, edge.source);
        let to = screen_position(layout, edge.target);
        let from_radius = module_radius(module, edge.source);
        let to_radius = module_radius(module, edge.target);
        let delta = to - from;
        let length = delta.hypot();
        if length < 1e-6 {
            continue; // 同一点 (壊れた IR への防御)
        }
        let unit = delta / length;
        let start = from + unit * from_radius;
        let end = to - unit * to_radius;
        writeln!(
            out,
            r##"<line class="edge-control-flow" x1="{}" y1="{}" x2="{}" y2="{}" stroke="#000000" stroke-width="{}"/>"##,
            num(start.x),
            num(start.y),
            num(end.x),
            num(end.y),
            num(EDGE_STROKE),
        )?;
    }

    for sigil in &module.sigils {
        let center = screen_position(layout, sigil.id);
        match sigil.kind {
            SigilKind::MainRing => {
                let radius = sigil_radius(sigil.kind);
                circle(
                    out,
                    "main-ring",
                    center,
                    radius,
                    "#000000",
                    MAIN_RING_STROKE,
                )?;
                // async fn は二重線 (spec §6.1.3)。
                let is_async = sigil
                    .layers
                    .concurrency
                    .as_ref()
                    .is_some_and(|c| c.is_async);
                if is_async {
                    circle(
                        out,
                        "main-ring-async",
                        center,
                        radius - ASYNC_INNER_RING_OFFSET,
                        "#000000",
                        MAIN_RING_STROKE * 0.5,
                    )?;
                }
                if early_return_count(sigil) > 0 {
                    // MainRing の早期リターンは 9時 (流れの出口側) に描く。
                    early_return_arrow(out, center, Vec2::new(-1.0, 0.0), radius)?;
                }
            }
            SigilKind::AuxRing => {
                let radius = sigil_radius(sigil.kind);
                circle(out, "aux-ring", center, radius, "#000000", AUX_RING_STROKE)?;
                match aux_kind(sigil) {
                    Some(AuxRingKind::LoopBody(_)) => loop_symbol(out, center, radius)?,
                    Some(_) => branch_symbol(out, center)?,
                    None => {}
                }
                if early_return_count(sigil) > 0 {
                    let direction = outward_direction(module, layout, sigil.id);
                    early_return_arrow(out, center, direction, radius)?;
                }
            }
            SigilKind::SummonGlyph | SigilKind::GateGlyph => {} // effects 層で描く
        }
    }
    Ok(())
}

/// 分岐 (if/match): 二股に分かれる線の合流点 (spec §6.1.3)。リング中央の Y 字。
fn branch_symbol(out: &mut String, c: Point) -> std::fmt::Result {
    writeln!(
        out,
        r##"<path class="sym-branch" d="M{} {} L{} {} M{} {} L{} {} M{} {} L{} {}" stroke="#000000" stroke-width="1" fill="none"/>"##,
        num(c.x),
        num(c.y + 9.0),
        num(c.x),
        num(c.y),
        num(c.x),
        num(c.y),
        num(c.x - 7.0),
        num(c.y - 8.0),
        num(c.x),
        num(c.y),
        num(c.x + 7.0),
        num(c.y - 8.0),
    )
}

/// ループ: 円形リング内側の小さな矢印 (spec §6.1.3)。
/// リング内周上部に、反時計回り (画面上は左向き) の三角形を置く。
fn loop_symbol(out: &mut String, c: Point, radius: f64) -> std::fmt::Result {
    let track = radius - 10.0;
    let y = c.y - track;
    writeln!(
        out,
        r##"<polygon class="sym-loop" points="{},{} {},{} {},{}" fill="#000000"/>"##,
        num(c.x + 5.0),
        num(y - 4.0),
        num(c.x + 5.0),
        num(y + 4.0),
        num(c.x - 5.0),
        num(y),
    )
}

/// 早期リターン: リング内側から外へ抜ける矢印 (spec §6.1.3)。
fn early_return_arrow(out: &mut String, c: Point, unit: Vec2, radius: f64) -> std::fmt::Result {
    let inner = c + unit * (radius - 10.0);
    let tip = c + unit * (radius + 12.0);
    let perp = Vec2::new(-unit.y, unit.x);
    let base = tip - unit * 6.0;
    let wing_a = base + perp * 4.0;
    let wing_b = base - perp * 4.0;
    writeln!(
        out,
        r##"<line class="sym-early-return" x1="{}" y1="{}" x2="{}" y2="{}" stroke="#000000" stroke-width="1.5"/>"##,
        num(inner.x),
        num(inner.y),
        num(tip.x),
        num(tip.y),
    )?;
    writeln!(
        out,
        r##"<polygon class="sym-early-return" points="{},{} {},{} {},{}" fill="#000000"/>"##,
        num(tip.x),
        num(tip.y),
        num(wing_a.x),
        num(wing_a.y),
        num(wing_b.x),
        num(wing_b.y),
    )
}

// ===== layer-effects =====

fn write_effects(out: &mut String, module: &Module, layout: &LayoutResult) -> std::fmt::Result {
    for sigil in &module.sigils {
        let center = screen_position(layout, sigil.id);
        match sigil.kind {
            // リング内の処理列: 3時起点・反時計回りに Operation ドットを並べる
            // (spec §6.1.2)。色は Operation の効果カテゴリ。
            SigilKind::MainRing | SigilKind::AuxRing => {
                let ring_radius = sigil_radius(sigil.kind);
                let track = (ring_radius - OPERATION_DOT_INSET).max(6.0);
                let count = sigil.content.len();
                for (index, op) in sigil.content.iter().enumerate() {
                    let angle = TAU * usize_to_f64(index) / usize_to_f64(count.max(1));
                    // y 反転済み画面座標で反時計回りに見える向き。
                    let dot = Point::new(
                        center.x + track * angle.cos(),
                        center.y - track * angle.sin(),
                    );
                    writeln!(
                        out,
                        r#"<circle class="op-dot" cx="{}" cy="{}" r="{}" fill="{}"/>"#,
                        num(dot.x),
                        num(dot.y),
                        num(OPERATION_DOT_RADIUS),
                        palette::effect_color(&op.effects),
                    )?;
                }
            }
            // 召喚記号: 効果カテゴリ色で塗った小円 (spec §6.1.2 / §6.1.3)。
            SigilKind::SummonGlyph | SigilKind::GateGlyph => {
                let radius = sigil_radius(sigil.kind);
                let color = sigil
                    .content
                    .first()
                    .map_or(palette::PURE, |op| palette::effect_color(&op.effects));
                writeln!(
                    out,
                    r#"<circle class="summon-glyph" cx="{}" cy="{}" r="{}" fill="{color}"/>"#,
                    num(center.x),
                    num(center.y),
                    num(radius),
                )?;
            }
        }
    }
    Ok(())
}

// ===== layer-type-info =====

fn write_type_info(out: &mut String, module: &Module, layout: &LayoutResult) -> std::fmt::Result {
    for sigil in &module.sigils {
        if sigil.kind != SigilKind::MainRing {
            continue;
        }
        let Some(type_info) = &sigil.layers.type_info else {
            continue;
        };
        let center = screen_position(layout, sigil.id);
        let radius = sigil_radius(sigil.kind);

        // 関数シグネチャ: 最外周の円弧ラベル (spec §6.1.3)。
        if let Some(signature) = &type_info.signature {
            writeln!(
                out,
                r##"<text class="signature" font-size="11" fill="#000000"><textPath href="#sig-arc-{}-{}" startOffset="50%" text-anchor="middle">{}</textPath></text>"##,
                module.id.0,
                sigil.id.0,
                escape_xml(signature),
            )?;
        }

        // Result / Option 戻り値: 9時から出る正常/異常の分岐線 (spec §6.1.3)。
        // 異常パス (Err / None) は破線で区別する。
        if type_info.returns_result || type_info.returns_option {
            let start = Point::new(center.x - radius, center.y);
            let ok_end = Point::new(
                start.x - RETURN_BRANCH_LENGTH,
                start.y - RETURN_BRANCH_LENGTH * 0.45,
            );
            let err_end = Point::new(
                start.x - RETURN_BRANCH_LENGTH,
                start.y + RETURN_BRANCH_LENGTH * 0.45,
            );
            writeln!(
                out,
                r##"<line class="return-path-ok" x1="{}" y1="{}" x2="{}" y2="{}" stroke="#000000" stroke-width="1"/>"##,
                num(start.x),
                num(start.y),
                num(ok_end.x),
                num(ok_end.y),
            )?;
            writeln!(
                out,
                r##"<line class="return-path-err" x1="{}" y1="{}" x2="{}" y2="{}" stroke="#000000" stroke-width="1" stroke-dasharray="4 3"/>"##,
                num(start.x),
                num(start.y),
                num(err_end.x),
                num(err_end.y),
            )?;
        }
    }
    Ok(())
}

// ===== 共通ヘルパー =====

fn circle(
    out: &mut String,
    class: &str,
    c: Point,
    r: f64,
    stroke: &str,
    stroke_width: f64,
) -> std::fmt::Result {
    writeln!(
        out,
        r#"<circle class="{class}" cx="{}" cy="{}" r="{}" fill="none" stroke="{stroke}" stroke-width="{}"/>"#,
        num(c.x),
        num(c.y),
        num(r),
        num(stroke_width),
    )
}

/// レイアウト座標 (数学系) を画面座標へ変換する。未配置 (壊れた IR) は原点。
fn screen_position(layout: &LayoutResult, id: SigilId) -> Point {
    layout
        .positions
        .get(&id)
        .map_or(Point::ZERO, |p| Point::new(p.x, -p.y))
}

// Edge ごとの線形探索は Phase 1 の Sigil 数 (数十) では問題ない。
// 複数関数レンダリング (Phase 2+) でスケールしたら id → Sigil の Map 化を検討する。
fn module_radius(module: &Module, id: SigilId) -> f64 {
    module
        .sigils
        .iter()
        .find(|s| s.id == id)
        .map_or(0.0, |s| sigil_radius(s.kind))
}

/// AuxRing が親から離れる方向 (画面座標)。親が見つからなければ 3時方向。
fn outward_direction(module: &Module, layout: &LayoutResult, id: SigilId) -> Vec2 {
    let Some(edge) = module.edges.iter().find(|e| e.target == id) else {
        return Vec2::new(1.0, 0.0);
    };
    let parent = screen_position(layout, edge.source);
    let child = screen_position(layout, id);
    let delta = child - parent;
    let length = delta.hypot();
    if length < 1e-6 {
        Vec2::new(1.0, 0.0)
    } else {
        delta / length
    }
}

fn aux_kind(sigil: &Sigil) -> Option<AuxRingKind> {
    sigil
        .layers
        .control_flow
        .as_ref()
        .and_then(|c| c.role.as_ref())
        .map(|role| role.kind)
}

fn early_return_count(sigil: &Sigil) -> u32 {
    sigil
        .layers
        .control_flow
        .as_ref()
        .map_or(0, |c| c.early_return_count)
}

/// 数値の文字列化: 小数2桁固定で属性値を安定させる。`-0.00` は `0.00` に正規化。
fn num(value: f64) -> String {
    let formatted = format!("{value:.2}");
    if formatted == "-0.00" {
        "0.00".to_string()
    } else {
        formatted
    }
}

/// SVG パス文字列中の数値を `num()` と同じ2桁固定に揃える。
/// コマンド文字 (M/C/L 等) と区切りはそのまま通す。数値として解釈できない
/// トークンは無加工で残す (防御)。
fn normalize_path_numbers(d: &str) -> String {
    let mut out = String::new();
    let mut token = String::new();
    for ch in d.chars() {
        let continues_number = ch.is_ascii_digit()
            || ch == '.'
            || ch == 'e'
            || ch == 'E'
            || (ch == '-' && token.ends_with(['e', 'E']))
            || (ch == '+' && token.ends_with(['e', 'E']));
        if continues_number {
            token.push(ch);
        } else if ch == '-' && token.is_empty() {
            token.push(ch); // 負号で数値開始
        } else if ch == '-' {
            flush_number(&mut out, &mut token); // "5-3" 形式: 直前の数値を確定
            token.push(ch);
        } else {
            flush_number(&mut out, &mut token);
            out.push(ch);
        }
    }
    flush_number(&mut out, &mut token);
    out
}

fn flush_number(out: &mut String, token: &mut String) {
    if token.is_empty() {
        return;
    }
    match token.parse::<f64>() {
        Ok(value) => out.push_str(&num(value)),
        Err(_) => out.push_str(token),
    }
    token.clear();
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// レイアウト側と同じ判断: カウントは 2^53 未満なので精度劣化は起きない。
#[allow(clippy::cast_precision_loss)]
fn usize_to_f64(value: usize) -> f64 {
    value as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn num_normalizes_negative_zero() {
        assert_eq!(num(-0.0001), "0.00");
        // 1.005 は二進表現では 1.00499... のため format! は "1.00" を返す
        // (丸めモードの問題ではなく浮動小数表現の誤差)。
        assert_eq!(num(1.005), "1.00");
        assert_eq!(num(-3.5), "-3.50");
    }

    #[test]
    fn normalize_path_numbers_rounds_noise() {
        assert_eq!(
            normalize_path_numbers("M-132 0.000000000000016165337748745062"),
            "M-132.00 0.00"
        );
        assert_eq!(
            normalize_path_numbers("C1.5,2 3 4.125"),
            "C1.50,2.00 3.00 4.12"
        );
        // 区切りなしの負数 (コンパクト SVG 形式) も分割できる。
        assert_eq!(normalize_path_numbers("L5-3"), "L5.00-3.00");
        // 指数表記も2桁固定へ。
        assert_eq!(normalize_path_numbers("M1e-13 2E+1"), "M0.00 20.00");
    }

    #[test]
    fn escape_xml_handles_signature_characters() {
        assert_eq!(
            escape_xml("fn f(a: &str) -> Result<i32, String>"),
            "fn f(a: &amp;str) -&gt; Result&lt;i32, String&gt;"
        );
    }
}
