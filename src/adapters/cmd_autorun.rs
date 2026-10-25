//! cmd.exe startup: the per-user AutoRun value runs before every interactive prompt.
//! The app only ever owns its own line. Any other AutoRun value is left untouched,
//! and the line never runs an error when fastfetch is missing.

use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

pub const KEY: &str = r"HKCU\Software\Microsoft\Command Processor";
/// Marks the line as ours: it always points at the studio config.
const OWNER: &str = r"termfetch-studio\fastfetch.jsonc";

/// Characters that end or change a cmd.exe command line; paths containing them are refused.
const UNSAFE: [char; 7] = ['&', '|', '<', '>', '^', '%', '"'];

/// The AutoRun line. Windows Terminal gets the sixel logo; the classic console gets
/// fastfetch's built-in logo. `None` when a path cannot be written safely.
pub fn line(exe: &Path, config: &Path) -> Option<String> {
    let (e, c) = (exe.display().to_string(), config.display().to_string());
    if e.contains(UNSAFE) || c.contains(UNSAFE) {
        return None;
    }
    Some(format!(
        r#"if exist "{e}" (if defined WT_SESSION ("{e}" -c "{c}") else ("{e}" -c "{c}" --logo-type builtin))"#
    ))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Plan {
    /// Write this value (new, or ours and out of date).
    Write(String),
    /// Already current.
    Current,
    /// Someone else's value: leave it alone.
    Foreign,
}

pub fn plan(current: Option<&str>, line: &str) -> Plan {
    match current {
        None => Plan::Write(line.to_string()),
        Some(v) if v == line => Plan::Current,
        Some(v) if v.contains(OWNER) => Plan::Write(line.to_string()),
        Some(_) => Plan::Foreign,
    }
}

/// Parses the `reg query` output for the AutoRun value.
pub fn parse_query(text: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let l = l.trim_start();
        let rest = l.strip_prefix("AutoRun")?;
        let (kind, value) = rest.trim_start().split_once("REG_SZ")?;
        kind.is_empty().then(|| value.trim().to_string())
    })
}

fn read(key: &str) -> Result<Option<String>> {
    let out = Command::new("reg")
        .args(["query", key, "/v", "AutoRun"])
        .output()
        .context("running reg query")?;
    // reg exits with 1 when the value does not exist.
    Ok(out
        .status
        .success()
        .then(|| parse_query(&String::from_utf8_lossy(&out.stdout)))
        .flatten())
}

fn write(key: &str, value: &str) -> Result<()> {
    let status = Command::new("reg")
        .args([
            "add", key, "/v", "AutoRun", "/t", "REG_SZ", "/d", value, "/f",
        ])
        .status()
        .context("running reg add")?;
    if !status.success() {
        bail!("reg add failed with {status}");
    }
    Ok(())
}

/// Bring the AutoRun value up to date. Returns a short message for the status line.
pub fn integrate(key: &str, exe: Option<&Path>, config: &Path) -> String {
    let Some(exe) = exe else {
        return "cmd: fastfetch not found, skipped".into();
    };
    let Some(line) = line(exe, config) else {
        return "cmd: path has characters cmd cannot run, skipped".into();
    };
    match read(key).and_then(|current| match plan(current.as_deref(), &line) {
        Plan::Write(v) => write(key, &v).map(|()| "cmd prompt set up"),
        Plan::Current => Ok("cmd prompt up to date"),
        Plan::Foreign => Ok("cmd: AutoRun belongs to another tool, left unchanged"),
    }) {
        Ok(msg) => msg.into(),
        Err(e) => format!("cmd prompt not set up: {e:#}"),
    }
}

