use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeCategory {
    Community,
    Dark,
    Light,
    Pastel,
    PopCulture,
}

impl ThemeCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Community => "Top Community",
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Pastel => "Pastel",
            Self::PopCulture => "Pop Culture & Gaming",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticGroups {
    pub os: String,
    pub desktop: String,
    pub hardware: String,
    pub network: String,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub category: ThemeCategory,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub text: String,
    pub groups: SemanticGroups,
}

impl Theme {
    pub fn parse_color(hex: &str) -> Color {
        let clean = hex.trim().trim_start_matches('#');
        if clean.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&clean[0..2], 16),
                u8::from_str_radix(&clean[2..4], 16),
                u8::from_str_radix(&clean[4..6], 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
        Color::Reset
    }

    pub fn primary_color(&self) -> Color {
        Self::parse_color(&self.primary)
    }

    pub fn secondary_color(&self) -> Color {
        Self::parse_color(&self.secondary)
    }

    pub fn accent_color(&self) -> Color {
        Self::parse_color(&self.accent)
    }

    pub fn background_color(&self) -> Color {
        Self::parse_color(&self.background)
    }

    pub fn text_color(&self) -> Color {
        Self::parse_color(&self.text)
    }

    /// OSC sequences to apply background and foreground to host terminal
    pub fn osc_sequences(&self) -> String {
        format!(
            "\x1b]10;{}\x07\x1b]11;{}\x07",
            self.text, self.background
        )
    }
}

