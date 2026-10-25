//! Installing fastfetch with the package manager of the current system.

use std::process::{Command, Stdio};

/// One package-manager command that installs fastfetch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub program: &'static str,
    pub args: &'static [&'static str],
}

impl Plan {
    /// The command as the user would type it.
    pub fn command_line(&self) -> String {
        std::iter::once(self.program)
            .chain(self.args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    MacOs,
    Linux,
    Other,
}

pub fn current_os() -> Os {
    if cfg!(windows) {
        Os::Windows
    } else if cfg!(target_os = "macos") {
        Os::MacOs
    } else if cfg!(target_os = "linux") {
        Os::Linux
    } else {
        Os::Other
    }
}

/// Install command for `os`. `has` tells whether a program is available;
/// `distro_ids` are the lower-case `ID` and `ID_LIKE` words of os-release.
pub fn plan(os: Os, has: impl Fn(&str) -> bool, distro_ids: &[String]) -> Option<Plan> {
    match os {
        Os::Windows if has("winget") => Some(Plan {
            program: "winget",
            args: &[
                "install",
                "--id",
                "Fastfetch-cli.Fastfetch",
                "--exact",
                "--accept-source-agreements",
                "--accept-package-agreements",
            ],
        }),
        Os::Windows if has("scoop") => Some(Plan {
            program: "scoop",
            args: &["install", "fastfetch"],
        }),
        Os::MacOs if has("brew") => Some(Plan {
            program: "brew",
            args: &["install", "fastfetch"],
        }),
        Os::Linux => linux_plan(distro_ids),
        _ => None,
    }
}

fn linux_plan(ids: &[String]) -> Option<Plan> {
    let is = |names: &[&str]| ids.iter().any(|id| names.contains(&id.as_str()));
    let args: &'static [&'static str] = if is(&["arch", "cachyos", "endeavouros", "manjaro"]) {
        &["pacman", "-S", "--needed", "--noconfirm", "fastfetch"]
    } else if is(&["debian", "ubuntu", "linuxmint", "pop"]) {
        &["apt-get", "install", "-y", "fastfetch"]
    } else if is(&["fedora", "rhel", "centos", "rocky", "almalinux"]) {
        &["dnf", "install", "-y", "fastfetch"]
    } else if is(&[
        "suse",
        "opensuse",
        "opensuse-tumbleweed",
        "opensuse-leap",
        "sles",
    ]) {
        &["zypper", "--non-interactive", "install", "fastfetch"]
    } else if is(&["alpine"]) {
        &["apk", "add", "fastfetch"]
    } else if is(&["void"]) {
        &["xbps-install", "-Sy", "fastfetch"]
    } else {
        return None;
    };
    Some(Plan {
        program: "sudo",
        args,
    })
}

/// Whether `cmd` starts; a missing program simply fails to spawn.
pub fn on_path(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// The install command for this machine, or `None` when no supported manager is found.
pub fn plan_for_this_system(distro_ids: &[String]) -> Option<Plan> {
    plan(current_os(), on_path, distro_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn windows_prefers_winget_then_scoop() {
        let both = plan(Os::Windows, |_| true, &[]).unwrap();
        assert_eq!(both.program, "winget");
        assert!(both.command_line().contains("Fastfetch-cli.Fastfetch"));
        let scoop = plan(Os::Windows, |c| c == "scoop", &[]).unwrap();
        assert_eq!(scoop.command_line(), "scoop install fastfetch");
        assert_eq!(plan(Os::Windows, |_| false, &[]), None);
    }

    #[test]
    fn macos_uses_brew_when_present() {
        let p = plan(Os::MacOs, |c| c == "brew", &[]).unwrap();
        assert_eq!(p.command_line(), "brew install fastfetch");
        assert_eq!(plan(Os::MacOs, |_| false, &[]), None);
    }

    #[test]
    fn linux_maps_distro_families_to_their_package_manager() {
        let cases = [
            (&["cachyos", "arch"][..], "pacman"),
            (&["ubuntu", "debian"][..], "apt-get"),
            (&["fedora"][..], "dnf"),
            (&["opensuse-tumbleweed", "suse"][..], "zypper"),
            (&["alpine"][..], "apk"),
        ];
        for (distro, manager) in cases {
            let p = plan(Os::Linux, |_| true, &ids(distro)).unwrap();
            assert_eq!(p.program, "sudo");
            assert_eq!(p.args[0], manager, "{distro:?}");
        }
        assert_eq!(plan(Os::Linux, |_| true, &ids(&["nixos"])), None);
    }
}
