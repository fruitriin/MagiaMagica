//! 変数スコープ追跡とデータフロー近似 (Phase 3.4, spec §5.1 `data_flow`)。
//!
//! syn ベースの**意味解決なしの近似**: `let` 束縛 = def、識別子の出現 = use、
//! 再代入 (`x = e` / `x += e`) = 再定義、シャドーイングは新しい def として扱う。
//! 近似の精度より**決定論と説明可能性**を優先する (計画の設計判断) — どの構文から
//! def/use を取ったかは `OperationPayload::{defs, uses}` に残り、デバッグ可能。
//!
//! 追わないもの (Phase 3 のスコープ外):
//! - クロージャ本体 (visitor が `visit_expr_closure` で再帰を止める)
//! - マクロ内のトークン列 (syn の既定どおり未解析)
//! - 非ローカルへの代入 (`self.x = e` / `arr[i] = e` の lhs は use のみ計上)
//! - 借用・寿命 (LifetimeInfo は Phase 5+)

use std::collections::{BTreeMap, BTreeSet};

use magia_core::ir::{DataFlowInfo, Edge, EdgeDataFlowInfo, EdgeKind, EdgeLayerData, SigilId};
use syn::visit::Visit;
use syn::{BinOp, Expr, Pat, Stmt};

// ===== 構文からの def/use 候補抽出 (スコープを知らない純粋構文解析) =====

/// statement から構文的に取れる def / use 候補。
/// `uses` は**候補**であり、スコープで解決できたものだけが採用される
/// (関数名・unit variant・定数はスコープに無いため自然に落ちる)。
pub(crate) struct StmtDataflow {
    /// `let` 束縛が生む新しい変数。
    pub(crate) defs: Vec<String>,
    /// 再代入される既存変数 (`x = e` / `x += e`)。
    pub(crate) reassigns: Vec<String>,
    /// 読み取り出現の候補。
    pub(crate) uses: Vec<String>,
}

pub(crate) fn stmt_dataflow(stmt: &Stmt) -> StmtDataflow {
    match stmt {
        Stmt::Local(local) => {
            let mut uses = Vec::new();
            if let Some(init) = &local.init {
                uses.extend(expr_use_candidates(&init.expr));
                if let Some((_, diverge)) = &init.diverge {
                    // let-else の else 節。中の def はブロックローカルだが、
                    // 必ず発散するため後続への漏れはない (use のみ拾う近似)。
                    uses.extend(expr_use_candidates(diverge));
                }
            }
            StmtDataflow {
                defs: pattern_idents(&local.pat),
                reassigns: Vec::new(),
                uses,
            }
        }
        // x = e: lhs が単純な識別子なら再定義。それ以外 (self.x / arr[i]) は
        // lhs 中の変数を use として拾う (非ローカルへの代入は追わない)。
        Stmt::Expr(Expr::Assign(assign), _) => {
            let mut reassigns = Vec::new();
            let mut uses = expr_use_candidates(&assign.right);
            match single_ident(&assign.left) {
                Some(name) => reassigns.push(name),
                None => uses.extend(expr_use_candidates(&assign.left)),
            }
            StmtDataflow {
                defs: Vec::new(),
                reassigns,
                uses,
            }
        }
        // x += e (syn 2 では複合代入も ExprBinary): lhs は読み + 再定義。
        Stmt::Expr(Expr::Binary(binary), _) if is_assign_op(binary.op) => {
            let mut uses = expr_use_candidates(&binary.right);
            let mut reassigns = Vec::new();
            match single_ident(&binary.left) {
                Some(name) => {
                    uses.push(name.clone());
                    reassigns.push(name);
                }
                None => uses.extend(expr_use_candidates(&binary.left)),
            }
            StmtDataflow {
                defs: Vec::new(),
                reassigns,
                uses,
            }
        }
        Stmt::Expr(expr, _) => StmtDataflow {
            defs: Vec::new(),
            reassigns: Vec::new(),
            uses: expr_use_candidates(expr),
        },
        // ネスト item は別スコープ、マクロ statement は未解析 (近似の範囲外)。
        Stmt::Item(_) | Stmt::Macro(_) => StmtDataflow {
            defs: Vec::new(),
            reassigns: Vec::new(),
            uses: Vec::new(),
        },
    }
}

/// 式が単純な単一識別子 (`x`) ならその名前を返す。
fn single_ident(expr: &Expr) -> Option<String> {
    let Expr::Path(path) = expr else {
        return None;
    };
    if path.qself.is_some() || path.path.segments.len() != 1 {
        return None;
    }
    let segment = path.path.segments.first()?;
    segment
        .arguments
        .is_empty()
        .then(|| segment.ident.to_string())
}

fn is_assign_op(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::AddAssign(_)
            | BinOp::SubAssign(_)
            | BinOp::MulAssign(_)
            | BinOp::DivAssign(_)
            | BinOp::RemAssign(_)
            | BinOp::BitXorAssign(_)
            | BinOp::BitAndAssign(_)
            | BinOp::BitOrAssign(_)
            | BinOp::ShlAssign(_)
            | BinOp::ShrAssign(_)
    )
}

