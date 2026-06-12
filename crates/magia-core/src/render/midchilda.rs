//! ミッドチルダ式の幾何ヘルパ (Phase 1.6 起源、Phase 4.3 M5 で SVG 出力を削除)。
//!
//! **SVG 文字列の生成は Vue SSR (`web/src/render/ssr.ts` + MagicCircle ツリー) へ
//! 一本化済み** — 本モジュールに残るのは `ir_export` が使う幾何計算
//! (画面座標変換・外向き方向・シグネチャ円弧の衝突回避半径) と
//! 数値正規化 (`num` の2桁固定は IR の `nz` と同精度規約) のみ。

use kurbo::{Point, Vec2};

use crate::ir::{AuxRingKind, EdgeKind, Module, Sigil, SigilId, SigilKind};
use crate::layout::constants::{SIGNATURE_ARC_OFFSET, SIGNATURE_BAND_HALF};
use crate::layout::{LayoutResult, sigil_radius};

/// シグネチャ円弧の半径 (他リングとの交差を避けて外へ逃がす、spec §6.1.5)。
pub(crate) fn signature_arc_radius(
    module: &Module,
    layout: &LayoutResult,
    main_id: SigilId,
    center: Point,
) -> f64 {
    let mut radius = sigil_radius(SigilKind::MainRing) + SIGNATURE_ARC_OFFSET;
    for _ in 0..module.sigils.len() {
        let mut extended = false;
        for other in &module.sigils {
            if other.id == main_id {
                continue;
            }
            let position = screen_position(layout, other.id);
            if position.y >= center.y {
                continue; // 下半分は円弧と交差しない
            }
            let distance = (position - center).hypot();
            let other_radius = sigil_radius(other.kind);
            let (low, high) = (distance - other_radius, distance + other_radius);
            if low < radius + SIGNATURE_BAND_HALF && high > radius - SIGNATURE_BAND_HALF {
                radius = high + SIGNATURE_ARC_OFFSET;
                extended = true;
            }
        }
        if !extended {
            break;
        }
    }
    radius
}

pub(crate) fn screen_position(layout: &LayoutResult, id: SigilId) -> Point {
    layout
        .positions
        .get(&id)
        .map_or(Point::ZERO, |p| Point::new(p.x, -p.y))
}

/// AuxRing が親から離れる方向 (画面座標)。親が見つからなければ 3時方向。
pub(crate) fn outward_direction(module: &Module, layout: &LayoutResult, id: SigilId) -> Vec2 {
    // 親子関係は ControlFlow Edge のみ (DataFlow Edge が先に並ぶ入力でも誤らない)。
    let Some(edge) = module
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::ControlFlow && e.target == id)
    else {
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

pub(crate) fn aux_kind(sigil: &Sigil) -> Option<AuxRingKind> {
    sigil
        .layers
        .control_flow
        .as_ref()
        .and_then(|c| c.role.as_ref())
        .map(|role| role.kind)
}

pub(crate) fn early_return_count(sigil: &Sigil) -> u32 {
    sigil
        .layers
        .control_flow
        .as_ref()
        .map_or(0, |c| c.early_return_count)
}

/// 数値の文字列化: 小数2桁固定で属性値を安定させる。`-0.00` は `0.00` に正規化。
/// belka レンダラとも共有する (出力数値の規約は式をまたいで統一)。
pub(crate) fn num(value: f64) -> String {
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
pub(crate) fn normalize_path_numbers(d: &str) -> String {
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

#[allow(clippy::cast_precision_loss)]
pub(crate) fn usize_to_f64(value: usize) -> f64 {
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
}
