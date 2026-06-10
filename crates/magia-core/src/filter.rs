//! フィルター言語 (Phase 2.3, spec v0.2 §8)。
//!
//! `.magia` ファイルの最小文法をパースし、レンダラに渡す [`FilterSpec`] を作る。
//!
//! ```text
//! # コメント
//! show: control_flow + effects[network, db]
//! hide: type_info
//! ```
//!
//! - ディレクティブは `show:` / `hide:` / `highlight:` の3種。`hide` が `show` に優先する
//! - `effects[カテゴリ, ...]` で効果カテゴリの絞り込みができる (effects レイヤーのみ)
//! - `highlight: changed` は diff 文脈での差分強調 (Phase 3.2, spec v0.3 §8)。
//!   その他の `highlight:` 値と `filter:` は引き続き予約語 (専用エラーで案内する)
//! - 未知のレイヤー名・カテゴリ名はタイポ防止のため行番号つきエラーにする

use std::fmt;
use std::str::FromStr;

/// 描画レイヤー名 (spec v0.2 §5 の語彙、CLI `--layers` と共通)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerName {
    ControlFlow,
    Effects,
    TypeInfo,
}

impl LayerName {
    pub const ALL: [LayerName; 3] = [
        LayerName::ControlFlow,
        LayerName::Effects,
        LayerName::TypeInfo,
    ];

    /// DSL / CLI で使う snake_case 名。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            LayerName::ControlFlow => "control_flow",
            LayerName::Effects => "effects",
            LayerName::TypeInfo => "type_info",
        }
    }
}

impl FromStr for LayerName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LayerName::ALL
            .into_iter()
            .find(|layer| layer.as_str() == s)
            .ok_or_else(|| {
                format!(
                    "未知のレイヤー名 `{s}` (使用可能: {})",
                    LayerName::ALL.map(LayerName::as_str).join(", ")
                )
            })
    }
}

/// 効果カテゴリ (spec §6.1.3 の色相に対応する語彙)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectCategory {
    Pure,
    Io,
    Network,
    Db,
    Filesystem,
    Unsafe,
}

impl EffectCategory {
    /// `EffectSet` を表示カテゴリ1つに潰す。
    ///
    /// `EffectSet` は直交フラグの集合で複数同時に立ちうる (`operation.rs` の規約:
    /// 「矛盾の解消はレンダラの色相規約に委ねる」)。優先順位は**危険度・希少度の
    /// 高い順**: unsafe > network > db > filesystem > io > pure。
    /// レンダラ (色相)・フィルター (絞り込み)・書き起こし (ラベル) が同じ分類を共有する。
    #[must_use]
    pub fn of(effects: &crate::ir::EffectSet) -> Self {
        if effects.unsafe_block {
            EffectCategory::Unsafe
        } else if effects.network {
            EffectCategory::Network
        } else if effects.db {
            EffectCategory::Db
        } else if effects.filesystem {
            EffectCategory::Filesystem
        } else if effects.io {
            EffectCategory::Io
        } else {
            EffectCategory::Pure
        }
    }

    /// 危険度の序列 (大きいほど危険)。同一呼び出し先のカテゴリが揺れたときの
    /// 「危険側を採用する」集計などに使う。
    #[must_use]
    pub fn danger_rank(self) -> u8 {
        match self {
            EffectCategory::Pure => 0,
            EffectCategory::Io => 1,
            EffectCategory::Filesystem => 2,
            EffectCategory::Db => 3,
            EffectCategory::Network => 4,
            EffectCategory::Unsafe => 5,
        }
    }

    pub const ALL: [EffectCategory; 6] = [
        EffectCategory::Pure,
        EffectCategory::Io,
        EffectCategory::Network,
        EffectCategory::Db,
        EffectCategory::Filesystem,
        EffectCategory::Unsafe,
    ];

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            EffectCategory::Pure => "pure",
            EffectCategory::Io => "io",
            EffectCategory::Network => "network",
            EffectCategory::Db => "db",
            EffectCategory::Filesystem => "filesystem",
            EffectCategory::Unsafe => "unsafe",
        }
    }
}

impl FromStr for EffectCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EffectCategory::ALL
            .into_iter()
            .find(|category| category.as_str() == s)
            .ok_or_else(|| {
                format!(
                    "未知の効果カテゴリ `{s}` (使用可能: {})",
                    EffectCategory::ALL.map(EffectCategory::as_str).join(", ")
                )
            })
    }
}

/// `show:` の1セレクタ。`categories` は `effects[...]` のときのみ `Some`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerSelector {
    pub layer: LayerName,
    pub categories: Option<Vec<EffectCategory>>,
}

/// パース済みフィルター (spec v0.2 §8)。`FilterSpec::default()` は「全レイヤー表示」。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FilterSpec {
    /// `show:` で列挙されたセレクタ。`None` は全レイヤー表示。
    pub show: Option<Vec<LayerSelector>>,
    /// `hide:` で列挙されたレイヤー (show より優先)。
    pub hide: Vec<LayerName>,
    /// `highlight: changed` — diff 文脈での差分強調 (spec v0.3 §8)。
    /// diff 文脈なしで指定された場合のエラー化は呼び出し側 (CLI) の責務:
    /// パーサは文脈を知らないため、ここでは真偽だけを運ぶ。
    /// レンダラはこのフィールドを**参照しない** (overlay の有無は `SpellDiff` を
    /// 渡すかどうかで決まる)。本フィールドは文脈バリデーション専用のマーカー。
    pub highlight_changed: bool,
}

