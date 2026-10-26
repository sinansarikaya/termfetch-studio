//! Icon / text colour choices shared by the fastfetch config generator and the
//! live preview, so both always resolve to the same colour.

use crate::domain::theme::Theme;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SemanticGroup {
    Os,
    Desktop,
    Hardware,
    Network,
    Time,
}

impl SemanticGroup {
    pub const ALL: [SemanticGroup; 5] = [
        Self::Os,
        Self::Desktop,
        Self::Hardware,
        Self::Network,
        Self::Time,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Os => "OS",
            Self::Desktop => "Desktop",
            Self::Hardware => "Hardware",
            Self::Network => "Network",
            Self::Time => "Time & Media",
        }
    }

    pub fn theme_hex(self, theme: &Theme) -> &str {
        match self {
            Self::Os => &theme.groups.os,
            Self::Desktop => &theme.groups.desktop,
            Self::Hardware => &theme.groups.hardware,
            Self::Network => &theme.groups.network,
            Self::Time => &theme.groups.time,
        }
    }
}

/// A colour setting as stored in `settings.json` (`"theme"`, `"inherit"`,
/// `"default"`, an SGR code such as `"34"` or a hex value).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum ColorValue {
    Inherit,
    Theme,
    Default,
    Ansi(u8),
    Hex(String),
}

/// Same palette and order as the original installer's colour editor.
pub const PALETTE: &[(&str, &str)] = &[
    ("theme", "Theme"),
    ("default", "Default"),
    ("31", "Red"),
    ("32", "Green"),
    ("33", "Yellow"),
    ("34", "Blue"),
    ("35", "Magenta"),
    ("36", "Cyan"),
    ("37", "White"),
    ("90", "Bright Black"),
    ("91", "Bright Red"),
    ("92", "Bright Green"),
    ("93", "Bright Yellow"),
    ("94", "Bright Blue"),
    ("95", "Bright Magenta"),
    ("96", "Bright Cyan"),
    ("97", "Bright White"),
    ("#ff8c00", "Orange"),
    ("#ff7f50", "Coral"),
    ("#ff6347", "Tomato"),
    ("#ffd700", "Gold"),
    ("#adff2f", "Lime"),
    ("#40e0d0", "Turquoise"),
    ("#00bfa5", "Teal"),
    ("#87ceeb", "Sky"),
    ("#4169e1", "Royal Blue"),
    ("#6a5acd", "Slate"),
    ("#8a2be2", "Violet"),
    ("#da70d6", "Orchid"),
    ("#ff69b4", "Pink"),
    ("#dc143c", "Crimson"),
    ("#e6e6fa", "Lavender"),
    ("#c0c0c0", "Silver"),
];

impl ColorValue {
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        match s.to_ascii_lowercase().as_str() {
            "inherit" | "" => return Some(Self::Inherit),
            "theme" => return Some(Self::Theme),
            "default" | "39" => return Some(Self::Default),
            _ => {}
        }
        if let Ok(n) = s.parse::<u8>() {
            if (30..=37).contains(&n) || (90..=97).contains(&n) {
                return Some(Self::Ansi(n));
            }
            return None;
        }
        let hex = s.trim_start_matches('#');
        if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(Self::Hex(format!("#{}", hex.to_ascii_lowercase())));
        }
        None
    }

    pub fn as_setting(&self) -> String {
        match self {
            Self::Inherit => "inherit".into(),
            Self::Theme => "theme".into(),
            Self::Default => "default".into(),
            Self::Ansi(n) => n.to_string(),
            Self::Hex(h) => h.clone(),
        }
    }

    pub fn label(&self) -> String {
        if *self == Self::Inherit {
            return "Inherit".into();
        }
        let key = self.as_setting();
        PALETTE
            .iter()
            .find(|(v, _)| *v == key)
            .map(|(_, n)| (*n).to_string())
            .unwrap_or(key)
    }

    /// Next/previous palette entry. Group overrides additionally offer `Inherit`.
    pub fn cycle(&self, forward: bool, with_inherit: bool) -> Self {
        let mut values: Vec<ColorValue> = Vec::with_capacity(PALETTE.len() + 1);
        if with_inherit {
            values.push(Self::Inherit);
        }
        values.extend(PALETTE.iter().filter_map(|(v, _)| Self::parse(v)));
        let idx = values.iter().position(|v| v == self).unwrap_or(0);
        let n = values.len();
        let next = if forward {
            (idx + 1) % n
        } else {
            (idx + n - 1) % n
        };
        values[next].clone()
    }

    /// Value written into fastfetch's `keyColor` / `outputColor`.
    /// Only call on concrete values (`Default`, `Ansi`, `Hex`).
    pub fn fastfetch_value(&self) -> String {
        match self {
            Self::Default | Self::Inherit | Self::Theme => "39".into(),
            Self::Ansi(n) => n.to_string(),
            Self::Hex(h) => h.clone(),
        }
    }

    /// SGR parameters for raw escape sequences (title / colour dots).
    pub fn sgr(&self) -> String {
        match self {
            Self::Hex(h) => match Theme::parse_color(h) {
                Color::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
                _ => "39".into(),
            },
            Self::Ansi(n) => n.to_string(),
            _ => "39".into(),
        }
    }

    /// Colour used by the TUI preview. `Default` follows the theme foreground,
    /// which is what the terminal shows once the theme's OSC colours are applied.
    pub fn tui_color(&self, default_fg: Color) -> Color {
        match self {
            Self::Hex(h) => Theme::parse_color(h),
            Self::Ansi(n) => match n {
                30 => Color::Black,
                31 => Color::Red,
                32 => Color::Green,
                33 => Color::Yellow,
                34 => Color::Blue,
                35 => Color::Magenta,
                36 => Color::Cyan,
                37 => Color::Gray,
                90 => Color::DarkGray,
                91 => Color::LightRed,
                92 => Color::LightGreen,
                93 => Color::LightYellow,
                94 => Color::LightBlue,
                95 => Color::LightMagenta,
                96 => Color::LightCyan,
                _ => Color::White,
            },
            _ => default_fg,
        }
    }
}

