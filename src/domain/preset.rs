//! Fastfetch modules and the presets ported from the original installer
//! (`create_{full,minimal,focused,developer,gaming,custom}_config`).

use crate::domain::color::SemanticGroup;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleKind {
    Os,
    Kernel,
    Packages,
    Shell,
    Wm,
    De,
    WmTheme,
    Icons,
    Cursor,
    Terminal,
    TerminalFont,
    Host,
    Cpu,
    Gpu,
    Memory,
    Swap,
    Disk,
    Battery,
    Display,
    LocalIp,
    PublicIp,
    Wifi,
    DateTime,
    Uptime,
    Media,
}

impl ModuleKind {
    /// Module Builder order (same as the original `configure_custom_preset`).
    pub const ALL: [ModuleKind; 25] = [
        Self::Os,
        Self::Kernel,
        Self::Packages,
        Self::Shell,
        Self::Wm,
        Self::De,
        Self::WmTheme,
        Self::Icons,
        Self::Cursor,
        Self::Terminal,
        Self::TerminalFont,
        Self::Host,
        Self::Cpu,
        Self::Gpu,
        Self::Memory,
        Self::Swap,
        Self::Disk,
        Self::Battery,
        Self::Display,
        Self::LocalIp,
        Self::PublicIp,
        Self::Wifi,
        Self::DateTime,
        Self::Uptime,
        Self::Media,
    ];

    pub fn fastfetch_type(self) -> &'static str {
        match self {
            Self::Os => "os",
            Self::Kernel => "kernel",
            Self::Packages => "packages",
            Self::Shell => "shell",
            Self::Wm => "wm",
            Self::De => "de",
            Self::WmTheme => "wmtheme",
            Self::Icons => "icons",
            Self::Cursor => "cursor",
            Self::Terminal => "terminal",
            Self::TerminalFont => "terminalfont",
            Self::Host => "host",
            Self::Cpu => "cpu",
            Self::Gpu => "gpu",
            Self::Memory => "memory",
            Self::Swap => "swap",
            Self::Disk => "disk",
            Self::Battery => "battery",
            Self::Display => "display",
            Self::LocalIp => "localip",
            Self::PublicIp => "publicip",
            Self::Wifi => "wifi",
            Self::DateTime => "datetime",
            Self::Uptime => "uptime",
            Self::Media => "media",
        }
    }

    pub fn from_type(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|m| m.fastfetch_type().eq_ignore_ascii_case(s.trim()))
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Os => "󰣇",
            Self::Kernel => "󰌢",
            Self::Packages => "󰏖",
            Self::Shell => "󰆍",
            Self::Wm => "󰖲",
            Self::De => "󱂬",
            Self::WmTheme => "󰉼",
            Self::Icons => "󰀻",
            Self::Cursor => "󰘔",
            Self::Terminal => "󰆍",
            Self::TerminalFont => "󰛖",
            Self::Host => "󰌢",
            Self::Cpu => "󰻠",
            Self::Gpu => "󰢮",
            Self::Memory => "󰍛",
            Self::Swap => "󰓡",
            Self::Disk => "󰋊",
            Self::Battery => "󰁹",
            Self::Display => "󰍹",
            Self::LocalIp => "󰩟",
            Self::PublicIp => "󰞉",
            Self::Wifi => "󰖩",
            Self::DateTime => "󰅐",
            Self::Uptime => "󰔚",
            Self::Media => "󰎈",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Os => "Operating System",
            Self::Kernel => "Kernel",
            Self::Packages => "Packages",
            Self::Shell => "Shell",
            Self::Wm => "Window Manager",
            Self::De => "Desktop Environment",
            Self::WmTheme => "WM Theme",
            Self::Icons => "Icon Theme",
            Self::Cursor => "Cursor Theme",
            Self::Terminal => "Terminal",
            Self::TerminalFont => "Terminal Font",
            Self::Host => "Host",
            Self::Cpu => "CPU",
            Self::Gpu => "GPU",
            Self::Memory => "Memory",
            Self::Swap => "Swap",
            Self::Disk => "Disk",
            Self::Battery => "Battery",
            Self::Display => "Display",
            Self::LocalIp => "Local IP",
            Self::PublicIp => "Public IP",
            Self::Wifi => "WiFi",
            Self::DateTime => "Date & Time",
            Self::Uptime => "Uptime",
            Self::Media => "Media",
        }
    }

    /// Semantic colour group (original `module_color_group`).
    pub fn group(self) -> SemanticGroup {
        match self {
            Self::Os | Self::Kernel | Self::Packages | Self::Shell => SemanticGroup::Os,
            Self::Wm
            | Self::De
            | Self::WmTheme
            | Self::Icons
            | Self::Cursor
            | Self::Terminal
            | Self::TerminalFont => SemanticGroup::Desktop,
            Self::Host
            | Self::Cpu
            | Self::Gpu
            | Self::Memory
            | Self::Swap
            | Self::Disk
            | Self::Battery
            | Self::Display => SemanticGroup::Hardware,
            Self::LocalIp | Self::PublicIp | Self::Wifi => SemanticGroup::Network,
            Self::DateTime | Self::Uptime | Self::Media => SemanticGroup::Time,
        }
    }

    /// Privacy-safe value used in demo mode (original `get_demo_format`).
    pub fn demo_value(self) -> &'static str {
        match self {
            Self::Os => DEMO_DATA.os,
            Self::Kernel => DEMO_DATA.kernel,
            Self::Packages => "1337 (pacman), 42 (flatpak)",
            Self::Shell => DEMO_DATA.shell,
            Self::Wm => "Hyprland (Wayland)",
            Self::De => "none",
            Self::WmTheme => "Catppuccin Mocha",
            Self::Icons => "Papirus-Dark",
            Self::Cursor => "Bibata-Modern-Ice",
            Self::Terminal => DEMO_DATA.terminal,
            Self::TerminalFont => "JetBrainsMono Nerd Font (12pt)",
            Self::Host => DEMO_DATA.host,
            Self::Cpu => DEMO_DATA.cpu,
            Self::Gpu => DEMO_DATA.gpu,
            Self::Memory => DEMO_DATA.memory,
            Self::Swap => "8.00 GiB / 16.00 GiB",
            Self::Disk => "512.00 GiB / 2.00 TiB (25%)",
            Self::Battery => "100% [Full]",
            Self::Display => "3840x2160 @ 144Hz",
            Self::LocalIp => DEMO_DATA.local_ip,
            Self::PublicIp => "203.0.113.42",
            Self::Wifi => "MyNetwork (802.11ac)",
            Self::DateTime => "2026-10-24 17:05:00",
            Self::Uptime => DEMO_DATA.uptime,
            Self::Media => "Spotify - Daft Punk - Around the World",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Module groups; fastfetch prints a `break` between consecutive groups.
    pub sections: Vec<Vec<ModuleKind>>,
    /// Coloured `●` row after the title and at the end (all presets except Full).
    pub color_dots: bool,
}

