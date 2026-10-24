use clap::Parser;
use termfetch_studio_v2::cli::{Cli, Command};

#[test]
fn demo_flags_toggle_both_ways() {
    assert_eq!(Cli::parse_from(["tfs"]).demo_override(), None);
    assert_eq!(Cli::parse_from(["tfs", "--demo"]).demo_override(), Some(true));
    assert_eq!(Cli::parse_from(["tfs", "--no-demo"]).demo_override(), Some(false));
    assert_eq!(Cli::parse_from(["tfs", "apply", "--theme", "nord", "--no-demo"]).demo_override(), Some(false));
}

#[test]
fn apply_subcommand_parses_selection() {
    let cli = Cli::parse_from(["tfs", "apply", "--theme", "nord", "--preset", "minimal", "--logo-size", "large"]);
    match cli.command {
        Some(Command::Apply(sel)) => {
            assert_eq!(sel.theme.as_deref(), Some("nord"));
            assert_eq!(sel.preset.as_deref(), Some("minimal"));
            assert_eq!(sel.logo_size.as_deref(), Some("large"));
        }
        other => panic!("{other:?}"),
    }
}
