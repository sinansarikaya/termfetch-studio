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
- **Flicker-free rendering:** dedicated alternate-screen sessions and off-screen frame preparation for smooth terminal navigation.
- **Shell integration:** configure startup output for Bash, Zsh or Fish with automatic deduplication.
- **Multi-distro support:** verified compatibility across Arch, Debian 12/13, Ubuntu 24.04, Fedora, and openSUSE Tumbleweed.
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
