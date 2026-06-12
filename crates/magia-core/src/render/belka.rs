//! **deprecated (Phase 4.0.9)**: Phase 4.3 (Vue SSR 一本化) でリメイクし、本モジュールは削除する。
//! serve の動的 UI でのベルカ表示も 4.3 で Vue コンポーネントへ移植する。
//!
//! **保守方針 (オーナー判定 2026-06-11)**: リメイクが終わるまで本モジュールの保守価値は低め。
//! バグ修正・意匠調整の要望が来ても、軽微なら 4.3 のリメイクに織り込む方を優先し、
//! ここへの投資 (リファクタ・テスト追加・最適化) は避ける。
//!
//! ベルカ式レンダラ — データフロー三角力場 (Phase 3.5, spec v0.3 §14)。
//!
//! ミッドチルダ式 (ノード中心の同心円 = Call Graph) と**同じ IR の別射影**:
//! 関数内の値の流れを「生成 / 変換 / 消費」の3極に集約し、三角形の頂点に置く
//! (spec §14.2)。頂点の重み (Operation 数) の偏りが三角形の歪みとして現れ、
//! 各頂点の放射状グラデーション力場の濃淡がデータフロー量を表す。
//!
//! 規約はミッドチルダ式と共通: 乱数なし・固定桁数値・属性順固定 (spec §6.1.4)。
//! 座標は最初から画面系 (y 下向き) で組む (レイアウトエンジンを経由しない —
//! 三角配置は閉形式で決まるため)。
//!
//! 操作ドットの色は効果カテゴリの色相規約 (spec §6.1.3) を式をまたいで共有する。
//! Phase 3.5 では `FilterSpec` (レイヤー語彙はミッドチルダ式の3層が前提) を
//! 適用しない — CLI 側が `--style belka` + フィルター併用を明示エラーにする。

use std::collections::BTreeMap;
use std::fmt::Write;

use kurbo::Point;

use crate::filter::EffectCategory;
use crate::ir::{MagiaGraph, Module, Operation, OperationKind, SigilKind};
use crate::render::midchilda::{escape_xml, num};
use crate::render::palette;

// 寸法は仮置き (Phase 1.5 と同じ「動かしてから目視で調整」方針)。
/// 三角形の基準半径 (中心から頂点まで)。
const TRI_BASE: f64 = 150.0;
/// 重みの偏りによる頂点距離の振れ幅 (0.7×〜1.3× を作る)。
const TRI_SKEW: f64 = 0.3;
/// 極円の最小半径。
const POLE_MIN_RADIUS: f64 = 26.0;
/// 極円半径の Operation 数による成長係数 (sqrt スケール)。
const POLE_GROWTH: f64 = 7.0;
/// 力場の極円外への伸び (フロー量 1 あたり)。
const FIELD_REACH_PER_FLOW: f64 = 9.0;
/// 力場の基礎の伸び。
const FIELD_BASE_REACH: f64 = 24.0;
/// 力場グラデーション中心の不透明度。
const FIELD_OPACITY: f64 = 0.28;
/// 操作ドット半径 (ミッドチルダ式と同じ見え方)。
const DOT_RADIUS: f64 = 3.5;
/// phyllotaxis (ひまわり配置) の黄金角 [rad]。無理数だが定数なので決定論的。
const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;
/// フロー線の太さ: 1変数あたりの増分と上限。
const FLOW_WIDTH_STEP: f64 = 0.9;
const FLOW_WIDTH_MAX: f64 = 7.0;
/// キャンバス余白。
const MARGIN: f64 = 30.0;

/// ベルカ式で描画する。モジュールが複数あっても最初の1つ (Phase 1〜3 は単一関数)。
pub(crate) fn render(graph: &MagiaGraph) -> String {
    let mut out = String::new();
    write_document(&mut out, graph).expect("String への SVG 書き込みは失敗しない");
    out
}

// ===== 配置済み IR (Phase 4.3 M3 — Vue 移植の境界) =====
//
// 射影 (project / pole_of) と三角配置 (place_poles) は意味論 + レイアウトなので
// 本モジュール削除 (M5) 後も残す。SVG 文字列化 (write_document 系) だけが削除対象。
// 色・ラベル文言・矢じり形状は「描き方」— Vue 側 (BelkaCircle) が持つ。