/// 式の中の「変数の読み取り候補」= 単一セグメント・型引数なしのパス出現。
/// スコープ解決前なので関数名や unit variant も混ざるが、解決時に自然に落ちる。
pub(crate) fn expr_use_candidates(expr: &Expr) -> Vec<String> {
    let mut visitor = UseCandidates::default();
    visitor.visit_expr(expr);
    visitor.names
}

#[derive(Default)]
struct UseCandidates {
    names: Vec<String>,
}

impl<'ast> Visit<'ast> for UseCandidates {
    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if node.qself.is_none()
            && node.path.segments.len() == 1
            && let Some(segment) = node.path.segments.first()
            && segment.arguments.is_empty()
        {
            self.names.push(segment.ident.to_string());
        }
    }

    fn visit_expr_closure(&mut self, _node: &'ast syn::ExprClosure) {
        // クロージャ内は Phase 3 では追わない (計画のスコープ)。
    }
}

/// パターンが束縛する変数名 (再帰)。
/// 先頭大文字の単独識別子は unit variant の可能性が高いため除外する
/// (syn は意味解決をしないため `Some(x)` の `Variant` と束縛を区別できない近似)。
pub(crate) fn pattern_idents(pat: &Pat) -> Vec<String> {
    let mut visitor = PatIdents::default();
    visitor.visit_pat(pat);
    visitor.names
}

#[derive(Default)]
struct PatIdents {
    names: Vec<String>,
}

impl<'ast> Visit<'ast> for PatIdents {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        let name = node.ident.to_string();
        if !name.starts_with(char::is_uppercase) {
            self.names.push(name);
        }
        syn::visit::visit_pat_ident(self, node);
    }
}

// ===== スコープ追跡 (リング構築と並走する状態機械) =====

/// 関数1本分の変数スコープと use-def の記録。
///
/// `RingBuilder` がリングを再帰構築する間、フレームの push/pop で字句スコープを
/// 模倣する。def は「どのリングで生まれたか」を持ち、use の解決時にリングが
/// 異なれば (def リング → use リング) のフローとして積む。
#[derive(Default)]
pub(crate) struct ScopeTracker {
    /// 字句スコープのフレーム列。値は `records` への添字。
    frames: Vec<BTreeMap<String, usize>>,
    records: Vec<DefRecord>,
    /// (def リング, use リング) → 流れた変数名。BTreeMap で決定論的に列挙できる。
    flows: BTreeMap<(SigilId, SigilId), BTreeSet<String>>,
}

/// def 1件の記録。変数名は `frames` のキーと `flows` 側が持つためここには置かない。
struct DefRecord {
    ring: SigilId,
    /// この def に触れた Operation 数 (def 1 + use 回数) = チェーン長。
    touches: u32,
    used: bool,
}

impl ScopeTracker {
    pub(crate) fn push_frame(&mut self) {
        self.frames.push(BTreeMap::new());
    }

    pub(crate) fn pop_frame(&mut self) {
        self.frames.pop();
    }

    /// 新しい変数を現在のフレームに定義する (シャドーイングは新 def)。
    pub(crate) fn define(&mut self, ring: SigilId, name: &str) {
        let index = self.push_record(ring);
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name.to_string(), index);
        }
    }

    /// 既存変数の再代入 = 再定義。変数が見えるフレームの位置はそのままに、
    /// 新しい def レコード (再定義が起きたリング由来) へ差し替える。
    /// ループ内の `total += ...` が「ループ → 親」の流れを作る根拠になる。
    /// 非ローカル (スコープに無い名前) への代入は無視する。
    pub(crate) fn redefine(&mut self, ring: SigilId, name: &str) {
        let index = self.push_record(ring);
        for frame in self.frames.iter_mut().rev() {
            if let Some(slot) = frame.get_mut(name) {
                *slot = index;
                return;
            }
        }
        // どのフレームにも無い: レコードは積んだが誰からも参照されない (集計対象外)。
        self.records.pop();
    }

    /// 変数の読み取り。スコープで解決できたら true。
    /// def と異なるリングでの use はクロスリングのデータフローとして記録する。
    pub(crate) fn use_var(&mut self, ring: SigilId, name: &str) -> bool {
        let Some(&index) = self.frames.iter().rev().find_map(|frame| frame.get(name)) else {
            return false;
        };
        let record = &mut self.records[index];
        record.touches += 1;
        record.used = true;
        if record.ring != ring {
            self.flows
                .entry((record.ring, ring))
                .or_default()
                .insert(name.to_string());
        }
        true
    }

    /// リング単位の集計 (spec §5.1: チェーン数 = def されて使われた変数の数)。
    pub(crate) fn ring_stats(&self) -> BTreeMap<SigilId, DataFlowInfo> {
        let mut stats: BTreeMap<SigilId, DataFlowInfo> = BTreeMap::new();
        for record in &self.records {
            if !record.used {
                continue;
            }
            let info = stats.entry(record.ring).or_default();
            info.use_def_chains += 1;
            info.longest_chain = info.longest_chain.max(record.touches);
        }
        stats
    }

    /// クロスリングの DataFlow Edge 群 ((source, target) 昇順で決定論的)。
    pub(crate) fn dataflow_edges(&self) -> Vec<Edge> {
        self.flows
            .iter()
            .map(|((source, target), variables)| Edge {
                source: *source,
                target: *target,
                kind: EdgeKind::DataFlow,
                cardinality: usize_to_f64(variables.len()),
                layers: EdgeLayerData {
                    data_flow: Some(EdgeDataFlowInfo {
                        variables: variables.iter().cloned().collect(),
                    }),
                    ..EdgeLayerData::default()
                },
            })
            .collect()
    }

    fn push_record(&mut self, ring: SigilId) -> usize {
        self.records.push(DefRecord {
            ring,
            touches: 1,
            used: false,
        });
        self.records.len() - 1
    }
}

