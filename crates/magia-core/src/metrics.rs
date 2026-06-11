//! モジュール単位のメトリクス集計 (Phase 3.1)。
//!
//! 呪文書き起こし (`transcript`) と差分エンジン (`diff`) が**同じ数字**を報告する
//! ための共有集計。二重実装は数字の食い違い (信頼性の毀損) に直結するため禁止。

use crate::filter::EffectCategory;
use crate::ir::{Module, SigilKind};

/// モジュール (Phase 1〜3 では関数1つ) のメトリクス。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metrics {
    /// 循環的複雑度の近似: 1 + 全リングの分岐数 + ループ数。
    pub complexity: u32,
    /// 記号に現れた純粋以外の効果カテゴリの種類数。
    pub effect_categories: u32,
    /// リング数 (MainRing + AuxRing)。
    pub rings: u32,
    /// 召喚記号 (SummonGlyph / GateGlyph) 数。
    pub glyphs: u32,
    /// 早期リターン経路の総数 (全リングの合算)。
    pub early_returns: u32,
    /// MainRing の Operation 数 (書き起こしの規模表現に使う)。
    pub main_operations: u32,
    /// unsafe な Operation の総数 (CI しきい値判定 spec v0.3 §9.3 の基礎数値)。
    pub unsafe_ops: u32,
}

impl Default for Metrics {
    /// 空モジュールの `measure` と一致する基底値 (複雑度は最低 1)。
    fn default() -> Self {
        Metrics {
            complexity: 1,
            effect_categories: 0,
            rings: 0,
            glyphs: 0,
            early_returns: 0,
            main_operations: 0,
            unsafe_ops: 0,
        }
    }
}

/// モジュールを1回走査してメトリクスを測る。決定論的。
#[must_use]
pub fn measure(module: &Module) -> Metrics {
    let mut complexity: u32 = 1;
    let mut categories: Vec<EffectCategory> = Vec::new();
    let mut rings: u32 = 0;
    let mut glyphs: u32 = 0;
    let mut early_returns: u32 = 0;
    let mut main_operations: u32 = 0;
    let mut unsafe_ops: u32 = 0;

    for sigil in &module.sigils {
        match sigil.kind {
            SigilKind::MainRing => {
                rings += 1;
                main_operations = u32::try_from(sigil.content.len())
                    .expect("1 Sigil の Operation 数が u32 を超えることはない");
            }
            SigilKind::AuxRing => rings += 1,
            SigilKind::SummonGlyph | SigilKind::GateGlyph => glyphs += 1,
        }
        if let Some(info) = sigil.layers.control_flow.as_ref() {
            complexity += info.branch_count + info.loop_count;
            early_returns += info.early_return_count;
        }
        for op in &sigil.content {
            let category = EffectCategory::of(&op.effects);
            if category != EffectCategory::Pure && !categories.contains(&category) {
                categories.push(category);
            }
            if op.effects.unsafe_block {
                unsafe_ops += 1;
            }
        }
    }

    Metrics {
        complexity,
        effect_categories: u32::try_from(categories.len())
            .expect("EffectCategory の種類数が u32 を超えることはない"),
        rings,
        glyphs,
        early_returns,
        main_operations,
        unsafe_ops,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ControlFlowInfo, EffectSet, LayerData, ModuleId, Operation, Sigil};

    #[test]
    fn empty_module_has_baseline_metrics() {
        let module = Module {
            id: ModuleId(0),
            name: "demo".to_string(),
            sigils: Vec::new(),
            edges: Vec::new(),
        };
        let metrics = measure(&module);
        assert_eq!(metrics.complexity, 1);
        assert_eq!(metrics.rings, 0);
    }

    #[test]
    fn counts_are_aggregated_across_sigils() {
        let main = Sigil {
            kind: SigilKind::MainRing,
            content: vec![Operation::default(); 3],
            layers: LayerData {
                control_flow: Some(ControlFlowInfo {
                    branch_count: 1,
                    loop_count: 1,
                    early_return_count: 1,
                    role: None,
                }),
                ..LayerData::default()
            },
            ..Sigil::default()
        };
        let glyph = Sigil {
            kind: SigilKind::SummonGlyph,
            content: vec![Operation {
                effects: EffectSet {
                    io: true,
                    ..EffectSet::default()
                },
                ..Operation::default()
            }],
            ..Sigil::default()
        };
        let module = Module {
            id: ModuleId(0),
            name: "demo".to_string(),
            sigils: vec![main, glyph],
            edges: Vec::new(),
        };
        let metrics = measure(&module);
        assert_eq!(metrics.complexity, 3); // 1 + 分岐1 + ループ1
        assert_eq!(metrics.effect_categories, 1); // io
        assert_eq!(metrics.rings, 1);
        assert_eq!(metrics.glyphs, 1);
        assert_eq!(metrics.early_returns, 1);
        assert_eq!(metrics.main_operations, 3);
    }
}