/// ベルカ式の配置済み IR。`pole` 語彙 (genesis/transmute/consume) を境界に、
/// Vue が色・ラベル・グラデーションを引く (ミッドチルダ IR の effect 語彙と同型)。
#[derive(serde::Serialize)]
pub struct BelkaIr {
    pub view_box: [f64; 4],
    pub poles: Vec<BelkaPoleIr>,
    pub flows: Vec<BelkaFlowIr>,
    /// シグネチャ (上端の平書き — 円弧ラベルはミッドチルダ式の意匠)。
    pub signature: Option<BelkaSignatureIr>,
}

#[derive(serde::Serialize)]
pub struct BelkaPoleIr {
    /// 極の語彙 (genesis / transmute / consume)。
    pub pole: &'static str,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub field_radius: f64,
    /// 極名ラベルの位置 (円の外側、ベースライン補正済み)。
    pub label_x: f64,
    pub label_y: f64,
    /// 操作ドット (phyllotaxis 配置済み)。
    pub dots: Vec<BelkaDotIr>,
}

#[derive(serde::Serialize)]
pub struct BelkaDotIr {
    pub x: f64,
    pub y: f64,
    pub effect: EffectCategory,
}

#[derive(serde::Serialize)]
pub struct BelkaFlowIr {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub width: f64,
    /// 矢じりの頂点 (極円の縁)。羽の形は Vue が tip → 線端の方向から計算する。
    pub tip_x: f64,
    pub tip_y: f64,
}

#[derive(serde::Serialize)]
pub struct BelkaSignatureIr {
    pub text: String,
    pub x: f64,
    pub y: f64,
}

impl Pole {
    /// IR 境界の語彙 (Vue 側のテーブルキー)。
    fn key(self) -> &'static str {
        match self {
            Pole::Genesis => "genesis",
            Pole::Transmute => "transmute",
            Pole::Consume => "consume",
        }
    }
}

/// 配置済みベルカ IR を構築する (`write_document` と同じ射影・配置計算)。
#[must_use]
pub fn belka_ir(graph: &MagiaGraph) -> BelkaIr {
    use crate::render::ir_export::nz;
    let Some(module) = graph.modules.first() else {
        return BelkaIr {
            view_box: [0.0, 0.0, 10.0, 10.0],
            poles: Vec::new(),
            flows: Vec::new(),
            signature: None,
        };
    };
    let model = project(module);
    let placed = place_poles(&model);
    let reach = placed
        .iter()
        .map(|p| p.center.to_vec2().hypot() + p.field_radius)
        .fold(0.0_f64, f64::max)
        + MARGIN;

    let poles = placed
        .iter()
        .map(|p| {
            let direction = p.center.to_vec2();
            let label_direction = if direction.hypot() < 1e-6 {
                kurbo::Vec2::new(0.0, -1.0)
            } else {
                direction / direction.hypot()
            };
            let label_at = p.center + label_direction * (p.radius + 14.0);
            let dots = model.dots.get(&p.pole).map_or(&[][..], Vec::as_slice);
            let count = dots.len();
            let dots = dots
                .iter()
                .enumerate()
                .map(|(index, category)| {
                    let fraction = (usize_to_f64(index) + 0.5) / usize_to_f64(count.max(1));
                    let r = (p.radius - DOT_RADIUS - 4.0).max(0.0) * fraction.sqrt();
                    let theta = usize_to_f64(index) * GOLDEN_ANGLE;
                    BelkaDotIr {
                        x: nz(p.center.x + r * theta.cos()),
                        y: nz(p.center.y + r * theta.sin()),
                        effect: *category,
                    }
                })
                .collect();
            BelkaPoleIr {
                pole: p.pole.key(),
                x: nz(p.center.x),
                y: nz(p.center.y),
                radius: nz(p.radius),
                field_radius: nz(p.field_radius),
                label_x: nz(label_at.x),
                label_y: nz(label_at.y + 4.0), // ベースライン補正 (write_pole と同値)
                dots,
            }
        })
        .collect();

    let flows = model
        .flows
        .iter()
        .filter(|((source, target), _)| source != target)
        .filter_map(|(&(source, target), &count)| {
            let from = placed.iter().find(|p| p.pole == source)?;
            let to = placed.iter().find(|p| p.pole == target)?;
            let delta = to.center - from.center;
            let length = delta.hypot();
            if length < 1e-6 {
                return None;
            }
            let unit = delta / length;
            let start = from.center + unit * from.radius;
            let end = to.center - unit * (to.radius + 6.0);
            let tip = to.center - unit * to.radius;
            Some(BelkaFlowIr {
                x1: nz(start.x),
                y1: nz(start.y),
                x2: nz(end.x),
                y2: nz(end.y),
                width: nz((f64::from(count) * FLOW_WIDTH_STEP + 1.0).min(FLOW_WIDTH_MAX)),
                tip_x: nz(tip.x),
                tip_y: nz(tip.y),
            })
        })
        .collect();

    let signature = module
        .sigils
        .iter()
        .find(|s| s.kind == SigilKind::MainRing)
        .and_then(|s| s.layers.type_info.as_ref())
        .and_then(|t| t.signature.as_deref())
        .map(|text| BelkaSignatureIr {
            text: text.to_string(),
            x: 0.0,
            y: nz(-reach + 16.0),
        });

    BelkaIr {
        view_box: [nz(-reach), nz(-reach), nz(reach * 2.0), nz(reach * 2.0)],
        poles,
        flows,
        signature,
    }
}

