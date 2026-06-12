//! 関数本体を MainRing + AuxRing + SummonGlyph 群へ再帰展開する
//! (Phase 1.3〜1.4, spec §6.1.2)。
//!
//! 制御構造 (`if` / `match` / `for` / `while` / `loop`) は親リングの `content` に
//! 1個の Operation (Branch/Match/Loop) として現れ、その本体は独立した AuxRing として
//! 切り出される。親子関係は `EdgeKind::ControlFlow` の Edge で表現する (spec §4.2)。
//! 入れ子は深さ制限なく再帰展開する (Phase 1.6 の視覚検証で破綻したら再検討)。
//!
//! call site は所属リング (statement を直接持つリング) から SummonGlyph として
//! ぶら下がる。制御構造のガード式中の call は親リング側、本体中の call は AuxRing 側
//! に係留される (二重計上なし)。同一関数の複数回呼び出しは呼び出しごとに別 glyph
//! とする (merge は Phase 1.5 のレイアウトに余地を残す)。

use magia_core::ir::{
    AuxRingKind, AuxRingRole, Cardinality, ControlFlowInfo, Edge, EdgeKind, EdgeLayerData,
    EffectSet, LayerData, LoopKind, Operation, OperationKind, OperationPayload, Sigil, SigilId,
    SigilKind, SourceSpan,
};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Block, Expr, ExprIf, ExprMatch, ItemFn, Stmt};

use crate::allocator::SigilIdAllocator;
use crate::dataflow::{ScopeTracker, expr_use_candidates, pattern_idents, stmt_dataflow};
use crate::statement::{ParseContext, scan_expression, statement_to_operation};
use crate::summon::{CallSite, UseMap, collect_calls_in_expr, collect_calls_in_stmt};

/// `build_rings` の結果: 1関数分のリング・記号群と接続。
pub(crate) struct RingForest {
    /// MainRing を先頭に、`SigilId` 昇順 (= ソース出現順の深さ優先) で並ぶ。
    pub(crate) sigils: Vec<Sigil>,
    /// 親リング → AuxRing / SummonGlyph の ControlFlow Edge。`target` の `SigilId` 昇順。
    pub(crate) edges: Vec<Edge>,
}

/// 関数本体をリング群へ展開する。MainRing は必ず `sigils[0]` に置かれる。
pub(crate) fn build_rings(
    item_fn: &ItemFn,
    allocator: &mut SigilIdAllocator,
    ctx: ParseContext,
    uses: &UseMap,
) -> RingForest {
    let mut builder = RingBuilder {
        allocator,
        ctx,
        uses,
        dataflow: ScopeTracker::default(),
        sigils: Vec::new(),
        edges: Vec::new(),
    };
    // 関数引数は MainRing で生まれる値 (ベルカ式「生成」極の種、Phase 3.4)。
    let param_seeds: Vec<String> = item_fn
        .sig
        .inputs
        .iter()
        .flat_map(|arg| match arg {
            syn::FnArg::Receiver(_) => vec!["self".to_string()],
            syn::FnArg::Typed(typed) => pattern_idents(&typed.pat),
        })
        .collect();
    builder.build_ring(
        SigilKind::MainRing,
        &item_fn.block.stmts,
        None,
        item_fn.span(),
        &param_seeds,
    );
    // 再帰中は「子を先に push → 親を後で push」の順になるため、ID 順 (= ソース出現順の
    // 深さ優先) に並べ直して決定論的な出力にする。
    let mut sigils = builder.sigils;
    sigils.sort_by_key(|sigil| sigil.id);

    // データフロー集計をリングへ充填する (Phase 3.4)。glyph は対象外。
    let stats = builder.dataflow.ring_stats();
    for sigil in &mut sigils {
        if matches!(sigil.kind, SigilKind::MainRing | SigilKind::AuxRing) {
            sigil.layers.data_flow = Some(stats.get(&sigil.id).cloned().unwrap_or_default());
        }
    }

    let mut edges = builder.edges;
    edges.extend(builder.dataflow.dataflow_edges());
    // ControlFlow (木構造、target 一意) → DataFlow の順。kind 内は (target, source) 昇順。
    edges.sort_by_key(|edge| (edge_kind_rank(edge.kind), edge.target, edge.source));
    RingForest { sigils, edges }
}