impl From<ColorValue> for String {
    fn from(v: ColorValue) -> Self {
        v.as_setting()
    }
}

impl TryFrom<String> for ColorValue {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s).ok_or_else(|| format!("invalid colour value '{s}'"))
    }
}

const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

fn dist((r, g, b): (u8, u8, u8), (r2, g2, b2): (u8, u8, u8)) -> u32 {
    let d = |a: u8, b: u8| (a as i32 - b as i32).pow(2) as u32;
    d(r, r2) + d(g, g2) + d(b, b2)
}

/// Nearest xterm-256 index: the 6x6x6 cube (16-231) or the grey ramp (232-255).
pub fn nearest_256(r: u8, g: u8, b: u8) -> u8 {
    let level = |v: u8| {
        (0..CUBE.len())
            .min_by_key(|&i| (CUBE[i] as i32 - v as i32).abs())
            .unwrap() as u8
    };
    let (ri, gi, bi) = (level(r), level(g), level(b));
    let cube = (CUBE[ri as usize], CUBE[gi as usize], CUBE[bi as usize]);
    let avg = (r as u32 + g as u32 + b as u32) / 3;
    let gi_ = (avg.saturating_sub(3) / 10).min(23) as u8;
    let grey = 8 + 10 * gi_;
    if dist((r, g, b), (grey, grey, grey)) < dist((r, g, b), cube) {
        232 + gi_
    } else {
        16 + 36 * ri + 6 * gi + bi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_palette_entries() {
        for (v, _) in PALETTE {
            assert!(ColorValue::parse(v).is_some(), "{v}");
        }
        assert_eq!(
            ColorValue::parse("#ABCDEF"),
            Some(ColorValue::Hex("#abcdef".into()))
        );
        assert_eq!(ColorValue::parse("12"), None);
        assert_eq!(ColorValue::parse("zzz"), None);
    }

    #[test]
    fn cycle_wraps_and_includes_inherit_for_groups() {
        assert_eq!(
            ColorValue::Theme.cycle(false, false),
            ColorValue::Hex("#c0c0c0".into())
        );
        assert_eq!(ColorValue::Theme.cycle(false, true), ColorValue::Inherit);
        assert_eq!(ColorValue::Inherit.cycle(true, true), ColorValue::Theme);
    }

    #[test]
    fn sgr_for_hex() {
        assert_eq!(ColorValue::Hex("#ff0001".into()).sgr(), "38;2;255;0;1");
    }

    #[test]
    fn maps_to_cube_and_grey_ramp() {
        assert_eq!(nearest_256(0, 0, 0), 16);
        assert_eq!(nearest_256(255, 255, 255), 231);
        assert_eq!(nearest_256(255, 0, 0), 196);
        assert_eq!(nearest_256(0x80, 0x80, 0x80), 244);
        // Studio navy canvas stays a dark colour, not black or grey noise.
        let navy = nearest_256(0x0b, 0x0f, 0x1a);
        assert!(navy == 233 || navy == 16 || navy == 17, "{navy}");
    }
}
