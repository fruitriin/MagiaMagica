//! 近接度モデル (Phase 4.2 本実装)。
//!
//! 「フォーカス関数から見て他の関数がどれくらい近いか」を連続値で数値化する。
//! 距離の意味論はこのモジュールに一元化し、リング離散化 (描画上の同心円割付) は
//! 描画側 (`render::ir_export::focus_layout`) のパラメータとして外出しする
//! (POSD 一般的インターフェース — UI が境界を変えてもモデルは不変)。
//!
//! 距離の序列 (計画 4.2 の設計判断):
//! - 同 impl/trait = 「同じオブジェクトの別側面」 = 最も近い (0.5)
//! - 呼び出し関係 = 「動作上連動する」 (直接 0.7、2ホップ 1.4)
//! - 同ファイル = 「人間が一緒に置いた」 (1.0)
//! - 複数経路で近いものは **min 合成** (内リングに入れたい関数を取りこぼさない)
//!
//! 本計画のスコープは同ファイル内のため、全関数が `WEIGHT_SAME_FILE` を天井に
//! 持つ (2ホップの 1.4 は min で食われて実質出ない — ファイル外へ拡張する
//! Phase 4.5 で生きる係数として正しく実装しておく)。

use std::collections::{BTreeMap, BTreeSet};

/// 距離係数。fixture で違和感が出たらここだけ調整する (計画の想定リスク対応)。
pub const WEIGHT_SAME_IMPL: f32 = 0.5;
pub const WEIGHT_CALL_DIRECT: f32 = 0.7;
pub const WEIGHT_SAME_FILE: f32 = 1.0;
pub const WEIGHT_CALL_2HOP: f32 = 1.4;

/// 近接度分類の入力 (関数メタの最小写像 — 呼び出し側の型に依存しない)。
#[derive(Debug, Clone)]
pub struct NeighborSeed {
    /// 一意キー (`Foo::bar` / `bar`)。
    pub qualified: String,
    /// impl ブロックの self 型。トップレベル関数は `None`。
    pub impl_context: Option<String>,
}

/// フォーカスから見た周辺関数 (連続距離つき)。
#[derive(Debug, Clone, PartialEq)]
pub struct Neighbor {
    pub qualified: String,
    /// 連続距離 (小さいほど近い)。同ファイルスコープでは
    /// `WEIGHT_SAME_IMPL ..= WEIGHT_SAME_FILE` に収まる。
    pub distance: f32,
}

/// フォーカス以外の関数を距離つきで分類する。
///
/// `call_edges` は (caller, callee) の qualified ペア (`magia_rust::call_graph`)。
/// 近接度では向きを区別せず無向の最短ホップを採る (向きは Phase 4.4 のレイヤー)。
///
/// 出力は (距離, 名前) 順にソート済み — 同一入力からは常に同じ並び (決定論、
/// リング上の等角度割付が再描画で動かないための前提)。
#[must_use]
pub fn classify_neighbors(
    focus: &NeighborSeed,
    all: &[NeighborSeed],
    call_edges: &[(String, String)],
) -> Vec<Neighbor> {
    let hops = call_hops(&focus.qualified, call_edges);
    let mut neighbors: Vec<Neighbor> = all
        .iter()
        .filter(|seed| seed.qualified != focus.qualified)
        .map(|seed| {
            let same_impl = focus.impl_context.is_some() && seed.impl_context == focus.impl_context;
            // min 合成: 複数経路で近いものは近い方を採る。
            let mut distance = WEIGHT_SAME_FILE;
            if same_impl {
                distance = distance.min(WEIGHT_SAME_IMPL);
            }
            match hops.get(seed.qualified.as_str()) {
                Some(1) => distance = distance.min(WEIGHT_CALL_DIRECT),
                Some(_) => distance = distance.min(WEIGHT_CALL_2HOP),
                None => {}
            }
            Neighbor {
                qualified: seed.qualified.clone(),
                distance,
            }
        })
        .collect();
    neighbors.sort_by(|a, b| {
        a.distance
            .total_cmp(&b.distance)
            .then_with(|| a.qualified.cmp(&b.qualified))
    });
    neighbors
}

/// focus からの無向最短ホップ数 (2ホップまで)。focus 自身は含まない —
/// 循環 (A → B → A) で自分に戻っても距離にならない。
/// 2段の手展開は意図的 (同ファイルスコープでは 2 ホップで十分 — 3 ホップ以上が
/// 要るファイル外スコープ (Phase 4.5) ではキュー式 BFS に書き直す)。
fn call_hops<'a>(focus: &str, call_edges: &'a [(String, String)]) -> BTreeMap<&'a str, u8> {
    let mut adjacency: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (a, b) in call_edges {
        adjacency.entry(a).or_default().insert(b);
        adjacency.entry(b).or_default().insert(a);
    }
    let mut hops: BTreeMap<&str, u8> = BTreeMap::new();
    let Some(direct) = adjacency.get(focus) else {
        return hops;
    };
    for &n in direct {
        if n != focus {
            hops.insert(n, 1);
        }
    }
    for &n in direct {
        let Some(second) = adjacency.get(n) else {
            continue;
        };
        for &m in second {
            if m != focus {
                hops.entry(m).or_insert(2);
            }
        }
    }
    hops
}

