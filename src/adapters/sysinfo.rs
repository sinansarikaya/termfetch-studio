//! Static host facts for the header bar: distribution and local clock.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct SysInfo {
    /// `NAME` from os-release (e.g. "Arch Linux", "CachyOS").
    pub distro: String,
    /// `ID` / `ID_LIKE` from os-release, lower-case.
    pub distro_ids: Vec<String>,
    /// Local offset from UTC in seconds, read once at startup.
    pub utc_offset: i64,
}

/// Value of `key` in an os-release file (quotes stripped).
fn os_release_value(text: &str, key: &str) -> Option<String> {
    text.lines()
        .filter_map(|l| l.split_once('='))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| v.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|v| !v.is_empty())
}

/// `+0300` / `-0530` → seconds.
fn parse_offset(s: &str) -> Option<i64> {
    let s = s.trim();
    let (sign, digits) = match s.as_bytes().first()? {
        b'+' => (1, &s[1..]),
        b'-' => (-1, &s[1..]),
        _ => (1, s),
    };
    if digits.len() != 4 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let h: i64 = digits[..2].parse().ok()?;
    let m: i64 = digits[2..].parse().ok()?;
    Some(sign * (h * 3600 + m * 60))
}

impl SysInfo {
    pub fn detect() -> Self {
        let release = ["/etc/os-release", "/usr/lib/os-release"].iter().find_map(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
        let distro = os_release_value(&release, "NAME").unwrap_or_else(|| std::env::consts::OS.to_string());
        let mut distro_ids: Vec<String> = os_release_value(&release, "ID").into_iter().collect();
        if let Some(like) = os_release_value(&release, "ID_LIKE") {
            distro_ids.extend(like.split_whitespace().map(str::to_string));
        }
        distro_ids.iter_mut().for_each(|s| s.make_ascii_lowercase());
        let utc_offset = Command::new("date")
            .arg("+%z")
            .output()
            .ok()
            .and_then(|o| parse_offset(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or(0);
        Self { distro, distro_ids, utc_offset }
    }

    /// Nerd Font glyph for the distribution.
    pub fn distro_icon(&self) -> &'static str {
        let is = |id: &str| self.distro_ids.iter().any(|d| d == id);
        if is("arch") || is("cachyos") || is("endeavouros") || is("manjaro") {
            "\u{f08c7}"
        } else if is("ubuntu") {
            "\u{f0548}"
        } else if is("debian") {
            "\u{f08da}"
        } else if is("fedora") {
            "\u{f08db}"
        } else if cfg!(target_os = "macos") {
            "\u{f0035}"
        } else {
            "\u{f033d}"
        }
    }

    /// Local wall-clock time as `HH:MM`.
    pub fn clock(&self) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        let secs = (now + self.utc_offset).rem_euclid(86_400);
        format!("{:02}:{:02}", secs / 3600, secs % 3600 / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_os_release_and_offsets() {
        let text = "NAME=\"CachyOS Linux\"\nID=cachyos\nID_LIKE=arch\n";
        assert_eq!(os_release_value(text, "NAME").as_deref(), Some("CachyOS Linux"));
        assert_eq!(os_release_value(text, "ID_LIKE").as_deref(), Some("arch"));
        assert_eq!(parse_offset("+0300\n"), Some(10_800));
        assert_eq!(parse_offset("-0530"), Some(-19_800));
        assert_eq!(parse_offset("garbage"), None);
        let info = SysInfo { distro: "CachyOS".into(), distro_ids: vec!["cachyos".into(), "arch".into()], utc_offset: 0 };
        assert_eq!(info.distro_icon(), "\u{f08c7}");
        assert_eq!(info.clock().len(), 5);
    }
}