/// 変数カウントは 2^53 未満なので精度劣化は起きない (layout 側と同じ判断)。
#[allow(clippy::cast_precision_loss)]
fn usize_to_f64(value: usize) -> f64 {
    value as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_stmt(src: &str) -> Stmt {
        syn::parse_str::<Stmt>(src).expect("parse stmt")
    }

    #[test]
    fn let_binding_defines_and_uses() {
        let flow = stmt_dataflow(&parse_stmt("let total = base + bonus;"));
        assert_eq!(flow.defs, vec!["total"]);
        assert!(flow.uses.contains(&"base".to_string()));
        assert!(flow.uses.contains(&"bonus".to_string()));
    }

    #[test]
    fn plain_assign_is_reassign_without_use_of_lhs() {
        let flow = stmt_dataflow(&parse_stmt("total = base;"));
        assert_eq!(flow.reassigns, vec!["total"]);
        assert!(!flow.uses.contains(&"total".to_string()));
    }

    #[test]
    fn compound_assign_reads_and_redefines_lhs() {
        let flow = stmt_dataflow(&parse_stmt("total += step;"));
        assert_eq!(flow.reassigns, vec!["total"]);
        assert!(flow.uses.contains(&"total".to_string()));
        assert!(flow.uses.contains(&"step".to_string()));
    }

    #[test]
    fn field_assign_only_uses_base() {
        let flow = stmt_dataflow(&parse_stmt("state.count = next;"));
        assert!(flow.reassigns.is_empty());
        assert!(flow.uses.contains(&"state".to_string()));
        assert!(flow.uses.contains(&"next".to_string()));
    }

    #[test]
    fn closure_bodies_are_not_tracked() {
        let flow = stmt_dataflow(&parse_stmt("let f = |x| x + captured;"));
        assert_eq!(flow.defs, vec!["f"]);
        assert!(flow.uses.is_empty(), "クロージャ内の use は追わない");
    }

    #[test]
    fn pattern_idents_skip_unit_variants() {
        let pat: Pat = syn::parse_str::<Stmt>("let Some(value) = opt else { return; };")
            .ok()
            .and_then(|stmt| match stmt {
                Stmt::Local(local) => Some(local.pat),
                _ => None,
            })
            .expect("let-else をパースできる");
        assert_eq!(pattern_idents(&pat), vec!["value"]);
    }

    #[test]
    fn scope_tracker_builds_cross_ring_flow() {
        let main = SigilId(0);
        let child = SigilId(1);
        let mut tracker = ScopeTracker::default();
        tracker.push_frame();
        tracker.define(main, "total");
        tracker.push_frame();
        assert!(tracker.use_var(child, "total"));
        tracker.pop_frame();
        tracker.pop_frame();
        let edges = tracker.dataflow_edges();
        assert_eq!(edges.len(), 1);
        assert_eq!((edges[0].source, edges[0].target), (main, child));
        let stats = tracker.ring_stats();
        assert_eq!(stats[&main].use_def_chains, 1);
        assert_eq!(stats[&main].longest_chain, 2); // def 1 + use 1
    }

    #[test]
    fn shadowing_starts_a_new_chain() {
        let main = SigilId(0);
        let mut tracker = ScopeTracker::default();
        tracker.push_frame();
        tracker.define(main, "x");
        assert!(tracker.use_var(main, "x"));
        tracker.define(main, "x"); // let x = x + 1 の左辺
        assert!(tracker.use_var(main, "x"));
        tracker.pop_frame();
        let stats = tracker.ring_stats();
        assert_eq!(stats[&main].use_def_chains, 2);
        assert_eq!(stats[&main].longest_chain, 2);
    }

    #[test]
    fn unresolved_names_do_not_resolve() {
        let mut tracker = ScopeTracker::default();
        tracker.push_frame();
        assert!(!tracker.use_var(SigilId(0), "helper"), "関数名は落ちる");
        tracker.pop_frame();
        assert!(tracker.ring_stats().is_empty());
    }
}