#[cfg(test)]
// 期待値は WEIGHT_* 定数の min 選択そのもの (演算を経ない) なのでビット同一が
// 保証される — 厳密比較が意図。
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    fn seed(qualified: &str, impl_context: Option<&str>) -> NeighborSeed {
        NeighborSeed {
            qualified: qualified.to_string(),
            impl_context: impl_context.map(String::from),
        }
    }

    fn edge(a: &str, b: &str) -> (String, String) {
        (a.to_string(), b.to_string())
    }

    fn distance_of(neighbors: &[Neighbor], qualified: &str) -> f32 {
        neighbors
            .iter()
            .find(|n| n.qualified == qualified)
            .expect("対象が分類結果にいる")
            .distance
    }

    #[test]
    fn same_impl_is_closest_others_same_file() {
        let focus = seed("Caster::cast", Some("Caster"));
        let all = [
            seed("Caster::cast", Some("Caster")),
            seed("Caster::charge", Some("Caster")),
            seed("free_fn", None),
            seed("Other::run", Some("Other")),
        ];
        let neighbors = classify_neighbors(&focus, &all, &[]);
        assert_eq!(distance_of(&neighbors, "Caster::charge"), WEIGHT_SAME_IMPL);
        assert_eq!(distance_of(&neighbors, "free_fn"), WEIGHT_SAME_FILE);
        assert_eq!(distance_of(&neighbors, "Other::run"), WEIGHT_SAME_FILE);
    }

    #[test]
    fn direct_call_is_closer_than_same_file() {
        let focus = seed("entry", None);
        let all = [
            seed("entry", None),
            seed("helper", None),
            seed("other", None),
        ];
        let edges = [edge("entry", "helper")];
        let neighbors = classify_neighbors(&focus, &all, &edges);
        assert_eq!(distance_of(&neighbors, "helper"), WEIGHT_CALL_DIRECT);
        assert_eq!(distance_of(&neighbors, "other"), WEIGHT_SAME_FILE);
    }

    #[test]
    fn callers_of_focus_are_also_close_undirected() {
        // 呼び出しは無向 — focus を呼ぶ側も同じ距離。
        let focus = seed("helper", None);
        let all = [seed("helper", None), seed("entry", None)];
        let edges = [edge("entry", "helper")];
        let neighbors = classify_neighbors(&focus, &all, &edges);
        assert_eq!(distance_of(&neighbors, "entry"), WEIGHT_CALL_DIRECT);
    }

    #[test]
    fn two_hop_call_is_capped_by_same_file_via_min() {
        // 2ホップ係数 (1.4) は同ファイル天井 (1.0) に min で食われる —
        // ファイル外スコープ (4.5) で生きる係数であることの固定。
        let focus = seed("a", None);
        let all = [seed("a", None), seed("b", None), seed("c", None)];
        let edges = [edge("a", "b"), edge("b", "c")];
        let neighbors = classify_neighbors(&focus, &all, &edges);
        assert_eq!(distance_of(&neighbors, "b"), WEIGHT_CALL_DIRECT);
        assert_eq!(
            distance_of(&neighbors, "c"),
            WEIGHT_SAME_FILE.min(WEIGHT_CALL_2HOP)
        );
    }

    #[test]
    fn cyclic_calls_do_not_loop_or_shrink_focus_distance() {
        // 循環 A → B → A: B は1ホップのまま、focus 自身は結果に出ない。
        let focus = seed("a", None);
        let all = [seed("a", None), seed("b", None)];
        let edges = [edge("a", "b"), edge("b", "a")];
        let neighbors = classify_neighbors(&focus, &all, &edges);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(distance_of(&neighbors, "b"), WEIGHT_CALL_DIRECT);
    }

    #[test]
    fn same_impl_and_call_take_min() {
        let focus = seed("Wand::cast", Some("Wand"));
        let all = [
            seed("Wand::cast", Some("Wand")),
            seed("Wand::charge", Some("Wand")),
        ];
        let edges = [edge("Wand::cast", "Wand::charge")];
        let neighbors = classify_neighbors(&focus, &all, &edges);
        // min(同impl 0.5, 呼び出し 0.7) = 0.5
        assert_eq!(distance_of(&neighbors, "Wand::charge"), WEIGHT_SAME_IMPL);
    }

    #[test]
    fn same_name_different_impl_are_distinct() {
        let focus = seed("Widget::render", Some("Widget"));
        let all = [
            seed("Widget::render", Some("Widget")),
            seed("Gadget::render", Some("Gadget")),
        ];
        let neighbors = classify_neighbors(&focus, &all, &[]);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(distance_of(&neighbors, "Gadget::render"), WEIGHT_SAME_FILE);
    }

    #[test]
    fn output_is_deterministic_sorted_by_distance_then_name() {
        let focus = seed("f", None);
        let all = [
            seed("f", None),
            seed("zeta", None),
            seed("alpha", None),
            seed("called", None),
        ];
        let edges = [edge("f", "called")];
        let names: Vec<_> = classify_neighbors(&focus, &all, &edges)
            .into_iter()
            .map(|n| n.qualified)
            .collect();
        assert_eq!(names, ["called", "alpha", "zeta"]);
    }
}
