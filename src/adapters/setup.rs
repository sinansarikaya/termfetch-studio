//! Makes the system match the applied theme: the shell startup block and, when the
//! theme's colors are applied, the Windows Terminal scheme.

use crate::adapters::{shell, wt_terminal};
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