/// Delete the AutoRun value when it is ours. Returns a short message.
pub fn remove(key: &str) -> String {
    match read(key) {
        Ok(Some(v)) if v.contains(OWNER) => match Command::new("reg")
            .args(["delete", key, "/v", "AutoRun", "/f"])
            .status()
        {
            Ok(s) if s.success() => "cmd prompt removed".into(),
            Ok(s) => format!("cmd prompt not removed: reg exited with {s}"),
            Err(e) => format!("cmd prompt not removed: {e}"),
        },
        Ok(Some(_)) => "cmd: AutoRun belongs to another tool, left unchanged".into(),
        Ok(None) => "cmd prompt: nothing to remove".into(),
        Err(e) => format!("cmd prompt not removed: {e:#}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXE: &str = r"C:\WinGet\fastfetch.exe";
    const CFG: &str = r"C:\Users\x\.config\termfetch-studio\fastfetch.jsonc";

    #[test]
    fn line_checks_fastfetch_exists_and_picks_the_logo() {
        let l = line(Path::new(EXE), Path::new(CFG)).unwrap();
        assert!(l.starts_with(r#"if exist "C:\WinGet\fastfetch.exe" (if defined WT_SESSION"#));
        assert!(l.contains("--logo-type builtin"));
    }

    #[test]
    fn unsafe_paths_are_refused() {
        assert!(line(Path::new(r"C:\A&B\fastfetch.exe"), Path::new(CFG)).is_none());
        assert!(
            line(
                Path::new(EXE),
                Path::new(r"C:\100%\termfetch-studio\fastfetch.jsonc")
            )
            .is_none()
        );
    }

    #[test]
    fn only_our_line_is_ever_replaced() {
        let ours = line(Path::new(EXE), Path::new(CFG)).unwrap();
        assert_eq!(plan(None, &ours), Plan::Write(ours.clone()));
        assert_eq!(plan(Some(&ours), &ours), Plan::Current);
        let old = format!("{ours} old");
        assert_eq!(plan(Some(&old), &ours), Plan::Write(ours.clone()));
        assert_eq!(plan(Some("doskey ls=dir"), &ours), Plan::Foreign);
    }

    #[test]
    fn parses_reg_query_output() {
        let out = "\r\nHKEY_CURRENT_USER\\Software\\Microsoft\\Command Processor\r\n    AutoRun    REG_SZ    echo hi & dir\r\n\r\n";
        assert_eq!(parse_query(out).as_deref(), Some("echo hi & dir"));
        assert_eq!(
            parse_query("ERROR: The system was unable to find the specified registry key"),
            None
        );
    }

    #[test]
    #[cfg(windows)]
    fn integrate_sets_updates_and_leaves_foreign_values_alone() {
        let key = r"HKCU\Software\TermFetchStudioTest\Integrate";
        let _ = Command::new("reg")
            .args(["delete", r"HKCU\Software\TermFetchStudioTest", "/f"])
            .status();
        let exe = Path::new(EXE);
        let cfg = Path::new(CFG);
        assert_eq!(integrate(key, Some(exe), cfg), "cmd prompt set up");
        assert_eq!(integrate(key, Some(exe), cfg), "cmd prompt up to date");
        write(key, "doskey ls=dir").unwrap();
        assert!(integrate(key, Some(exe), cfg).contains("left unchanged"));
        assert_eq!(read(key).unwrap().as_deref(), Some("doskey ls=dir"));
        assert_eq!(
            integrate(key, None, cfg),
            "cmd: fastfetch not found, skipped"
        );
        let _ = Command::new("reg")
            .args(["delete", r"HKCU\Software\TermFetchStudioTest", "/f"])
            .status();
    }

    #[test]
    #[cfg(windows)]
    fn write_and_read_round_trip_on_a_scratch_key() {
        let key = r"HKCU\Software\TermFetchStudioRoundTrip\Scratch";
        let value = r#"if exist "C:\a b\fastfetch.exe" ("C:\a b\fastfetch.exe" -c "C:\cfg.jsonc")"#;
        write(key, value).unwrap();
        assert_eq!(read(key).unwrap().as_deref(), Some(value));
        let _ = Command::new("reg")
            .args(["delete", r"HKCU\Software\TermFetchStudioRoundTrip", "/f"])
            .status();
    }
}