impl FilterSpec {
    /// 指定レイヤーだけを表示するフィルター (`--layers` の糖衣が使う)。
    /// `show: None` (全表示) と `Some(vec![])` (全非表示) の区別を呼び出し側に
    /// 意識させないためのコンストラクタ。
    #[must_use]
    pub fn show_only(layers: impl IntoIterator<Item = LayerName>) -> Self {
        Self {
            show: Some(
                layers
                    .into_iter()
                    .map(|layer| LayerSelector {
                        layer,
                        categories: None,
                    })
                    .collect(),
            ),
            hide: Vec::new(),
            highlight_changed: false,
        }
    }

    /// レイヤーを描画すべきか。`hide` が `show` に優先する (spec §8)。
    #[must_use]
    pub fn is_visible(&self, layer: LayerName) -> bool {
        if self.hide.contains(&layer) {
            return false;
        }
        match &self.show {
            None => true,
            Some(selectors) => selectors.iter().any(|s| s.layer == layer),
        }
    }

    /// effects レイヤーのカテゴリ絞り込み。`None` は全カテゴリ。
    #[must_use]
    pub fn effect_categories(&self) -> Option<&[EffectCategory]> {
        self.show
            .as_ref()?
            .iter()
            .find(|s| s.layer == LayerName::Effects)?
            .categories
            .as_deref()
    }

    /// `.magia` テキストをパースする。
    pub fn parse(text: &str) -> Result<FilterSpec, FilterParseError> {
        let mut spec = FilterSpec::default();
        for (index, raw_line) in text.lines().enumerate() {
            let line_no = index + 1;
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let error = |message: String| FilterParseError {
                line: line_no,
                message,
            };
            if let Some(rest) = line.strip_prefix("show:") {
                let selectors = parse_selectors(rest).map_err(error)?;
                let merged = spec.show.get_or_insert_with(Vec::new);
                for selector in selectors {
                    // 同一レイヤーの重複指定は無言マージせずエラーにする
                    // (後続の categories が黙って捨てられる事故の防止)。
                    if merged
                        .iter()
                        .any(|existing| existing.layer == selector.layer)
                    {
                        return Err(FilterParseError {
                            line: line_no,
                            message: format!(
                                "レイヤー `{}` は既に show に指定されています",
                                selector.layer.as_str()
                            ),
                        });
                    }
                    merged.push(selector);
                }
            } else if let Some(rest) = line.strip_prefix("hide:") {
                for selector in parse_selectors(rest).map_err(error)? {
                    if selector.categories.is_some() {
                        return Err(FilterParseError {
                            line: line_no,
                            message: "hide にカテゴリ指定 `[...]` はできません".to_string(),
                        });
                    }
                    spec.hide.push(selector.layer);
                }
            } else if let Some(rest) = line.strip_prefix("highlight:") {
                // spec v0.3 §8: 解禁は `changed` のみ。その他の値は引き続き予約語。
                let value = rest.trim();
                if value == "changed" {
                    spec.highlight_changed = true;
                } else {
                    return Err(error(format!(
                        "highlight: の値 `{value}` は予約されています (現在使用可能: changed)"
                    )));
                }
            } else if line.starts_with("filter:") {
                return Err(error(
                    "filter: は Phase 3 以降で導入予定の予約語です".to_string(),
                ));
            } else {
                return Err(error(format!(
                    "不明なディレクティブです (show: / hide: / highlight: のみ使用可能): {line}"
                )));
            }
        }
        Ok(spec)
    }
}

/// `control_flow + effects[network, db]` 形式のセレクタ列をパースする。
fn parse_selectors(rest: &str) -> Result<Vec<LayerSelector>, String> {
    let mut selectors = Vec::new();
    for part in rest.split('+') {
        let part = part.trim();
        if part.is_empty() {
            return Err("レイヤー名が空です (`+` の前後を確認)".to_string());
        }
        let (name, categories) = match part.split_once('[') {
            None => (part, None),
            Some((name, bracket)) => {
                let inner = bracket
                    .strip_suffix(']')
                    .ok_or_else(|| format!("`]` が閉じていません: {part}"))?;
                // 空ブラケット・末尾カンマは from_str の「未知のカテゴリ ``」より先に
                // 専用メッセージで弾く (誤誘導の防止)。
                if inner.trim().is_empty() {
                    return Err("カテゴリ指定 `[...]` が空です".to_string());
                }
                if inner.split(',').any(|c| c.trim().is_empty()) {
                    return Err(format!("カテゴリ指定に空の要素があります: [{inner}]"));
                }
                let categories = inner
                    .split(',')
                    .map(|c| EffectCategory::from_str(c.trim()))
                    .collect::<Result<Vec<_>, _>>()?;
                (name.trim(), Some(categories))
            }
        };
        let layer = LayerName::from_str(name)?;
        if categories.is_some() && layer != LayerName::Effects {
            return Err(format!(
                "カテゴリ指定 `[...]` は effects レイヤーのみ使用できます (指定: {name})"
            ));
        }
        selectors.push(LayerSelector { layer, categories });
    }
    Ok(selectors)
}

