use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "termfetch-studio",
    author = "Sinan Sarıkaya",
    version = "2.0.0",
    about = "Cross-platform terminal customization and Fastfetch theme studio"
)]
pub struct Cli {
    /// Start with privacy demo mode on (mock specs)
    #[arg(long, global = true, overrides_with = "no_demo")]
    pub demo: bool,

    /// Start with demo mode off (real system specs)
    #[arg(long, global = true)]
    pub no_demo: bool,

    /// Print a non-interactive preview of a theme id (e.g. catppuccin-mocha)
    #[arg(short, long, value_name = "THEME")]
    pub preview: Option<String>,

    /// Run diagnostics (same as the `doctor` subcommand)
    #[arg(long)]
    pub doctor: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    /// `Some(true)` for --demo, `Some(false)` for --no-demo, `None` if neither.
    pub fn demo_override(&self) -> Option<bool> {
        match (self.demo, self.no_demo) {
            (true, _) => Some(true),
            (_, true) => Some(false),
            _ => None,
        }
    }
}

#[derive(Args, Debug, Default, Clone)]
pub struct Selection {
    /// Theme id (see `list themes`)
    #[arg(long)]
    pub theme: Option<String>,
    /// Preset id: full, minimal, focused, developer, gaming, custom
    #[arg(long)]
    pub preset: Option<String>,
    /// Logo id (see `list logos`)
    #[arg(long)]
    pub logo: Option<String>,
    /// Image logo size: small, medium, large
    #[arg(long)]
    pub logo_size: Option<String>,
    /// Icon colour: theme, default, 31-37, 90-97 or #rrggbb
    #[arg(long)]
    pub icon_color: Option<String>,
    /// Text colour: theme, default, 31-37, 90-97 or #rrggbb
    #[arg(long)]
    pub text_color: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Generate and atomically write both fastfetch configs
    Apply(Selection),
    /// Print the live preview to stdout
    Preview(Selection),
    /// Run fastfetch with the generated studio config
    Run,
    /// Show or change demo mode (regenerates the config)
    Demo {
        #[arg(value_enum, default_value_t = DemoArg::Status)]
        state: DemoArg,
    },
    /// List catalogue entries
    List {
        #[arg(value_enum)]
        what: ListArg,
    },
    /// Backup & restore settings and configs
    Backup {
        #[command(subcommand)]
        action: BackupCmd,
    },
    /// Run diagnostics
    Doctor,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoArg {
    On,
    Off,
    Toggle,
    Status,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListArg {
    Themes,
    Presets,
    Logos,
    Modules,
    Colors,
}

#[derive(Subcommand, Debug)]
pub enum BackupCmd {
    Create,
    List,
    Restore { name: String },
    Delete { name: String },
}