// ===== 極モデル (IR → 三極への射影) =====

/// 三極 (spec §14.2)。`Ord` で BTreeMap キー化と決定論的列挙を担保する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Pole {
    /// 生成: 値の誕生 (uses を持たない def)。
    Genesis,
    /// 変換: 中間の use-def (読みながら書く・純粋計算)。
    Transmute,
    /// 消費: 戻り値・早期リターン・副作用 (外界への放出)。
    Consume,
}

const POLES: [Pole; 3] = [Pole::Genesis, Pole::Transmute, Pole::Consume];

impl Pole {
    fn label(self) -> &'static str {
        match self {
            Pole::Genesis => "生成",
            Pole::Transmute => "変換",
            Pole::Consume => "消費",
        }
    }

    fn color(self) -> &'static str {
        match self {
            Pole::Genesis => palette::BELKA_GENESIS,
            Pole::Transmute => palette::BELKA_TRANSMUTE,
            Pole::Consume => palette::BELKA_CONSUME,
        }
    }

    /// 頂点の方位 [rad]、画面座標系 (y 下向き)。生成 = 真上、
    /// 変換 = 左下、消費 = 右下 (生成 → 変換 → 消費 が時計回りに流れる)。
    fn angle(self) -> f64 {
        match self {
            Pole::Genesis => -std::f64::consts::FRAC_PI_2,
            Pole::Transmute => std::f64::consts::PI * (5.0 / 6.0),
            Pole::Consume => std::f64::consts::PI / 6.0,
        }
    }

    fn gradient_id(self) -> &'static str {
        match self {
            Pole::Genesis => "belka-field-genesis",
            Pole::Transmute => "belka-field-transmute",
            Pole::Consume => "belka-field-consume",
        }
    }
}

/// Operation を三極のどれかに分類する (決定論的ヒューリスティック)。
///
/// 優先順位: 帰還・副作用 (消費) > 純粋な誕生 (生成) > それ以外 (変換)。
/// `is_tail` は MainRing 末尾の Operation (Rust の暗黙の戻り値) — 関数の帰結として
/// 消費に倒す。dataflow 情報を持たない純粋計算は中立の変換に倒す。
fn pole_of(op: &Operation, is_tail: bool) -> Pole {
    if is_tail || op.kind == OperationKind::Return || op.payload.early_return {
        return Pole::Consume;
    }
    if EffectCategory::of(&op.effects) != EffectCategory::Pure {
        return Pole::Consume;
    }
    if !op.payload.defs.is_empty() && op.payload.uses.is_empty() {
        return Pole::Genesis;
    }
    Pole::Transmute
}

