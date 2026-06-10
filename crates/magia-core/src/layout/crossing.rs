//! 線分交差のカウントと判定 (Phase 1.5 の局所最適化用)。
//!
//! 接続線は全て「親 Sigil 中心 → 子 Sigil 中心」の線分として近似する。
//! 同一中心から出る放射状の線分どうしは交差しないため、交差は異なる親に属する
//! 線分間でのみ発生する。

use kurbo::Point;

/// 線分 (始点, 終点)。
pub(crate) type Segment = (Point, Point);

/// 線分集合の交差ペア数を数える。端点を共有するペアは交差と見なさない
/// (同じリングから出る放射線が中心で「交差」扱いになるのを防ぐ)。
pub(crate) fn count_crossings(segments: &[Segment]) -> usize {
    let mut count = 0;
    for (i, a) in segments.iter().enumerate() {
        for b in &segments[i + 1..] {
            if shares_endpoint(a, b) {
                continue;
            }
            if properly_intersect(a, b) {
                count += 1;
            }
        }
    }
    count
}

fn shares_endpoint(a: &Segment, b: &Segment) -> bool {
    points_eq(a.0, b.0) || points_eq(a.0, b.1) || points_eq(a.1, b.0) || points_eq(a.1, b.1)
}

/// 端点同一視の許容誤差。機械イプシロンでは三角関数の丸め誤差
/// (`sin` / `cos` 経由で得た座標) を吸収できないため、レイアウト座標の
/// スケール (数百) に対して十分小さい 1e-9 を使う。
const ENDPOINT_TOLERANCE: f64 = 1e-9;

fn points_eq(p: Point, q: Point) -> bool {
    (p.x - q.x).abs() < ENDPOINT_TOLERANCE && (p.y - q.y).abs() < ENDPOINT_TOLERANCE
}

/// 2線分が真に交差するか (端点接触・共線オーバーラップは交差と数えない近似)。
///
/// 符号付き面積 (外積) の符号が両線分でねじれていれば交差。Phase 1 の交差カウント
/// 用途では退化ケース (共線) を厳密に扱う必要がないため 0 判定は「交差なし」に倒す。
fn properly_intersect(a: &Segment, b: &Segment) -> bool {
    let d1 = cross(b.0, b.1, a.0);
    let d2 = cross(b.0, b.1, a.1);
    let d3 = cross(a.0, a.1, b.0);
    let d4 = cross(a.0, a.1, b.1);
    (d1 * d2 < 0.0) && (d3 * d4 < 0.0)
}

/// (q - p) × (r - p) の外積 z 成分。
fn cross(p: Point, q: Point, r: Point) -> f64 {
    (q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    #[test]
    fn crossing_pair_is_counted() {
        let segments = vec![(pt(-1.0, 0.0), pt(1.0, 0.0)), (pt(0.0, -1.0), pt(0.0, 1.0))];
        assert_eq!(count_crossings(&segments), 1);
    }

    #[test]
    fn parallel_segments_do_not_cross() {
        let segments = vec![(pt(0.0, 0.0), pt(1.0, 0.0)), (pt(0.0, 1.0), pt(1.0, 1.0))];
        assert_eq!(count_crossings(&segments), 0);
    }

    #[test]
    fn shared_endpoint_radial_lines_do_not_count() {
        // 同一中心から出る放射線。中心を共有するが交差ではない。
        let center = pt(0.0, 0.0);
        let segments = vec![
            (center, pt(1.0, 0.0)),
            (center, pt(0.0, 1.0)),
            (center, pt(-1.0, 0.0)),
        ];
        assert_eq!(count_crossings(&segments), 0);
    }

    #[test]
    fn touching_endpoint_is_not_proper_intersection() {
        // 端点が他方の線分上に乗るだけ (T字) は真の交差と数えない。
        let segments = vec![(pt(-1.0, 0.0), pt(1.0, 0.0)), (pt(0.0, 0.0), pt(0.0, 1.0))];
        assert_eq!(count_crossings(&segments), 0);
    }
}
