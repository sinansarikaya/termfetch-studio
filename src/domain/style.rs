//! Single source of truth for which colour every element gets. Both the
//! fastfetch config generator and the TUI preview call into this module.

use crate::domain::color::{ColorValue, SemanticGroup};
use crate::domain::settings::Settings;
use crate::domain::theme::Theme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleColors {
    pub key: ColorValue,
    pub output: ColorValue,
}

fn concrete(v: &ColorValue, theme_hex: &str) -> ColorValue {
    match v {
        ColorValue::Theme | ColorValue::Inherit => ColorValue::Hex(theme_hex.to_string()),
        other => other.clone(),
    }
}

/// Key/output colour for a module group, following the original rules:
/// a group override paints icon and text; otherwise the icon/text settings
/// apply, where `theme` means the group colour (icons) / theme foreground (text).
pub fn module_colors(theme: &Theme, settings: &Settings, group: SemanticGroup) -> ModuleColors {
    let group_hex = group.theme_hex(theme);
    match settings.group_colors.get(group) {
        ColorValue::Inherit => ModuleColors {
            key: concrete(&settings.icon_color, group_hex),
            output: concrete(&settings.text_color, &theme.text),
        },
        over => {
            let c = concrete(over, group_hex);
            ModuleColors {
                key: c.clone(),
                output: c,
            }
        }
    }
}

pub fn title_color(theme: &Theme) -> ColorValue {
    ColorValue::Hex(theme.primary.clone())
}

pub fn text_color(theme: &Theme, settings: &Settings) -> ColorValue {
    concrete(&settings.text_color, &theme.text)
}

pub fn logo_color(theme: &Theme) -> ColorValue {
    ColorValue::Hex(theme.primary.clone())
}

/// Eight `●` dots: primary, secondary, accent, then the five group colours.
pub fn dot_colors(theme: &Theme, settings: &Settings) -> Vec<ColorValue> {
    let mut v = vec![
        ColorValue::Hex(theme.primary.clone()),
        ColorValue::Hex(theme.secondary.clone()),
        ColorValue::Hex(theme.accent.clone()),
    ];
    v.extend(
        SemanticGroup::ALL
            .iter()
            .map(|&g| module_colors(theme, settings, g).key),
    );
    v
}