/// 1関数分の三極射影。
struct BelkaModel {
    /// 極ごとの操作ドット (効果カテゴリ色)。ソース出現順。
    dots: BTreeMap<Pole, Vec<EffectCategory>>,
    /// 極間のフロー量 (use 1回 = 1)。自己フロー (同一極) も持ち、力場の強度に使う。
    flows: BTreeMap<(Pole, Pole), u32>,
}

/// IR を三極へ射影する。
///
/// 走査は **anchor に沿った実行順** (リングの Operation を順に辿り、制御構造の
/// Operation の直後にその AuxRing 本体へ降りる)。深さ優先 (SigilId 順) だと
/// 「ループ本体での再定義」を末尾の戻り値より後に見てしまい、変換 → 消費の
/// フローを取り逃がす。ループ本体は1回だけ辿る静的近似。
/// Operation payload の defs/uses (Phase 3.4) を使い、「変数の最後の def がいる極
/// → use した極」をフローとして数える。決定論的 (子は ordinal / SigilId 順)。
fn project(module: &Module) -> BelkaModel {
    let mut model = BelkaModel {
        dots: POLES.iter().map(|p| (*p, Vec::new())).collect(),
        flows: BTreeMap::new(),
    };
    // ControlFlow Edge から anchor ごとの子リングを引く (DataFlow Edge は使わない —
    // フローは op 単位の defs/uses からこの場で導く)。
    let sigil_by_id: BTreeMap<_, _> = module.sigils.iter().map(|s| (s.id, s)).collect();
    let mut rings_at: BTreeMap<(crate::ir::SigilId, u32), Vec<&crate::ir::Sigil>> = BTreeMap::new();
    for edge in &module.edges {
        if edge.kind != crate::ir::EdgeKind::ControlFlow {
            continue;
        }
        let Some(child) = sigil_by_id.get(&edge.target).copied() else {
            continue;
        };
        if child.kind != SigilKind::AuxRing {
            continue;
        }
        let anchor = child
            .layers
            .control_flow
            .as_ref()
            .and_then(|c| c.role.as_ref())
            .map_or(0, |r| r.anchor_operation);
        rings_at
            .entry((edge.source, anchor))
            .or_default()
            .push(child);
    }
    // 子の順序は (ordinal, SigilId) — Edge の収集順に依存させない。
    for children in rings_at.values_mut() {
        children.sort_by_key(|s| {
            let ordinal = s
                .layers
                .control_flow
                .as_ref()
                .and_then(|c| c.role.as_ref())
                .map_or(0, |r| r.ordinal);
            (ordinal, s.id)
        });
    }

    let mut last_def: BTreeMap<&str, Pole> = BTreeMap::new();
    if let Some(main) = module.sigils.iter().find(|s| s.kind == SigilKind::MainRing) {
        visit_ring(main, true, &rings_at, &mut last_def, &mut model);
    }
    // glyph (call) は呼び出し1件 = Call Operation 1個。リングと同じ規則で分類する
    // (uses/defs は所属 statement の Operation 側にあるため、フローへの影響はない)。
    for sigil in &module.sigils {
        if matches!(sigil.kind, SigilKind::SummonGlyph | SigilKind::GateGlyph) {
            for op in &sigil.content {
                let pole = pole_of(op, false);
                model
                    .dots
                    .get_mut(&pole)
                    .expect("dots は全極で初期化済み")
                    .push(EffectCategory::of(&op.effects));
            }
        }
    }
    model
}

/// 実行順の再帰走査: Operation → その anchor にぶら下がる AuxRing 本体 → 次の Operation。
fn visit_ring<'a>(
    sigil: &'a crate::ir::Sigil,
    is_main: bool,
    rings_at: &BTreeMap<(crate::ir::SigilId, u32), Vec<&'a crate::ir::Sigil>>,
    last_def: &mut BTreeMap<&'a str, Pole>,
    model: &mut BelkaModel,
) {
    for (index, op) in sigil.content.iter().enumerate() {
        let is_tail = is_main && index + 1 == sigil.content.len();
        let pole = pole_of(op, is_tail);
        model
            .dots
            .get_mut(&pole)
            .expect("dots は全極で初期化済み")
            .push(EffectCategory::of(&op.effects));
        for name in &op.payload.uses {
            if let Some(&source) = last_def.get(name.as_str()) {
                *model.flows.entry((source, pole)).or_insert(0) += 1;
            }
        }
        for name in &op.payload.defs {
            last_def.insert(name.as_str(), pole);
        }
        // u32::MAX センチネルは禁じ手 (存在しない anchor への誤ヒット)。超過時は明示的に落とす。
        let anchor = u32::try_from(index).expect("1リングの Operation 数が u32 を超えることはない");
        if let Some(children) = rings_at.get(&(sigil.id, anchor)) {
            for child in children {
                visit_ring(child, false, rings_at, last_def, model);
            }
        }
    }
}

