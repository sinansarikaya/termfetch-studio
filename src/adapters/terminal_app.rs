//! macOS Terminal.app ignores OSC colour sequences, so the theme goes into a "TermFetch"
//! profile instead: a copy of the user's default profile (font, size, window) with the
//! theme colours, made the default. The previous default is remembered and restored on
//! uninstall. Driven with AppleScript, which Terminal accepts from its own shells.

use crate::domain::theme::Theme;
use anyhow::{Context, Result, bail};
use ratatui::style::Color;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const PROFILE: &str = "TermFetch";

/// Smallest window the profile opens: room for a large block-drawn logo beside the info
/// text, which is drawn for the size the window has when the shell starts.
pub const MIN_COLUMNS: u16 = 140;
pub const MIN_ROWS: u16 = 45;

/// Where the name of the profile that was the default before ours is kept.
pub fn previous_file(config_dir: &Path) -> PathBuf {
    config_dir.join("terminal-app-previous-profile")
}

/// AppleScript colour: 16-bit RGB.
fn color(hex: &str) -> Option<String> {
    match Theme::parse_color(hex) {
        Color::Rgb(r, g, b) => Some(format!(
            "{{{}, {}, {}}}",
            u32::from(r) * 257,
            u32::from(g) * 257,
            u32::from(b) * 257
        )),
        _ => None,
    }
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Script that creates or updates the profile, makes it the default and recolours the
/// open tabs. Prints the name of the default profile it replaced.
pub fn apply_script(theme: &Theme) -> Option<String> {
    let bg = color(&theme.background)?;
    let fg = color(&theme.text)?;
    let cursor = color(&theme.primary)?;
    let name = quote(PROFILE);
    Some(format!(
        r#"tell application "Terminal"
	set previous to name of default settings
	if not (exists settings set {name}) then
		set base to default settings
		set s to make new settings set with properties {{name:{name}}}
		set font name of s to font name of base
		set font size of s to font size of base
		set font antialiasing of s to font antialiasing of base
		set number of columns of s to number of columns of base
		set number of rows of s to number of rows of base
	end if
	set s to settings set {name}
	if number of columns of s < {MIN_COLUMNS} then set number of columns of s to {MIN_COLUMNS}
	if number of rows of s < {MIN_ROWS} then set number of rows of s to {MIN_ROWS}
	set background color of s to {bg}
	set normal text color of s to {fg}
	set bold text color of s to {fg}
	set cursor color of s to {cursor}
	set default settings to s
	set startup settings to s
	repeat with w in windows
		repeat with t in tabs of w
			set current settings of t to s
		end repeat
	end repeat
	return previous
end tell"#
    ))
}

/// Script that switches back to `previous` and deletes our profile.
pub fn remove_script(previous: &str) -> String {
    let (name, previous) = (quote(PROFILE), quote(previous));
    format!(
        r#"tell application "Terminal"
	if exists settings set {name} then
		if exists settings set {previous} then
			set s to settings set {previous}
		else
			set s to settings set "Basic"
		end if
		set default settings to s
		set startup settings to s
		repeat with w in windows
			repeat with t in tabs of w
				if name of current settings of t is {name} then set current settings of t to s
			end repeat
		end repeat
		delete settings set {name}
	end if
end tell"#
    )
}

fn osascript(script: &str) -> Result<String> {
    let out = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .context("running osascript")?;
    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Whether the running terminal is Terminal.app (only there is the profile needed, and
/// only there does scripting it need no extra permission).
pub fn is_current_terminal() -> bool {
    cfg!(target_os = "macos") && std::env::var("TERM_PROGRAM").is_ok_and(|p| p == "Apple_Terminal")
}

/// Put `theme` into the TermFetch profile and make it the default.
pub fn apply(theme: &Theme, config_dir: &Path) -> Result<()> {
    let script = apply_script(theme).context("theme colours are not #rrggbb")?;
    let previous = osascript(&script)?;
    let file = previous_file(config_dir);
    if previous != PROFILE && !file.exists() {
        std::fs::create_dir_all(config_dir)?;
        std::fs::write(&file, &previous)?;
    }
    Ok(())
}

/// Restore the profile that was the default before ours. Returns whether anything changed.
pub fn remove(config_dir: &Path) -> Result<bool> {
    let file = previous_file(config_dir);
    let Ok(previous) = std::fs::read_to_string(&file) else {
        return Ok(false);
    };
    osascript(&remove_script(previous.trim()))?;
    std::fs::remove_file(file)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::theme::get_builtin_themes;

    #[test]
    fn apply_script_sets_theme_colours_on_a_copy_of_the_default() {
        let theme = &get_builtin_themes()[0];
        let script = apply_script(theme).unwrap();
        assert!(script.contains("make new settings set with properties {name:\"TermFetch\"}"));
        assert!(script.contains("set font name of s to font name of base"));
        // #1e1e2e as 16-bit channels.
        assert!(script.contains("set background color of s to {7710, 7710, 11822}"));
        assert!(script.contains("set default settings to s"));
        assert!(
            script
                .contains("if number of columns of s < 140 then set number of columns of s to 140")
        );
        assert!(script.ends_with("return previous\nend tell"));
    }

    #[test]
    fn remove_script_restores_the_previous_default_and_quotes_names() {
        let script = remove_script(r#"My "Pro""#);
        assert!(script.contains(r#"settings set "My \"Pro\"""#));
        assert!(script.contains("delete settings set \"TermFetch\""));
    }

    #[test]
    fn remove_without_a_remembered_profile_does_nothing() {
        let dir = std::env::temp_dir().join(format!("tfs-terminal-app-{}", std::process::id()));
        assert!(!remove(&dir).unwrap());
    }
}