pub fn get_builtin_themes() -> Vec<Theme> {
    vec![
        // ==========================================
        // 1. TOP COMMUNITY PALETTES (NEW v2)
        // ==========================================
        Theme {
            id: "catppuccin-mocha".into(),
            name: "Catppuccin Mocha ☕".into(),
            category: ThemeCategory::Community,
            primary: "#89b4fa".into(),
            secondary: "#f5c2e7".into(),
            accent: "#a6e3a1".into(),
            background: "#1e1e2e".into(),
            text: "#cdd6f4".into(),
            groups: SemanticGroups {
                os: "#89b4fa".into(),
                desktop: "#cba6f7".into(),
                hardware: "#f9e2af".into(),
                network: "#a6e3a1".into(),
                time: "#94e2d5".into(),
            },
        },
        Theme {
            id: "catppuccin-latte".into(),
            name: "Catppuccin Latte 🥛".into(),
            category: ThemeCategory::Community,
            primary: "#1e66f5".into(),
            secondary: "#ea76cb".into(),
            accent: "#40a02b".into(),
            background: "#eff1f5".into(),
            text: "#4c4f69".into(),
            groups: SemanticGroups {
                os: "#1e66f5".into(),
                desktop: "#8839ef".into(),
                hardware: "#df8e1d".into(),
                network: "#40a02b".into(),
                time: "#179299".into(),
            },
        },
        Theme {
            id: "rose-pine".into(),
            name: "Rosé Pine 🌲".into(),
            category: ThemeCategory::Community,
            primary: "#ebbcba".into(),
            secondary: "#c4a7e7".into(),
            accent: "#9ccfd8".into(),
            background: "#191724".into(),
            text: "#e0def4".into(),
            groups: SemanticGroups {
                os: "#ebbcba".into(),
                desktop: "#c4a7e7".into(),
                hardware: "#f6c177".into(),
                network: "#9ccfd8".into(),
                time: "#31748f".into(),
            },
        },
        Theme {
            id: "kanagawa".into(),
            name: "Kanagawa 🌊".into(),
            category: ThemeCategory::Community,
            primary: "#7e9cd8".into(),
            secondary: "#957fb8".into(),
            accent: "#76946a".into(),
            background: "#1f1f28".into(),
            text: "#dcd7ba".into(),
            groups: SemanticGroups {
                os: "#7e9cd8".into(),
                desktop: "#957fb8".into(),
                hardware: "#dca561".into(),
                network: "#76946a".into(),
                time: "#6a9589".into(),
            },
        },
        Theme {
            id: "everforest".into(),
            name: "Everforest Dark 🍃".into(),
            category: ThemeCategory::Community,
            primary: "#a7c080".into(),
            secondary: "#d699b6".into(),
            accent: "#83c092".into(),
            background: "#2d353b".into(),
            text: "#d3c6aa".into(),
            groups: SemanticGroups {
                os: "#a7c080".into(),
                desktop: "#7fbbb3".into(),
                hardware: "#dbbc7f".into(),
                network: "#83c092".into(),
                time: "#e69875".into(),
            },
        },

        // ==========================================
        // 2. CLASSIC DARK PALETTES
        // ==========================================
        Theme {
            id: "dracula".into(),
            name: "Dracula 🧛".into(),
            category: ThemeCategory::Dark,
            primary: "#bd93f9".into(),
            secondary: "#ff79c6".into(),
            accent: "#50fa7b".into(),
            background: "#282a36".into(),
            text: "#f8f8f2".into(),
            groups: SemanticGroups {
                os: "#bd93f9".into(),
                desktop: "#ff79c6".into(),
                hardware: "#f1fa8c".into(),
                network: "#50fa7b".into(),
                time: "#8be9fd".into(),
            },
        },
        Theme {
            id: "nord".into(),
            name: "Nord ❄️".into(),
            category: ThemeCategory::Dark,
            primary: "#81a1c1".into(),
            secondary: "#88c0d0".into(),
            accent: "#a3be8c".into(),
            background: "#2e3440".into(),
            text: "#d8dee9".into(),
            groups: SemanticGroups {
                os: "#81a1c1".into(),
                desktop: "#b48ead".into(),
                hardware: "#ebcb8b".into(),
                network: "#a3be8c".into(),
                time: "#88c0d0".into(),
            },
        },
        Theme {
            id: "tokyo-night".into(),
            name: "Tokyo Night 🌃".into(),
            category: ThemeCategory::Dark,
            primary: "#7aa2f7".into(),
            secondary: "#bb9af7".into(),
            accent: "#9ece6a".into(),
            background: "#1a1b26".into(),
            text: "#c0caf5".into(),
            groups: SemanticGroups {
                os: "#7aa2f7".into(),
                desktop: "#bb9af7".into(),
                hardware: "#e0af68".into(),
                network: "#9ece6a".into(),
                time: "#7dcfff".into(),
            },
        },
        Theme {
            id: "gruvbox-dark".into(),
            name: "Gruvbox Dark 📦".into(),
            category: ThemeCategory::Dark,
            primary: "#d79921".into(),
            secondary: "#458588".into(),
            accent: "#98971a".into(),
            background: "#282828".into(),
            text: "#ebdbb2".into(),
            groups: SemanticGroups {
                os: "#d79921".into(),
                desktop: "#b16286".into(),
                hardware: "#fabd2f".into(),
                network: "#98971a".into(),
                time: "#689d6a".into(),
            },
        },
        Theme {
            id: "one-dark".into(),
            name: "One Dark 🌌".into(),
            category: ThemeCategory::Dark,
            primary: "#61afef".into(),
            secondary: "#c678dd".into(),
            accent: "#98c379".into(),
            background: "#282c34".into(),
            text: "#abb2bf".into(),
            groups: SemanticGroups {
                os: "#61afef".into(),
                desktop: "#c678dd".into(),
                hardware: "#e5c07b".into(),
                network: "#98c379".into(),
                time: "#56b6c2".into(),
            },
        },
        Theme {
            id: "synthwave".into(),
            name: "Synthwave '84 🎸".into(),
            category: ThemeCategory::Dark,
            primary: "#36f9f6".into(),
            secondary: "#ff7edb".into(),
            accent: "#72f1b8".into(),
            background: "#2a2139".into(),
            text: "#ffffff".into(),
            groups: SemanticGroups {
                os: "#36f9f6".into(),
                desktop: "#ff7edb".into(),
                hardware: "#fed330".into(),
                network: "#72f1b8".into(),
                time: "#fe4450".into(),
            },
        },
        Theme {
            id: "oceanic-next".into(),
            name: "Oceanic Next 🌊".into(),
            category: ThemeCategory::Dark,
            primary: "#6699cc".into(),
            secondary: "#5fb3b3".into(),
            accent: "#99c794".into(),
            background: "#1b2b34".into(),
            text: "#c0c5ce".into(),
            groups: SemanticGroups {
                os: "#6699cc".into(),
                desktop: "#c594c5".into(),
                hardware: "#fac863".into(),
                network: "#99c794".into(),
                time: "#ec5f67".into(),
            },
        },
        Theme {
            id: "monokai-pro".into(),
            name: "Monokai Pro 👾".into(),
            category: ThemeCategory::Dark,
            primary: "#fc9867".into(),
            secondary: "#ab9df2".into(),
            accent: "#a9dc76".into(),
            background: "#2d2a2e".into(),
            text: "#fcfcfa".into(),
            groups: SemanticGroups {
                os: "#fc9867".into(),
                desktop: "#ab9df2".into(),
                hardware: "#ffd866".into(),
                network: "#a9dc76".into(),
                time: "#78dce8".into(),
            },
        },
        Theme {
            id: "palenight".into(),
            name: "Material Palenight 🔮".into(),
            category: ThemeCategory::Dark,
            primary: "#82aaff".into(),
            secondary: "#c792ea".into(),
            accent: "#c3e88d".into(),
            background: "#292d3e".into(),
            text: "#a6accd".into(),
            groups: SemanticGroups {
                os: "#82aaff".into(),
                desktop: "#c792ea".into(),
                hardware: "#ffcb6b".into(),
                network: "#c3e88d".into(),
                time: "#89ddff".into(),
            },
        },
        Theme {
            id: "monochrome".into(),
            name: "Monochrome Minimal 🖤".into(),
            category: ThemeCategory::Dark,
            primary: "#ffffff".into(),
            secondary: "#cccccc".into(),
            accent: "#888888".into(),
            background: "#000000".into(),
            text: "#ffffff".into(),
            groups: SemanticGroups {
                os: "#ffffff".into(),
                desktop: "#dddddd".into(),
                hardware: "#aaaaaa".into(),
                network: "#888888".into(),
                time: "#666666".into(),
            },
        },

        // ==========================================
        // 3. AUTHENTIC LIGHT THEMES
        // ==========================================
        Theme {
            id: "solarized-light".into(),
            name: "Solarized Light ☀️".into(),
            category: ThemeCategory::Light,
            primary: "#b58900".into(), // Ethan Schoonover amber
            secondary: "#268bd2".into(), // Blue
            accent: "#2aa198".into(), // Cyan
            background: "#fdf6e3".into(), // Base3 authentic ivory
            text: "#657b83".into(), // Base00
            groups: SemanticGroups {
                os: "#b58900".into(),
                desktop: "#6c71c4".into(),
                hardware: "#cb4b16".into(),
                network: "#859900".into(),
                time: "#2aa198".into(),
            },
        },
        Theme {
            id: "ayu-light".into(),
            name: "Ayu Light 🌤️".into(),
            category: ThemeCategory::Light,
            primary: "#ff9940".into(), // Ayu orange
            secondary: "#399ee6".into(), // Ayu blue
            accent: "#86b300".into(), // Ayu green
            background: "#fafafa".into(), // Authentic clean white
            text: "#5c6166".into(),
            groups: SemanticGroups {
                os: "#ff9940".into(),
                desktop: "#a37acc".into(),
                hardware: "#f2ae49".into(),
                network: "#86b300".into(),
                time: "#4cbf99".into(),
            },
        },
        Theme {
            id: "gruvbox-light".into(),
            name: "Gruvbox Light 📜".into(),
            category: ThemeCategory::Light,
            primary: "#b57614".into(),
            secondary: "#076678".into(),
            accent: "#79740e".into(),
            background: "#fbf1c7".into(),
            text: "#3c3836".into(),
            groups: SemanticGroups {
                os: "#b57614".into(),
                desktop: "#8f3f71".into(),
                hardware: "#af3a03".into(),
                network: "#79740e".into(),
                time: "#427b58".into(),
            },
        },
        Theme {
            id: "one-light".into(),
            name: "One Light 💡".into(),
            category: ThemeCategory::Light,
            primary: "#4078f2".into(),
            secondary: "#a626a4".into(),
            accent: "#50a14f".into(),
            background: "#fafafa".into(),
            text: "#383a42".into(),
            groups: SemanticGroups {
                os: "#4078f2".into(),
                desktop: "#a626a4".into(),
                hardware: "#c18401".into(),
                network: "#50a14f".into(),
                time: "#0184bc".into(),
            },
        },

        // ==========================================
        // 4. PASTEL & AESTHETIC PALETTES
        // ==========================================
        Theme {
            id: "sakura".into(),
            name: "Sakura Cherry 🌸".into(),
            category: ThemeCategory::Pastel,
            primary: "#ff69b4".into(),
            secondary: "#ff1493".into(),
            accent: "#98d8c8".into(),
            background: "#2b2129".into(),
            text: "#fff0f5".into(),
            groups: SemanticGroups {
                os: "#ff69b4".into(),
                desktop: "#ffb7c5".into(),
                hardware: "#ffd1dc".into(),
                network: "#98fb98".into(),
                time: "#e6e6fa".into(),
            },
        },
        Theme {
            id: "lavender".into(),
            name: "Lavender Dream 💜".into(),
            category: ThemeCategory::Pastel,
            primary: "#9370db".into(),
            secondary: "#ba55d3".into(),
            accent: "#b19cd9".into(),
            background: "#262235".into(),
            text: "#f5f3ff".into(),
            groups: SemanticGroups {
                os: "#9370db".into(),
                desktop: "#ba55d3".into(),
                hardware: "#dcd0ff".into(),
                network: "#b19cd9".into(),
                time: "#e6e6fa".into(),
            },
        },
        Theme {
            id: "candy".into(),
            name: "Candy Pastel 🍬".into(),
            category: ThemeCategory::Pastel,
            primary: "#ff85ad".into(),
            secondary: "#d9b3ff".into(),
            accent: "#c2f0c2".into(),
            background: "#2e2530".into(),
            text: "#ffe6f0".into(),
            groups: SemanticGroups {
                os: "#ff85ad".into(),
                desktop: "#d9b3ff".into(),
                hardware: "#ffe0b2".into(),
                network: "#c2f0c2".into(),
                time: "#b3e5fc".into(),
            },
        },

        // ==========================================
        // 5. POP CULTURE & GAMING PALETTES
        // ==========================================
        Theme {
            id: "matrix".into(),
            name: "Matrix 💚".into(),
            category: ThemeCategory::PopCulture,
            primary: "#00ff00".into(),
            secondary: "#00aa00".into(),
            accent: "#003300".into(),
            background: "#000000".into(),
            text: "#00ff00".into(),
            groups: SemanticGroups {
                os: "#00ff00".into(),
                desktop: "#00ff66".into(),
                hardware: "#55ff55".into(),
                network: "#008f11".into(),
                time: "#00dd33".into(),
            },
        },
        Theme {
            id: "cyberpunk".into(),
            name: "Cyberpunk 2077 🌆".into(),
            category: ThemeCategory::PopCulture,
            primary: "#ffee00".into(),
            secondary: "#ff003c".into(),
            accent: "#00ffff".into(),
            background: "#0a0e27".into(),
            text: "#00ffff".into(),
            groups: SemanticGroups {
                os: "#ffee00".into(),
                desktop: "#ff003c".into(),
                hardware: "#ff7700".into(),
                network: "#00ff9f".into(),
                time: "#00b8ff".into(),
            },
        },
        Theme {
            id: "naruto".into(),
            name: "Naruto Uzumaki 🍜".into(),
            category: ThemeCategory::PopCulture,
            primary: "#ff4500".into(),
            secondary: "#ff8c00".into(),
            accent: "#4169e1".into(),
            background: "#1c140a".into(),
            text: "#ffefd5".into(),
            groups: SemanticGroups {
                os: "#ff4500".into(),
                desktop: "#ff8c00".into(),
                hardware: "#ffd700".into(),
                network: "#4169e1".into(),
                time: "#20b2aa".into(),
            },
        },
        Theme {
            id: "pokemon".into(),
            name: "Pokemon Pikachu ⚡".into(),
            category: ThemeCategory::PopCulture,
            primary: "#ffde00".into(),
            secondary: "#ff0000".into(),
            accent: "#0066cc".into(),
            background: "#1f1d0a".into(),
            text: "#ffffff".into(),
            groups: SemanticGroups {
                os: "#ffde00".into(),
                desktop: "#ff0000".into(),
                hardware: "#ffa500".into(),
                network: "#00ff00".into(),
                time: "#0066cc".into(),
            },
        },
        Theme {
            id: "spiderman".into(),
            name: "Spider-Man 🕷️".into(),
            category: ThemeCategory::PopCulture,
            primary: "#c41e3a".into(),
            secondary: "#0051ba".into(),
            accent: "#0066ff".into(),
            background: "#140508".into(),
            text: "#e8e8e8".into(),
            groups: SemanticGroups {
                os: "#c41e3a".into(),
                desktop: "#0051ba".into(),
                hardware: "#ff4444".into(),
                network: "#0066ff".into(),
                time: "#ffffff".into(),
            },
        },
        Theme {
            id: "doom".into(),
            name: "Doom Slayer 👹".into(),
            category: ThemeCategory::PopCulture,
            primary: "#ff0000".into(),
            secondary: "#8b0000".into(),
            accent: "#8b4513".into(),
            background: "#080101".into(),
            text: "#ff3333".into(),
            groups: SemanticGroups {
                os: "#ff0000".into(),
                desktop: "#8b0000".into(),
                hardware: "#ff5500".into(),
                network: "#8b4513".into(),
                time: "#b22222".into(),
            },
        },
        Theme {
            id: "valorant".into(),
            name: "Valorant Radiant 🔥".into(),
            category: ThemeCategory::PopCulture,
            primary: "#ff4655".into(),
            secondary: "#53fca8".into(),
            accent: "#3fa8c9".into(),
            background: "#0f1923".into(),
            text: "#ece8e1".into(),
            groups: SemanticGroups {
                os: "#ff4655".into(),
                desktop: "#53fca8".into(),
                hardware: "#3fa8c9".into(),
                network: "#ffffff".into(),
                time: "#e83d4c".into(),
            },
        },
        Theme {
            id: "minecraft".into(),
            name: "Minecraft Overworld ⛏️".into(),
            category: ThemeCategory::PopCulture,
            primary: "#7bc043".into(),
            secondary: "#00bfff".into(),
            accent: "#3c2415".into(),
            background: "#182012".into(),
            text: "#ffffff".into(),
            groups: SemanticGroups {
                os: "#7bc043".into(),
                desktop: "#00bfff".into(),
                hardware: "#d4af37".into(),
                network: "#4682b4".into(),
                time: "#8a2be2".into(),
            },
        },
    ]
}