impl BelkaModel {
    fn ops(&self, pole: Pole) -> usize {
        self.dots.get(&pole).map_or(0, Vec::len)
    }

    /// 力場の強度 = その極を通るフロー量 (in + out、自己フローは両側で数える)。
    fn field_strength(&self, pole: Pole) -> u32 {
        self.flows
            .iter()
            .map(|(&(source, target), count)| {
                let mut strength = 0;
                if source == pole {
                    strength += count;
                }
                if target == pole {
                    strength += count;
                }
                strength
            })
            .sum()
    }
}

// ===== 三角レイアウト (閉形式・決定論的) =====

struct PlacedPole {
    pole: Pole,
    center: Point,
    radius: f64,
    field_radius: f64,
}

fn place_poles(model: &BelkaModel) -> Vec<PlacedPole> {
    let max_ops = POLES
        .iter()
        .map(|p| model.ops(*p))
        .max()
        .unwrap_or(0)
        .max(1);
    POLES
        .iter()
        .map(|&pole| {
            let ops = model.ops(pole);
            // 重みの偏り = 頂点距離の歪み (重い極ほど中心から遠い、spec §14.2)。
            let weight_ratio = usize_to_f64(ops) / usize_to_f64(max_ops);
            let distance = TRI_BASE * (1.0 - TRI_SKEW + 2.0 * TRI_SKEW * weight_ratio);
            let radius = POLE_MIN_RADIUS + POLE_GROWTH * usize_to_f64(ops).sqrt();
            let field_radius = radius
                + FIELD_BASE_REACH
                + FIELD_REACH_PER_FLOW * f64::from(model.field_strength(pole));
            let angle = pole.angle();
            PlacedPole {
                pole,
                center: Point::new(distance * angle.cos(), distance * angle.sin()),
                radius,
                field_radius,
            }
        })
        .collect()
}

// ===== SVG 出力 =====

