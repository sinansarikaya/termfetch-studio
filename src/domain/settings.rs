//! Persistent user choices (`~/.config/termfetch-studio/settings.json`).

use crate::adapters::fsutil::atomic_write;
use crate::domain::color::{ColorValue, SemanticGroup};
use crate::domain::preset::{ModuleKind, DEFAULT_CUSTOM_MODULES};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogoSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl LogoSize {
    /// Maximum logo box in terminal cells (original `configured_logo_dimensions`).
    pub fn max_cells(self) -> (u16, u16) {
        match self {
            Self::Small => (20, 14),
            Self::Medium => (36, 22),
            Self::Large => (52, 32),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Small => Self::Medium,
            Self::Medium => Self::Large,
            Self::Large => Self::Small,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "small" | "s" => Some(Self::Small),
            "medium" | "m" => Some(Self::Medium),
            "large" | "l" => Some(Self::Large),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupColors {
    pub os: ColorValue,
    pub desktop: ColorValue,
    pub hardware: ColorValue,
    pub network: ColorValue,
    pub time: ColorValue,
}

impl Default for GroupColors {
    fn default() -> Self {
        Self {
            os: ColorValue::Inherit,
            desktop: ColorValue::Inherit,
            hardware: ColorValue::Inherit,
            network: ColorValue::Inherit,
            time: ColorValue::Inherit,
        }
    }
}

impl GroupColors {
    pub fn get(&self, g: SemanticGroup) -> &ColorValue {
        match g {
            SemanticGroup::Os => &self.os,
            SemanticGroup::Desktop => &self.desktop,
            SemanticGroup::Hardware => &self.hardware,
            SemanticGroup::Network => &self.network,
            SemanticGroup::Time => &self.time,
        }
    }

    pub fn get_mut(&mut self, g: SemanticGroup) -> &mut ColorValue {
        match g {
            SemanticGroup::Os => &mut self.os,
            SemanticGroup::Desktop => &mut self.desktop,
            SemanticGroup::Hardware => &mut self.hardware,
            SemanticGroup::Network => &mut self.network,
            SemanticGroup::Time => &mut self.time,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: String,
    pub preset: String,
    pub logo: String,
    pub logo_size: LogoSize,
    pub demo_mode: bool,
    /// Icon (key) colour; `theme` = per-group theme colour.
    pub icon_color: ColorValue,
    /// Text (output) colour; `theme` = theme foreground.
    pub text_color: ColorValue,
    pub group_colors: GroupColors,
    pub custom_modules: Vec<ModuleKind>,
    /// Send OSC 10/11 so the host terminal adopts the theme fg/bg on apply.
    pub apply_terminal_colors: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "catppuccin-mocha".into(),
            preset: "full".into(),
            logo: "arch".into(),
            logo_size: LogoSize::Medium,
            demo_mode: true,
            icon_color: ColorValue::Theme,
            text_color: ColorValue::Theme,
            group_colors: GroupColors::default(),
            custom_modules: DEFAULT_CUSTOM_MODULES.to_vec(),
            apply_terminal_colors: true,
        }
    }
}

impl Settings {
    /// Missing file -> defaults. A corrupt file is reported, not silently replaced.
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text)
                .with_context(|| format!("parsing {}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    pub fn load_or_default(path: &Path) -> Self {
        Self::load(path).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let mut json = serde_json::to_string_pretty(self)?;
        json.push('\n');
        atomic_write(path, json.as_bytes())
    }

    pub fn reset_colors(&mut self) {
        self.icon_color = ColorValue::Theme;
        self.text_color = ColorValue::Theme;
        self.group_colors = GroupColors::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_partial_files() {
        let dir = std::env::temp_dir().join(format!("tfs-settings-{}", std::process::id()));
        let p = dir.join("settings.json");
        let mut s = Settings::default();
        s.demo_mode = false;
        s.group_colors.network = ColorValue::Hex("#ff8c00".into());
        s.custom_modules = vec![ModuleKind::Cpu, ModuleKind::Gpu];
        s.save(&p).unwrap();
        assert_eq!(Settings::load(&p).unwrap(), s);

        std::fs::write(&p, r#"{"theme":"nord"}"#).unwrap();
        let partial = Settings::load(&p).unwrap();
        assert_eq!(partial.theme, "nord");
        assert!(partial.demo_mode, "missing fields fall back to defaults");

        std::fs::write(&p, "{not json").unwrap();
        assert!(Settings::load(&p).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
