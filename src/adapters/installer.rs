//! Windows install and uninstall. The PowerShell script is embedded in the exe, so the
//! exe alone can install itself; `scripts/install-windows.ps1` stays the single source.

use anyhow::{Context, Result};
use std::process::Command;

const SCRIPT: &str = include_str!("../../scripts/install-windows.ps1");

/// Run the embedded script with `args` and return its exit code.
pub fn run_script(args: &[&str]) -> Result<i32> {
    let path = std::env::temp_dir().join(format!("termfetch-install-{}.ps1", std::process::id()));
    std::fs::write(&path, SCRIPT).with_context(|| format!("writing {}", path.display()))?;
    let status = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&path)
        .args(args)
        .status();
    let _ = std::fs::remove_file(&path);
    let status = status.context("running powershell")?;
    Ok(status.code().unwrap_or(1))
}