fn write_document(out: &mut String, graph: &MagiaGraph) -> std::fmt::Result {
    let Some(module) = graph.modules.first() else {
        return writeln!(
            out,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"></svg>"#
        );
    };
    let model = project(module);
    let placed = place_poles(&model);

    // キャンバス: 力場円の合併 + シグネチャの行間。
    let reach = placed
        .iter()
        .map(|p| p.center.to_vec2().hypot() + p.field_radius)
        .fold(0.0_f64, f64::max)
        + MARGIN;
    writeln!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}">"#,
        num(-reach),
        num(-reach),
        num(reach * 2.0),
        num(reach * 2.0),
    )?;

    // 力場のグラデーション定義 (極ごとに1つ、中心が濃く外周で消える)。
    // gradientUnits は既定 (objectBoundingBox): 参照元が <circle> なので
    // 「バウンディングボックス中心 = 円の中心、50% = 半径」となり意図どおり (確認済み)。
    writeln!(out, "<defs>")?;
    for pole in POLES {
        writeln!(
            out,
            r#"<radialGradient id="{}"><stop offset="0" stop-color="{}" stop-opacity="{}"/><stop offset="1" stop-color="{}" stop-opacity="0"/></radialGradient>"#,
            pole.gradient_id(),
            pole.color(),
            num(FIELD_OPACITY),
            pole.color(),
        )?;
    }
    writeln!(out, "</defs>")?;

    // 1) 力場 (最背面、重なりがアディティブに濃くなる)。
    writeln!(out, r#"<g class="belka-field">"#)?;
    for p in &placed {
        writeln!(
            out,
            r#"<circle class="field-{}" cx="{}" cy="{}" r="{}" fill="url(#{})"/>"#,
            p.pole.gradient_id().trim_start_matches("belka-field-"),
            num(p.center.x),
            num(p.center.y),
            num(p.field_radius),
            p.pole.gradient_id(),
        )?;
    }
    writeln!(out, "</g>")?;

    // 2) フロー線 (極間のデータの流れ。太さ = フロー量)。
    writeln!(out, r#"<g class="belka-flows">"#)?;
    for (&(source, target), &count) in &model.flows {
        if source == target {
            continue; // 自己フローは力場の強度として表現済み
        }
        let from = placed
            .iter()
            .find(|p| p.pole == source)
            .expect("placed は全極を含む");
        let to = placed
            .iter()
            .find(|p| p.pole == target)
            .expect("placed は全極を含む");
        write_flow_line(out, from, to, count)?;
    }
    writeln!(out, "</g>")?;

    // 3) 極 (頂点円 + ラベル + 操作ドット)。
    writeln!(out, r#"<g class="belka-poles">"#)?;
    for p in &placed {
        write_pole(out, p, &model)?;
    }
    writeln!(out, "</g>")?;

    // 4) シグネチャ (上端の平書きテキスト — 円弧ラベルはミッドチルダ式の意匠)。
    if let Some(signature) = module
        .sigils
        .iter()
        .find(|s| s.kind == SigilKind::MainRing)
        .and_then(|s| s.layers.type_info.as_ref())
        .and_then(|t| t.signature.as_deref())
    {
        writeln!(
            out,
            r##"<text class="signature" x="0" y="{}" font-size="11" fill="#000000" text-anchor="middle">{}</text>"##,
            num(-reach + 16.0),
            escape_xml(signature),
        )?;
    }

    writeln!(out, "</svg>")
}

/// 極間のフロー線: 円の縁から縁へ。終端に小さな矢じりを置く。
fn write_flow_line(
    out: &mut String,
    from: &PlacedPole,
    to: &PlacedPole,
    count: u32,
) -> std::fmt::Result {
    let delta = to.center - from.center;
    let length = delta.hypot();
    if length < 1e-6 {
        return Ok(());
    }
    let unit = delta / length;
    let start = from.center + unit * from.radius;
    let end = to.center - unit * (to.radius + 6.0);
    let width = (f64::from(count) * FLOW_WIDTH_STEP + 1.0).min(FLOW_WIDTH_MAX);
    writeln!(
        out,
        r##"<line class="belka-flow" x1="{}" y1="{}" x2="{}" y2="{}" stroke="#555555" stroke-width="{}" stroke-opacity="0.75"/>"##,
        num(start.x),
        num(start.y),
        num(end.x),
        num(end.y),
        num(width),
    )?;
    // 矢じり (流れの向き)。
    let perp = kurbo::Vec2::new(-unit.y, unit.x);
    let tip = to.center - unit * to.radius;
    let base = tip - unit * 7.0;
    let wing_a = base + perp * 4.5;
    let wing_b = base - perp * 4.5;
    writeln!(
        out,
        r##"<polygon class="belka-flow-head" points="{},{} {},{} {},{}" fill="#555555"/>"##,
        num(tip.x),
        num(tip.y),
        num(wing_a.x),
        num(wing_a.y),
        num(wing_b.x),
        num(wing_b.y),
    )
}

/// 極1つ: 頂点円 + 極名 + 操作ドット (phyllotaxis 配置)。
fn write_pole(out: &mut String, p: &PlacedPole, model: &BelkaModel) -> std::fmt::Result {
    writeln!(
        out,
        r##"<circle class="belka-pole" cx="{}" cy="{}" r="{}" fill="#ffffff" fill-opacity="0.6" stroke="{}" stroke-width="2"/>"##,
        num(p.center.x),
        num(p.center.y),
        num(p.radius),
        p.pole.color(),
    )?;
    // 極名は円の外側 (中心から離れる方向) に置き、ドットと重ねない。
    let direction = p.center.to_vec2();
    let label_direction = if direction.hypot() < 1e-6 {
        kurbo::Vec2::new(0.0, -1.0)
    } else {
        direction / direction.hypot()
    };
    let label_at = p.center + label_direction * (p.radius + 14.0);
    writeln!(
        out,
        r#"<text class="belka-pole-label" x="{}" y="{}" font-size="12" fill="{}" text-anchor="middle">{}</text>"#,
        num(label_at.x),
        num(label_at.y + 4.0), // ベースライン補正
        p.pole.color(),
        p.pole.label(),
    )?;

    // 操作ドット: ひまわり配置で円内を均一に埋める (個数によらず決定論的)。
    let dots = model.dots.get(&p.pole).map_or(&[][..], Vec::as_slice);
    let count = dots.len();
    for (index, category) in dots.iter().enumerate() {
        let fraction = (usize_to_f64(index) + 0.5) / usize_to_f64(count.max(1));
        let r = (p.radius - DOT_RADIUS - 4.0).max(0.0) * fraction.sqrt();
        let theta = usize_to_f64(index) * GOLDEN_ANGLE;
        writeln!(
            out,
            r#"<circle class="op-dot" cx="{}" cy="{}" r="{}" fill="{}"/>"#,
            num(p.center.x + r * theta.cos()),
            num(p.center.y + r * theta.sin()),
            num(DOT_RADIUS),
            palette::color_of(*category),
        )?;
    }
    Ok(())
}

/// カウントは 2^53 未満なので精度劣化は起きない (layout 側と同じ判断)。
#[allow(clippy::cast_precision_loss)]
fn usize_to_f64(value: usize) -> f64 {
    value as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{EffectSet, OperationPayload};

    fn op(kind: OperationKind, defs: &[&str], uses: &[&str], io: bool) -> Operation {
        Operation {
            kind,
            effects: EffectSet {
                io,
                ..EffectSet::default()
            },
            payload: OperationPayload {
                defs: defs.iter().map(ToString::to_string).collect(),
                uses: uses.iter().map(ToString::to_string).collect(),
                ..OperationPayload::default()
            },
        }
    }

    #[test]
    fn pole_classification_follows_priority() {
        // 帰還は defs があっても消費。
        let ret = op(OperationKind::Return, &[], &["x"], false);
        assert_eq!(pole_of(&ret, false), Pole::Consume);
        // 副作用 (io) は消費。
        let print = op(OperationKind::Call, &[], &["x"], true);
        assert_eq!(pole_of(&print, false), Pole::Consume);
        // uses なしの def は生成。
        let birth = op(OperationKind::Compute, &["x"], &[], false);
        assert_eq!(pole_of(&birth, false), Pole::Genesis);
        // 読みながら書くのは変換。
        let rewrite = op(OperationKind::Compute, &["x"], &["x", "y"], false);
        assert_eq!(pole_of(&rewrite, false), Pole::Transmute);
        // dataflow 情報を持たない純粋計算は中立 (変換)。
        let neutral = op(OperationKind::Compute, &[], &[], false);
        assert_eq!(pole_of(&neutral, false), Pole::Transmute);
        // MainRing 末尾 (暗黙の戻り値) は帰結として消費。
        assert_eq!(pole_of(&rewrite, true), Pole::Consume);
    }

    #[test]
    fn skewed_weights_distort_the_triangle() {
        let mut model = BelkaModel {
            dots: POLES.iter().map(|p| (*p, Vec::new())).collect(),
            flows: BTreeMap::new(),
        };
        model
            .dots
            .get_mut(&Pole::Genesis)
            .unwrap()
            .extend([EffectCategory::Pure; 9]);
        model
            .dots
            .get_mut(&Pole::Consume)
            .unwrap()
            .push(EffectCategory::Pure);
        let placed = place_poles(&model);
        let genesis = placed.iter().find(|p| p.pole == Pole::Genesis).unwrap();
        let consume = placed.iter().find(|p| p.pole == Pole::Consume).unwrap();
        assert!(
            genesis.center.to_vec2().hypot() > consume.center.to_vec2().hypot(),
            "重い極ほど中心から遠い"
        );
        assert!(genesis.radius > consume.radius, "重い極ほど大きい");
    }
}