/// Edge ソート用の kind 序列 (ControlFlow が先 — 既存出力の不変を保つ)。
fn edge_kind_rank(kind: EdgeKind) -> u8 {
    match kind {
        EdgeKind::ControlFlow => 0,
        EdgeKind::Chain => 1, // 係留線の一種 (glyph の親) — DataFlow より構造に近い
        EdgeKind::DataFlow => 2,
        EdgeKind::Dependency | EdgeKind::Inheritance | EdgeKind::Implementation => 3,
    }
}

struct RingBuilder<'a> {
    allocator: &'a mut SigilIdAllocator,
    ctx: ParseContext,
    uses: &'a UseMap,
    /// 変数スコープとデータフローの追跡 (Phase 3.4)。リング再帰と並走する。
    dataflow: ScopeTracker,
    sigils: Vec<Sigil>,
    edges: Vec<Edge>,
}

/// statement のトップレベルに現れた制御構造の分類。
enum ControlStmt<'ast> {
    If(&'ast ExprIf),
    Match(&'ast ExprMatch),
    For(&'ast syn::ExprForLoop),
    While(&'ast syn::ExprWhile),
    Loop(&'ast syn::ExprLoop),
}

/// statement が制御構造そのものなら分類して返す。
///
/// `let x = if ... { .. }` のような式の内側の制御構造は Phase 1.3 では切り出さず、
/// statement 全体を1個の Compute として扱う (本体の効果は visitor が拾う)。
fn classify(stmt: &Stmt) -> Option<ControlStmt<'_>> {
    let Stmt::Expr(expr, _) = stmt else {
        return None;
    };
    match expr {
        Expr::If(node) => Some(ControlStmt::If(node)),
        Expr::Match(node) => Some(ControlStmt::Match(node)),
        Expr::ForLoop(node) => Some(ControlStmt::For(node)),
        Expr::While(node) => Some(ControlStmt::While(node)),
        Expr::Loop(node) => Some(ControlStmt::Loop(node)),
        _ => None,
    }
}

impl RingBuilder<'_> {
    /// statement 列から1個のリングを構築し、子 (AuxRing) を再帰的に生成する。
    ///
    /// `seeds` はこのリングのスコープ冒頭で生まれる変数 (関数引数・for のパターン・
    /// match アームの束縛・if let の束縛)。リング本体に対応する Operation を
    /// 持たないため、ここでまとめて def する (Phase 3.4)。
    fn build_ring(
        &mut self,
        kind: SigilKind,
        stmts: &[Stmt],
        role: Option<AuxRingRole>,
        span: proc_macro2::Span,
        seeds: &[String],
    ) -> SigilId {
        let id = self.allocator.allocate();
        self.dataflow.push_frame();
        for seed in seeds {
            self.dataflow.define(id, seed);
        }
        let mut content: Vec<Operation> = Vec::new();
        let mut info = ControlFlowInfo {
            role,
            ..ControlFlowInfo::default()
        };

        for stmt in stmts {
            // この制御構造が親リング上で占める Operation 位置 = AuxRing の anchor。
            // u32::MAX のようなセンチネルに黙って落とすと Phase 1.5 のレイアウトが
            // 「存在しない位置」を参照しうるため、超過時は明示的に落とす。
            let anchor = u32::try_from(content.len())
                .expect("1リングの Operation 数が u32 を超えることはない");
            match classify(stmt) {
                Some(ControlStmt::If(node)) => {
                    let head = format!("if {}", node.cond.to_token_stream());
                    let mut op = self.control_operation(
                        OperationKind::Branch,
                        head,
                        Some(&node.cond),
                        source_span_between(node.if_token.span, node.cond.span()),
                    );
                    // 条件式は本リングのスコープで評価される (use は本リングに計上)。
                    op.payload.uses = self.resolve_uses(id, &expr_use_candidates(&node.cond));
                    content.push(op);
                    // else if / else を含む連鎖全体で1分岐と数える (ControlFlowInfo の規約)。
                    info.branch_count += 1;
                    // ガード式中の call は本リング側に係留する (本体は AuxRing 側)。
                    self.spawn_glyphs(id, collect_calls_in_expr(&node.cond, self.uses));
                    self.spawn_if_chain(id, node, anchor);
                }
                Some(ControlStmt::Match(node)) => {
                    let head = format!("match {}", node.expr.to_token_stream());
                    let mut op = self.control_operation(
                        OperationKind::Match,
                        head,
                        Some(&node.expr),
                        source_span_between(node.match_token.span, node.expr.span()),
                    );
                    op.payload.uses = self.resolve_uses(id, &expr_use_candidates(&node.expr));
                    content.push(op);
                    info.branch_count += u32::try_from(node.arms.len().saturating_sub(1))
                        .expect("match のアーム数が u32 を超えることはない");
                    self.spawn_glyphs(id, collect_calls_in_expr(&node.expr, self.uses));
                    self.spawn_match_arms(id, node, anchor);
                }
                Some(ControlStmt::For(node)) => {
                    content.push(self.spawn_for_loop(id, node, anchor));
                    info.loop_count += 1;
                }
                Some(ControlStmt::While(node)) => {
                    content.push(self.spawn_while_loop(id, node, anchor));
                    info.loop_count += 1;
                }
                Some(ControlStmt::Loop(node)) => {
                    let header = source_span(node.loop_token.span);
                    content.push(self.control_operation(
                        OperationKind::Loop,
                        "loop".to_string(),
                        None,
                        header.clone(),
                    ));
                    info.loop_count += 1;
                    let role = aux_role(
                        AuxRingKind::LoopBody(LoopKind::Loop),
                        anchor,
                        0,
                        None,
                        Some(header),
                    );
                    self.spawn_block_child(id, role, &node.body, &[]);
                }
                None => {
                    content.push(self.plain_statement(id, stmt));
                    self.spawn_glyphs(id, collect_calls_in_stmt(stmt, self.uses));
                }
            }
        }

        self.seal_ring(id, kind, content, info, span);
        id
    }

    /// リングの Sigil を確定して push し、スコープフレームを閉じる。
    fn seal_ring(
        &mut self,
        id: SigilId,
        kind: SigilKind,
        content: Vec<Operation>,
        mut info: ControlFlowInfo,
        span: proc_macro2::Span,
    ) {
        // `return` 文と `?` 演算子はどちらも early_return フラグ経由で計上される。
        info.early_return_count =
            u32::try_from(content.iter().filter(|op| op.payload.early_return).count())
                .expect("early_return 数は content 長以下で u32 を超えることはない");

        // Phase 1.3 の重み近似 = Operation 数。Phase 1.5 のレイアウトで直径に反映する。
        let weight = f64::from(
            u32::try_from(content.len()).expect("1リングの Operation 数が u32 を超えることはない"),
        );
        self.sigils.push(Sigil {
            id,
            kind,
            content,
            layers: LayerData {
                control_flow: Some(info),
                ..LayerData::default()
            },
            source_location: source_span(span),
            cardinality: Cardinality {
                weight,
                density: None,
            },
        });
        self.dataflow.pop_frame();
    }

    /// 制御構造でない statement を Operation 化し、def/use を解決する (Phase 3.4)。
    fn plain_statement(&mut self, ring: SigilId, stmt: &Stmt) -> Operation {
        let mut op = statement_to_operation(stmt, self.ctx);
        let flow = stmt_dataflow(stmt);
        // 解決順序が要: `let x = x + 1` (シャドーイング) は旧 x の use を
        // 数えてから新 x を def する。
        op.payload.uses = self.resolve_uses(ring, &flow.uses);
        for name in &flow.reassigns {
            self.dataflow.redefine(ring, name);
        }
        for name in &flow.defs {
            self.dataflow.define(ring, name);
        }
        let mut defs = flow.defs;
        defs.extend(flow.reassigns);
        defs.sort();
        defs.dedup();
        op.payload.defs = defs;
        op
    }

    /// use 候補をスコープで解決し、解決できた変数名 (辞書順・重複なし) を返す。
    /// 出現1回ごとに use として数える (チェーン長は出現回数を反映、payload は重複排除)。
    fn resolve_uses(&mut self, ring: SigilId, candidates: &[String]) -> Vec<String> {
        let mut resolved = std::collections::BTreeSet::new();
        for name in candidates {
            if self.dataflow.use_var(ring, name) {
                resolved.insert(name.clone());
            }
        }
        resolved.into_iter().collect()
    }

    /// `for` ループ: ヘッダ Operation を作り、本体を AuxRing 化する。
    fn spawn_for_loop(&mut self, ring: SigilId, node: &syn::ExprForLoop, anchor: u32) -> Operation {
        let head = format!(
            "for {} in {}",
            node.pat.to_token_stream(),
            node.expr.to_token_stream()
        );
        let header = source_span_between(node.for_token.span, node.expr.span());
        let mut op =
            self.control_operation(OperationKind::Loop, head, Some(&node.expr), header.clone());
        op.payload.uses = self.resolve_uses(ring, &expr_use_candidates(&node.expr));
        // ループ変数の誕生は構文上ここ (説明可能性)。スコープは本体リング側。
        let seeds = pattern_idents(&node.pat);
        op.payload.defs.clone_from(&seeds);
        self.spawn_glyphs(ring, collect_calls_in_expr(&node.expr, self.uses));
        let role = aux_role(
            AuxRingKind::LoopBody(LoopKind::For),
            anchor,
            0,
            None,
            Some(header),
        );
        self.spawn_block_child(ring, role, &node.body, &seeds);
        op
    }

    /// `while` / `while let` ループ: ヘッダ Operation を作り、本体を AuxRing 化する。
    fn spawn_while_loop(&mut self, ring: SigilId, node: &syn::ExprWhile, anchor: u32) -> Operation {
        let head = format!("while {}", node.cond.to_token_stream());
        let header = source_span_between(node.while_token.span, node.cond.span());
        let mut op =
            self.control_operation(OperationKind::Loop, head, Some(&node.cond), header.clone());
        op.payload.uses = self.resolve_uses(ring, &expr_use_candidates(&node.cond));
        self.spawn_glyphs(ring, collect_calls_in_expr(&node.cond, self.uses));
        let role = aux_role(
            AuxRingKind::LoopBody(LoopKind::While),
            anchor,
            0,
            None,
            Some(header),
        );
        // while let の束縛は本体スコープで生まれる。
        self.spawn_block_child(ring, role, &node.body, &if_let_seeds(&node.cond));
        op
    }

    /// `if` / `else if` / `else` の連鎖を左から順に AuxRing 化する。
    fn spawn_if_chain(&mut self, parent: SigilId, expr_if: &ExprIf, anchor: u32) {
        let mut ordinal = 0u32;
        let mut current = expr_if;
        loop {
            // 腕のガード (`if cond`) — 補助リングのホバープレビューで条件を見せる。
            let guard = Some(source_span_between(
                current.if_token.span,
                current.cond.span(),
            ));
            let role = aux_role(AuxRingKind::IfBranch, anchor, ordinal, None, guard);
            // if let の束縛は then 節のスコープで生まれる。
            self.spawn_block_child(
                parent,
                role,
                &current.then_branch,
                &if_let_seeds(&current.cond),
            );
            ordinal += 1;
            let Some((_, else_expr)) = &current.else_branch else {
                break;
            };
            match else_expr.as_ref() {
                Expr::If(next) => {
                    // else if の条件は親スコープで評価される。対応する Operation は
                    // 連鎖先頭の1個だけなので payload には載らないが、フローには数える。
                    self.resolve_uses(parent, &expr_use_candidates(&next.cond));
                    current = next;
                }
                Expr::Block(block) => {
                    // else は無条件の腕 — ガードなし。
                    let role = aux_role(AuxRingKind::ElseBranch, anchor, ordinal, None, None);
                    self.spawn_block_child(parent, role, &block.block, &[]);
                    break;
                }
                // 文法上 else の直後は block か if のみだが、防御的に単一式リングで受ける。
                other => {
                    let role = aux_role(AuxRingKind::ElseBranch, anchor, ordinal, None, None);
                    self.spawn_expr_child(parent, role, other);
                    break;
                }
            }
        }
    }

    /// `match` のアームをそれぞれ AuxRing 化する。パターン文字列をラベルとして残す。
    fn spawn_match_arms(&mut self, parent: SigilId, expr_match: &ExprMatch, anchor: u32) {
        for (index, arm) in expr_match.arms.iter().enumerate() {
            let ordinal = u32::try_from(index).expect("match のアーム数が u32 を超えることはない");
            // アームガード (`pat if cond =>`) は親スコープで評価されるため、
            // ガード式中の call は被検査式と同様に親リング側へ係留する。
            if let Some((_, guard)) = &arm.guard {
                self.spawn_glyphs(parent, collect_calls_in_expr(guard, self.uses));
                self.resolve_uses(parent, &expr_use_candidates(guard));
            }
            let label = Some(arm.pat.to_token_stream().to_string());
            // 腕のヘッダ = パターン (+ アームガードがあれば `pat if guard` まで)。
            let guard_end = arm
                .guard
                .as_ref()
                .map_or_else(|| arm.pat.span(), |(_, guard)| guard.span());
            let guard = Some(source_span_between(arm.pat.span(), guard_end));
            let role = aux_role(AuxRingKind::MatchArm, anchor, ordinal, label, guard);
            // アームパターンの束縛 (`Some(x) =>` の x) はアーム本体のスコープで生まれる。
            let seeds = pattern_idents(&arm.pat);
            match arm.body.as_ref() {
                Expr::Block(block) => self.spawn_block_child(parent, role, &block.block, &seeds),
                expr => self.spawn_expr_child_with_seeds(parent, role, expr, &seeds),
            }
        }
    }

    fn spawn_block_child(
        &mut self,
        parent: SigilId,
        role: AuxRingRole,
        block: &Block,
        seeds: &[String],
    ) {
        self.spawn_child(parent, role, &block.stmts, block.span(), seeds);
    }

    /// 非ブロックのアーム体 (`1 => a`) も statement 化して再帰経路を一本化する。
    /// `_ => match ...` のような入れ子の制御構造もこの経路で AuxRing 展開される。
    fn spawn_expr_child(&mut self, parent: SigilId, role: AuxRingRole, expr: &Expr) {
        self.spawn_expr_child_with_seeds(parent, role, expr, &[]);
    }

    fn spawn_expr_child_with_seeds(
        &mut self,
        parent: SigilId,
        role: AuxRingRole,
        expr: &Expr,
        seeds: &[String],
    ) {
        let span = expr.span();
        let stmts = vec![Stmt::Expr(expr.clone(), None)];
        self.spawn_child(parent, role, &stmts, span, seeds);
    }

    /// AuxRing を構築し、親リングとの ControlFlow Edge を張る。
    fn spawn_child(
        &mut self,
        parent: SigilId,
        role: AuxRingRole,
        stmts: &[Stmt],
        span: proc_macro2::Span,
        seeds: &[String],
    ) {
        let child = self.build_ring(SigilKind::AuxRing, stmts, Some(role), span, seeds);
        // 不変条件: build_ring は自身の Sigil を最後に push するため、末尾が必ず子リング。
        debug_assert_eq!(self.sigils.last().map(|sigil| sigil.id), Some(child));
        let weight = self
            .sigils
            .last()
            .map_or(0.0, |sigil| sigil.cardinality.weight);
        self.edges.push(Edge {
            source: parent,
            target: child,
            kind: EdgeKind::ControlFlow,
            // 空ブロックでも接続線は描くため最低 1.0 (Phase 1.5 で太さに反映)。
            cardinality: weight.max(1.0),
            layers: EdgeLayerData::default(),
        });
    }

    /// call site 群を SummonGlyph として生成し、所属リングとの ControlFlow Edge を張る。
    ///
    /// glyph は「呼び出し1件 = Call Operation 1個」を content に持つ。呼び出し先パスと
    /// 効果カテゴリは Operation 側 (`call_target` / `EffectSet`) に載せ、Sigil の layers
    /// はリング専用の `control_flow` を持たない。
    fn spawn_glyphs(&mut self, parent: SigilId, calls: Vec<CallSite>) {
        // チェーン番号 → そのチェーンで直近に生成した glyph (Phase 4.8)。
        // collect が実行順 (index 昇順) で返す規約により、後続の参照先は必ず登録済み。
        // BTreeMap はプロジェクト規約 (決定論)。ここは lookup のみだが揃えておく。
        let mut chain_tail: std::collections::BTreeMap<u32, SigilId> =
            std::collections::BTreeMap::new();
        for call in calls {
            let mut effects = call.effects;
            // unsafe fn のコンテキストは召喚記号にも伝播する (色相規約 spec §6.1.3 の赤)。
            // `unsafe { ... }` ブロック単位の文脈追跡は Phase 1 ではしない。
            if self.ctx.fn_is_unsafe {
                effects.unsafe_block = true;
            }
            let glyph = self.allocator.allocate();
            self.sigils.push(Sigil {
                id: glyph,
                kind: SigilKind::SummonGlyph,
                content: vec![Operation {
                    kind: OperationKind::Call,
                    effects,
                    payload: OperationPayload {
                        source_excerpt: Some(call.excerpt),
                        source_span: Some(source_span(call.span)),
                        call_target: Some(call.target),
                        early_return: false,
                        ..OperationPayload::default()
                    },
                }],
                layers: LayerData::default(),
                source_location: source_span(call.span),
                cardinality: Cardinality {
                    weight: 1.0,
                    density: None,
                },
            });
            // 鎖の後続は先行 glyph から Chain edge、先頭・単独はリングから ControlFlow。
            let (source, kind) = match call.chain {
                Some((chain_id, index)) if index > 0 => (
                    *chain_tail
                        .get(&chain_id)
                        .expect("チェーンは実行順で生成される (index>0 の前に先行が登録済み)"),
                    EdgeKind::Chain,
                ),
                _ => (parent, EdgeKind::ControlFlow),
            };
            if let Some((chain_id, _)) = call.chain {
                chain_tail.insert(chain_id, glyph);
            }
            self.edges.push(Edge {
                source,
                target: glyph,
                kind,
                cardinality: 1.0,
                layers: EdgeLayerData::default(),
            });
        }
    }

    /// 制御構造そのものを表す Operation (親リングの `content` に置く)。
    ///
    /// `guard` は条件式・被検査式・イテレータ式のみ。本体ブロックは AuxRing 側で
    /// 処理されるため渡さない (二重計上の防止)。`if f()? { .. }` のように条件に
    /// `?` がある場合は kind を Branch のまま `early_return` フラグで伝える。
    /// `location` は head と同じ範囲 (キーワード〜ガード式) の原文位置。
    fn control_operation(
        &self,
        kind: OperationKind,
        head: String,
        guard: Option<&Expr>,
        location: SourceSpan,
    ) -> Operation {
        let scan = guard.map(scan_expression);
        let early_return = scan.as_ref().is_some_and(|s| s.has_try);
        let has_unsafe = scan.as_ref().is_some_and(|s| s.has_unsafe);
        let mut effects = EffectSet::default();
        if self.ctx.fn_is_unsafe || has_unsafe {
            effects.unsafe_block = true;
        }
        Operation {
            kind,
            effects,
            payload: OperationPayload {
                source_excerpt: Some(head),
                source_span: Some(location),
                call_target: None,
                early_return,
                ..OperationPayload::default()
            },
        }
    }
}

/// `if let PAT = expr` / `while let PAT = expr` の束縛変数 (本体スコープで生まれる)。
/// let chains (`a && let Some(x) = b`) は Phase 3 では追わない (近似)。
fn if_let_seeds(cond: &Expr) -> Vec<String> {
    match cond {
        Expr::Let(expr_let) => pattern_idents(&expr_let.pat),
        _ => Vec::new(),
    }
}

fn aux_role(
    kind: AuxRingKind,
    anchor_operation: u32,
    ordinal: u32,
    label: Option<String>,
    guard_location: Option<SourceSpan>,
) -> AuxRingRole {
    AuxRingRole {
        kind,
        anchor_operation,
        ordinal,
        label,
        guard_location,
    }
}

pub(crate) fn source_span(span: proc_macro2::Span) -> SourceSpan {
    source_span_between(span, span)
}

/// 2つの span をまたぐ範囲 (`start` の先頭〜`end` の末尾) を `SourceSpan` 化する。
/// `Span::join` が stable に無いための行・列ベースの合成 — 制御 Operation の
/// 「キーワード〜ガード式」(`if cond` / `for pat in expr`) の切り出しに使う。
pub(crate) fn source_span_between(start: proc_macro2::Span, end: proc_macro2::Span) -> SourceSpan {
    let start_line = u32::try_from(start.start().line).unwrap_or(0);
    let end_line = u32::try_from(end.end().line).unwrap_or(start_line);
    SourceSpan {
        file: String::new(),
        start_line,
        end_line,
        // proc_macro2 の LineColumn は 0-based (文字単位)。start().column は
        // 文字位置 (inclusive)、end().column は**既に最後の文字の直後 (exclusive)**。
        // SourceSpan の規約は 1-based・end exclusive なので、どちらも +1 だけで
        // 変換が成立する (end にさらに +1 してはならない)。
        start_column: u32::try_from(start.start().column + 1).ok(),
        end_column: u32::try_from(end.end().column + 1).ok(),
    }
}