/// フィルターのパースエラー (行番号つき)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for FilterParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}行目: {}", self.line, self.message)
    }
}

impl std::error::Error for FilterParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shows_everything() {
        let spec = FilterSpec::default();
        for layer in LayerName::ALL {
            assert!(spec.is_visible(layer));
        }
        assert!(spec.effect_categories().is_none());
    }

    #[test]
    fn parses_spec_example() {
        let spec = FilterSpec::parse(
            "# コメント\nshow: control_flow + effects[network, db]\nhide: type_info\n",
        )
        .unwrap();
        assert!(spec.is_visible(LayerName::ControlFlow));
        assert!(spec.is_visible(LayerName::Effects));
        assert!(!spec.is_visible(LayerName::TypeInfo));
        assert_eq!(
            spec.effect_categories(),
            Some([EffectCategory::Network, EffectCategory::Db].as_slice())
        );
    }

    #[test]
    fn hide_wins_over_show() {
        let spec = FilterSpec::parse("show: effects\nhide: effects\n").unwrap();
        assert!(!spec.is_visible(LayerName::Effects));
    }

    #[test]
    fn unknown_layer_reports_line_number() {
        let error = FilterSpec::parse("# ok\nshow: controlflow\n").unwrap_err();
        assert_eq!(error.line, 2);
        assert!(error.message.contains("未知のレイヤー名"));
        assert!(error.message.contains("control_flow"), "候補を提示する");
    }

    #[test]
    fn unknown_category_is_rejected() {
        let error = FilterSpec::parse("show: effects[netwrok]\n").unwrap_err();
        assert!(error.message.contains("未知の効果カテゴリ"));
    }

    #[test]
    fn category_on_non_effects_layer_is_rejected() {
        let error = FilterSpec::parse("show: control_flow[io]\n").unwrap_err();
        assert!(error.message.contains("effects レイヤーのみ"));
    }

    #[test]
    fn hide_with_categories_is_rejected() {
        let error = FilterSpec::parse("hide: effects[io]\n").unwrap_err();
        assert!(error.message.contains("hide にカテゴリ指定"));
    }

    #[test]
    fn reserved_directives_are_guided() {
        // notes の旧称 changed_in_pr は spec v0.3 §8 で changed に名称統合された。
        // 旧称で書かれた場合も「changed が使える」案内が出る。
        let error = FilterSpec::parse("highlight: changed_in_pr\n").unwrap_err();
        assert!(error.message.contains("予約"));
        assert!(error.message.contains("changed"), "解禁済みの値を案内する");
        let error = FilterSpec::parse("filter: complexity > 5\n").unwrap_err();
        assert!(error.message.contains("予約語"));
    }

    #[test]
    fn highlight_changed_is_accepted() {
        let spec = FilterSpec::parse("highlight: changed\n").unwrap();
        assert!(spec.highlight_changed);
        // レイヤー指定と独立して機能する (overlay はレイヤー show/hide の影響を受けない)。
        let spec = FilterSpec::parse("show: control_flow\nhighlight: changed\n").unwrap();
        assert!(spec.highlight_changed);
        assert!(!spec.is_visible(LayerName::Effects));
        assert!(!FilterSpec::default().highlight_changed);
    }

    #[test]
    fn unclosed_bracket_is_rejected() {
        let error = FilterSpec::parse("show: effects[io\n").unwrap_err();
        assert!(error.message.contains("閉じていません"));
    }

    #[test]
    fn multiple_show_lines_merge() {
        let spec = FilterSpec::parse("show: control_flow\nshow: effects\n").unwrap();
        assert!(spec.is_visible(LayerName::ControlFlow));
        assert!(spec.is_visible(LayerName::Effects));
        assert!(!spec.is_visible(LayerName::TypeInfo));
    }

    #[test]
    fn duplicate_show_layer_is_rejected() {
        // 重複は無言マージしない (後続 categories の取りこぼし防止)。
        let error = FilterSpec::parse("show: effects[io]\nshow: effects[network]\n").unwrap_err();
        assert_eq!(error.line, 2);
        assert!(error.message.contains("既に show に指定"));
    }

    #[test]
    fn empty_or_trailing_comma_categories_get_dedicated_errors() {
        let error = FilterSpec::parse("show: effects[]\n").unwrap_err();
        assert!(error.message.contains("空です"));
        let error = FilterSpec::parse("show: effects[io,]\n").unwrap_err();
        assert!(error.message.contains("空の要素"));
    }

    #[test]
    fn show_only_constructor_matches_layers_sugar() {
        let spec = FilterSpec::show_only([LayerName::Effects]);
        assert!(spec.is_visible(LayerName::Effects));
        assert!(!spec.is_visible(LayerName::ControlFlow));
        assert!(spec.effect_categories().is_none());
    }
}
