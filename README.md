<div align="center">

# TermFetch Studio

**An interactive terminal theme and fastfetch manager for Linux.**

Choose themes, organize system information and personalize your terminal with image or ASCII logos.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Application version](https://img.shields.io/badge/application-1.1.0-informational.svg)](CHANGELOG.md)
[![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)](#requirements)

[Quick start](#quick-start) · [User guide](docs/USER_GUIDE.md) · [Changelog](CHANGELOG.md) · [Release notes](docs/RELEASE_NOTES.md) · [Report a bug](https://github.com/sinansarikaya/termfetch-studio/issues)

<img src="menu.png" width="760" alt="TermFetch Studio interactive terminal menu">

</div>

## Why TermFetch Studio?

Terminal customization often means editing several configuration files. TermFetch Studio brings theme selection, fastfetch presets and logo configuration into one keyboard-driven interface.

Built and used daily by [Sinan Sarıkaya](https://sinansarikaya.dev).

## Features

- **Themes and previews:** choose built-in color schemes or create a custom theme.
- **Fastfetch presets:** select system-information modules, customize colors and reorder custom presets.
- **Image and ASCII logos:** personalize output to suit your terminal with persistent sizing (small, medium, large) and aspect-ratio preservation.
- **Live color editor:** draft and preview semantic group colors (OS, Desktop, Hardware, Network, Time) across all presets with 16 named RGB colors.
- **Stable menu rendering:** dedicated alternate-screen sessions and off-screen frame preparation for smooth terminal navigation.
- **Shell integration:** configure startup output for Bash, Zsh or Fish with automatic deduplication.
- **Distribution-aware installation:** tested installation and behavior are recorded by distribution, date and test type below.
- **Configuration tools:** preview your setup, create backups and restore saved settings.

<p align="center">
  <img src="full-info.png" width="760" alt="Fastfetch system information customized with TermFetch Studio">
</p>

## Version and project status

Current application version: **1.1.0**. Published release: [v1.1.0](https://github.com/sinansarikaya/termfetch-studio/releases/tag/v1.1.0).

See the [changelog](CHANGELOG.md) for dated history and [release notes](docs/RELEASE_NOTES.md) for detailed notes on each release.

## Requirements

- Linux and Bash to run the installer and application.
- fastfetch >= 2.42.0; the installer offers automatic dependency installation on supported distributions.
- A Nerd Font configured in your terminal for icon display. See [font setup](FONT_SETUP.md).
- Optional image tools such as `chafa` and `imagemagick`.

Bash, Zsh and Fish are shell-integration targets; the application itself runs in Bash. Colors and image rendering depend on the terminal. See the [user guide](docs/USER_GUIDE.md) for extended compatibility details.

## Compatibility and test evidence (x86_64)

**Docker tests validate distribution detection, dependencies, installation and basic
application behavior. They do not validate a desktop or graphical terminal.**
Results below belong to the recorded source snapshots/dates, not every release
of a distribution. Digests, fastfetch versions and rerun commands are in the linked reports.

| Distribution / version | Test type / date | Desktop / terminal | Result and evidence |
| --- | --- | --- | --- |
| Ubuntu 24.04 | Docker / 2026-10-09; UI follow-up 2026-10-22 | None | Passed both fastfetch initial states; [baseline](tests/REPORT.md), [120-check follow-up](tests/group-colors-evidence/README.md) |
| Debian 12 | Docker / 2026-10-09 | None | Passed both states with fastfetch 2.53.0; [report](tests/REPORT.md) |
| Debian 13 | Docker / 2026-10-09 | None | Passed with fastfetch 2.53.0; existing repository 2.40.4 was rejected without replacement; [report](tests/REPORT.md) |
| Fedora 43 | Docker / 2026-10-09 | None | Passed both states with fastfetch 2.68.1; [report](tests/REPORT.md) |
| Arch Linux rolling | Docker / 2026-10-09; UI follow-up 2026-10-22 | None | Passed both states; 2.69.0 baseline / 2.70.0 follow-up; [report](tests/group-colors-evidence/README.md) |
| openSUSE Tumbleweed 20261007 | Docker / 2026-10-09 | None | Passed both states with fastfetch 2.69.0; [report](tests/REPORT.md) |
| Ubuntu 24.04.5 | Real graphical VM / baseline report | GNOME 46 / Wayland; GNOME Terminal 3.52.0 + Kitty 0.32.2 | Recorded basic checks passed; [screenshots](tests/screenshots/README.md) |
| Debian 13.7 | Real graphical VM / baseline report | Plasma 6.3.6 / Wayland; Konsole 25.04.2 + Kitty 0.41.1 | Recorded basic checks passed; [report](tests/REPORT.md) |
| Fedora 43 | Real graphical VM / baseline report | GNOME 49.10 / Wayland; GNOME Terminal 3.56.3 + Kitty 0.43.1 | Recorded basic checks passed; [report](tests/REPORT.md) |
| Arch Linux rolling | Real graphical VM / baseline + 2026-10-22 UI follow-up | Hyprland 0.56.2 / Wayland; Kitty 0.49.2 | Recorded basic checks and ordered-color follow-up passed; [real screens](tests/ordered-colors-evidence/README.md) |
| openSUSE Tumbleweed 20261007 | Real graphical VM / baseline report | Plasma 6.7.5 / Wayland; Konsole 26.08.1 + Kitty 0.48.2 | Recorded checks passed after guest terminal packages were completed; [report](tests/REPORT.md) |
| Ubuntu 24.04.5 | Separate real X11 VM session / baseline report | GNOME 46 / X11; GNOME Terminal + Kitty | Recorded comparison passed; [screenshots](tests/screenshots/README.md) |
| CachyOS rolling | User-reported daily use | Niri / Wayland; Kitty (user screenshots) | Daily use reported; not an exhaustive automated validation |

The full baseline Docker run was **12 scenarios / 336 checks**. Later group-color
checks reran only Ubuntu and Arch (**4 scenarios / 120 checks**); the final ordered
preview follow-up recorded **26 PTY tests** and targeted Fish uninstall checks.
These are distinct runs: the complete matrix was not rerun for every UI change
or the 1.1.0 version-label update. Other desktops/terminals need a fresh UI regression
before those changes are claimed as validated there. See [remaining manual checks](tests/MANUAL_CHECKLIST.md).

Baseline VM checks covered menu navigation, Dracula/Nord colors, terminal reopen
persistence, sample Nerd Font icons, fastfetch, PNG or ASCII fallback, and Bash
startup. They do not prove every theme, glyph, image format, physical GPU, display
scale or monitor layout. Fish/Zsh startup was tested in Docker PTYs. Untested
versions, RHEL/CentOS/Leap and aarch64 are not listed as verified installations.

## Updating an existing checkout

If you have no local changes:

```bash
cd ~/termfetch-studio
git pull --ff-only
bash ./install.sh
```

If the checkout contains your edits, preserve them before updating. Reinstallation
updates the app and uninstaller; changing repository files alone does not update
installed files. Run as your normal user.

## Quick start

Review [install.sh](install.sh) before running it. Run the installer as your normal user; dependency installation may request sudo:

```bash
git clone https://github.com/sinansarikaya/termfetch-studio.git
cd termfetch-studio
bash install.sh
termfetch-studio
```

If the command is not found, run `~/.local/bin/termfetch-studio` and ensure `~/.local/bin` is in your shell's PATH.

**Navigation:** arrow keys to move, Enter to select, Q to go back.

## Commands

| Command | Purpose |
| --- | --- |
| `termfetch-studio` | Open the interactive menu |
| `termfetch-studio --preview` | Show the current setup |
| `termfetch-studio --version` | Show the application version |
| `termfetch-studio --help` | Show command help |
| `termfetch-studio --backup` | Create a configuration backup |
| `termfetch-studio --restore` | Select a backup to restore |
| `termfetch-studio uninstall` | Run the installed uninstaller |

Keep an independent copy of important configuration files when modifying system-wide settings.

## Configuration

| Location | Contents |
| --- | --- |
| `~/.local/bin` | Application commands |
| `~/.local/share/termfetch-studio` | Installed application files |
| `~/.config/termfetch-studio` | Settings, images and backups |

See the [user guide](docs/USER_GUIDE.md) for themes, presets, logos, demo mode and troubleshooting.

## Contributing and feedback

Bug reports and feature suggestions are welcome. Include:

- Linux distribution, terminal and shell versions.
- TermFetch Studio and fastfetch versions.
- Steps to reproduce, expected behavior and actual behavior.
- Relevant error output or terminal screenshots.

Theme contributions are welcome: include the theme file, a preview and the terminal used to test it.

## License and credits

[MIT](LICENSE). Built on [fastfetch](https://github.com/fastfetch-cli/fastfetch), with thanks to the terminal theme and Nerd Fonts communities.

Created by [Sinan Sarıkaya](https://github.com/sinansarikaya).