impl Preset {
    pub fn modules(&self) -> impl Iterator<Item = ModuleKind> + '_ {
        self.sections.iter().flatten().copied()
    }

    pub fn module_count(&self) -> usize {
        self.sections.iter().map(Vec::len).sum()
    }

    /// The user's Module Builder selection; a new section starts whenever the
    /// semantic group changes, mirroring the hand-written presets.
    pub fn custom(modules: &[ModuleKind]) -> Self {
        let mut sections: Vec<Vec<ModuleKind>> = Vec::new();
        for &m in modules {
            match sections.last_mut() {
                Some(last) if last.last().map(|l| l.group()) == Some(m.group()) => last.push(m),
                _ => sections.push(vec![m]),
            }
        }
        Self {
            id: "custom".into(),
            name: "Custom Modules".into(),
            description: "Your own module selection and order from the Module Builder".into(),
            sections,
            color_dots: true,
        }
    }
}

pub const DEFAULT_CUSTOM_MODULES: [ModuleKind; 4] = [
    ModuleKind::Os,
    ModuleKind::Kernel,
    ModuleKind::Shell,
    ModuleKind::Uptime,
];

pub fn get_builtin_presets() -> Vec<Preset> {
    use ModuleKind::*;
    vec![
        Preset {
            id: "full".into(),
            name: "Full Info".into(),
            description: "Everything: system, desktop, hardware, network, time & media".into(),
            sections: vec![
                vec![Os, Kernel, Packages, Shell],
                vec![Wm, De, WmTheme, Icons, Cursor, Terminal, TerminalFont],
                vec![Host, Cpu, Gpu, Memory, Swap, Disk, Battery, Display],
                vec![LocalIp, PublicIp, Wifi],
                vec![DateTime, Uptime, Media],
            ],
            color_dots: false,
        },
        Preset {
            id: "minimal".into(),
            name: "Minimal".into(),
            description: "Distraction-free essentials: OS, kernel, shell, uptime".into(),
            sections: vec![vec![Os, Kernel, Shell, Uptime]],
            color_dots: true,
        },
        Preset {
            id: "focused".into(),
            name: "Focused".into(),
            description: "Daily glance: system, terminal and core hardware".into(),
            sections: vec![
                vec![Os, Kernel, Packages, Uptime],
                vec![Terminal, Shell],
                vec![Cpu, Memory, Disk],
            ],
            color_dots: true,
        },
        Preset {
            id: "developer".into(),
            name: "Developer".into(),
            description: "Toolchain view: system, terminal/WM, hardware and network".into(),
            sections: vec![
                vec![Os, Kernel, Packages, Shell],
                vec![Terminal, TerminalFont, De, Wm],
                vec![Cpu, Gpu, Memory, Swap, Disk],
                vec![LocalIp, PublicIp],
            ],
            color_dots: true,
        },
        Preset {
            id: "gaming".into(),
            name: "Gaming".into(),
            description: "Rig showcase: GPU, CPU, display, battery and Wi-Fi".into(),
            sections: vec![
                vec![Os, Kernel, Uptime],
                vec![Cpu, Gpu, Memory, Swap, Disk, Display, Battery],
                vec![LocalIp, Wifi],
            ],
            color_dots: true,
        },
        Preset::custom(&DEFAULT_CUSTOM_MODULES),
    ]
}

pub struct DemoData {
    pub user_host: &'static str,
    pub os: &'static str,
    pub host: &'static str,
    pub kernel: &'static str,
    pub uptime: &'static str,
    pub shell: &'static str,
    pub terminal: &'static str,
    pub cpu: &'static str,
    pub gpu: &'static str,
    pub memory: &'static str,
    pub local_ip: &'static str,
}

pub const DEMO_DATA: DemoData = DemoData {
    user_host: "cyberdeck@matrix-station",
    os: "Arch Linux x86_64",
    host: "CyberDeck Hyperion Edition v2",
    kernel: "Linux 6.12.8-zen1-1-zen",
    uptime: "4 hours, 20 mins",
    shell: "zsh 5.9",
    terminal: "kitty 0.49.2",
    cpu: "AMD Ryzen 9 7950X (32) @ 5.700GHz",
    gpu: "NVIDIA GeForce RTX 4090 (24GB VRAM)",
    memory: "16.42 GiB / 64.00 GiB (25%)",
    local_ip: "192.168.1.100 / 24",
};
