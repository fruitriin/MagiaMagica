//! 近接度モデル (Phase 4.1 スタブ / Phase 4.2 で本実装に差し替え)。
//!
//! 「フォーカス関数から見て他の関数がどれくらい近いか」をリング距離に離散化する
//! (spec: ピン中心ビューの太陽系モデル — 連続値でなくリング 1/2/3)。
//!
//! スタブの分類規則: 同 impl = 1、同ファイル = 2。呼び出し関係 (距離 3 相当の
//! 引き上げ) は Phase 4.2 でデータフロー IR から実装する。1実装の段階なので
//! trait にはしない (POSD: 過剰な抽象化をしない) — 4.2 では本関数の中身を差し替える。

/// 近接度分類の入力 (関数メタの最小写像 — 呼び出し側の型に依存しない)。
#[derive(Debug, Clone)]
pub struct NeighborSeed {
    /// 一意キー (`Foo::bar` / `bar`)。
    pub qualified: String,
    /// impl ブロックの self 型。トップレベル関数は `None`。
    pub impl_context: Option<String>,
}

/// フォーカスから見た周辺関数 (リング距離つき)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Neighbor {
    pub qualified: String,
    /// リング距離: 1 = 同 impl、2 = 同ファイル (スタブでは 3 は出ない)。
    pub distance: u8,
}

/// フォーカス以外の関数をリング距離に分類する。
///
/// 出力は (距離, 名前) 順にソート済み — 同一入力からは常に同じ並び (決定論、
/// リング上の等角度割付が再描画で動かないための前提)。
#[must_use]
pub fn classify_neighbors(focus: &NeighborSeed, all: &[NeighborSeed]) -> Vec<Neighbor> {
    let mut neighbors: Vec<Neighbor> = all
        .iter()
        .filter(|seed| seed.qualified != focus.qualified)
        .map(|seed| {
            let same_impl = focus.impl_context.is_some() && seed.impl_context == focus.impl_context;
            Neighbor {
                qualified: seed.qualified.clone(),
                distance: if same_impl { 1 } else { 2 },
            }
        })
        .collect();
    neighbors.sort_by(|a, b| (a.distance, &a.qualified).cmp(&(b.distance, &b.qualified)));
    neighbors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed(qualified: &str, impl_context: Option<&str>) -> NeighborSeed {
        NeighborSeed {
            qualified: qualified.to_string(),
            impl_context: impl_context.map(String::from),
        }
    }

    #[test]
    fn same_impl_is_distance_1_others_2() {
        let focus = seed("Caster::cast", Some("Caster"));
        let all = [
            seed("Caster::cast", Some("Caster")),
            seed("Caster::charge", Some("Caster")),
            seed("free_fn", None),
            seed("Other::run", Some("Other")),
        ];
        let neighbors = classify_neighbors(&focus, &all);
        assert_eq!(
            neighbors,
            vec![
                Neighbor {
                    qualified: "Caster::charge".into(),
                    distance: 1
                },
                Neighbor {
                    qualified: "Other::run".into(),
                    distance: 2
                },
                Neighbor {
                    qualified: "free_fn".into(),
                    distance: 2
                },
            ]
        );
    }

    #[test]
    fn toplevel_focus_has_no_distance_1() {
        let focus = seed("free_fn", None);
        let all = [seed("free_fn", None), seed("another", None)];
        let neighbors = classify_neighbors(&focus, &all);
        assert!(neighbors.iter().all(|n| n.distance == 2));
    }

    #[test]
    fn output_is_deterministic_sorted() {
        let focus = seed("f", None);
        let all = [seed("f", None), seed("zeta", None), seed("alpha", None)];
        let names: Vec<_> = classify_neighbors(&focus, &all)
            .into_iter()
            .map(|n| n.qualified)
            .collect();
        assert_eq!(names, ["alpha", "zeta"]);
    }
}
