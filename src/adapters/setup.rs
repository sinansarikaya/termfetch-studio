//! Makes the system match the applied theme: the shell startup block and, when the
//! theme's colors are applied, the Windows Terminal scheme.

use crate::adapters::{cmd_autorun, desktop, shell, terminal_app, wt_terminal};
use crate::domain::theme::Theme;
use std::path::Path;

/// Runs every system step and returns one short message per step.
pub fn integrate(config: &Path, theme: &Theme, terminal_colors: bool) -> Vec<String> {
    let mut out = Vec::new();
    match shell::integrate(config) {
        Ok(changed) if changed.is_empty() => out.push("shell startup up to date".into()),
        Ok(_) => out.push("shell startup set up".into()),
        Err(e) => out.push(format!("shell startup failed: {e}")),
    }
    let app = std::env::current_exe().ok();
    if cfg!(windows) {
        let exe = crate::adapters::fastfetch::runner::find_fastfetch();
        out.push(cmd_autorun::integrate(
            cmd_autorun::KEY,
            exe.as_deref(),
            config,
        ));
        if let Some(app) = &app {
            for path in wt_terminal::settings_files() {
                match wt_terminal::ensure_profile(&path, app) {
                    Ok(true) => out.push("Windows Terminal profile \"TermFetch\" added".into()),
                    Ok(false) => {}
                    Err(e) => out.push(format!("Windows Terminal profile not added: {e:#}")),
                }
            }
        }
    }
    if let (true, Some(app), Some(path)) = (cfg!(target_os = "linux"), &app, desktop::path()) {
        match desktop::write(&path, app) {
            Ok(true) => out.push("application menu entry added".into()),
            Ok(false) => {}
            Err(e) => out.push(format!("application menu entry not added: {e:#}")),
        }
    }
    // The app bundle is its own launcher; the .command file is for plain binary installs.
    let bundled = crate::paths::app_bundle_bin_dir().is_some();
    if let (true, false, Some(path)) = (cfg!(target_os = "macos"), bundled, desktop::command_path())
    {
        match desktop::write_command(&path) {
            Ok(true) => out.push("~/Applications/TermFetch.command added".into()),
            Ok(false) => {}
            Err(e) => out.push(format!("TermFetch.command not added: {e:#}")),
        }
    }
    if let Some(bin) = crate::paths::app_bundle_bin_dir() {
        match install_fonts(&bin.join("../Resources/fonts")) {
            Ok(0) => {}
            Ok(_) => out.push("Nerd Font symbols installed".into()),
            Err(e) => out.push(format!("Nerd Font symbols not installed: {e:#}")),
        }
    }
    if let (true, true, Some(dir)) = (
        terminal_colors,
        terminal_app::is_current_terminal(),
        config.parent(),
    ) {
        match terminal_app::apply(theme, dir) {
            Ok(()) => out.push("Terminal.app profile \"TermFetch\" set".into()),
            Err(e) => out.push(format!("Terminal.app colors not set: {e:#}")),
        }
    }
    if terminal_colors {
        let colors = wt_terminal::Colors {
            background: theme.background.clone(),
            foreground: theme.text.clone(),
            cursor: theme.primary.clone(),
            selection: theme.accent.clone(),
        };
        for path in wt_terminal::settings_files() {
            match wt_terminal::apply_scheme(&path, &colors) {
                Ok(true) => out.push("Windows Terminal colors set".into()),
                Ok(false) => out.push("Windows Terminal colors up to date".into()),
                Err(e) => out.push(format!("Windows Terminal colors not set: {e:#}")),
            }
        }
    }
    out
}

/// Copy the fonts the macOS app bundles (Nerd Font symbols, so module icons render in
/// any font) into ~/Library/Fonts. Fonts already there are left alone. Returns how many
/// were copied.
fn install_fonts(from: &Path) -> std::io::Result<usize> {
    let Some(home) = directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf()) else {
        return Ok(0);
    };
    copy_missing(from, &home.join("Library").join("Fonts"))
}

fn copy_missing(from: &Path, to: &Path) -> std::io::Result<usize> {
    let Ok(entries) = std::fs::read_dir(from) else {
        return Ok(0);
    };
    let mut copied = 0;
    for entry in entries.flatten() {
        let dest = to.join(entry.file_name());
        if entry.path().extension().is_some_and(|e| e == "ttf") && !dest.exists() {
            std::fs::create_dir_all(to)?;
            std::fs::copy(entry.path(), dest)?;
            copied += 1;
        }
    }
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonts_are_copied_once_and_existing_ones_kept() {
        let root = std::env::temp_dir().join(format!("tfs-fonts-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (from, to) = (root.join("from"), root.join("to"));
        std::fs::create_dir_all(&from).unwrap();
        std::fs::write(from.join("Symbols.ttf"), b"font").unwrap();
        std::fs::write(from.join("LICENSE"), b"text").unwrap();
        assert_eq!(copy_missing(&from, &to).unwrap(), 1);
        assert!(!to.join("LICENSE").exists());
        std::fs::write(to.join("Symbols.ttf"), b"user's").unwrap();
        assert_eq!(copy_missing(&from, &to).unwrap(), 0);
        assert_eq!(std::fs::read(to.join("Symbols.ttf")).unwrap(), b"user's");
        assert_eq!(copy_missing(&root.join("missing"), &to).unwrap(), 0);
        let _ = std::fs::remove_dir_all(root);
    }
}
