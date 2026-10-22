# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-10-22

### Added
- **Live Color & Semantic Group Customization**: Interactive draft color editor with real-time live preview for OS, Desktop, Hardware, Network, and Time groups across all 6 presets.
- **Extended RGB Palette**: 16 named RGB colors added alongside Theme and Default handling, with per-field cancel, save, and reset options.
- **Persistent Logo Sizing**: Small, medium, and large image-logo sizing with bounded aspect-preserving dimensions.
- **Flicker-Free Rendering & Alternate Screen**: Retain one alternate-screen session across interactive submenus and prepare complete frames off-screen before display.
- **Enhanced Distro Detection**: Distribution family recognition via `ID` and `ID_LIKE` (openSUSE Tumbleweed, Debian 12/13, Ubuntu 24.04, Arch Linux, Fedora).
- **Test Suite**: 26 PTY regression tests, 4 Docker scenario suites (120 checks), ShellCheck validation, and native Kitty VM rendering evidence.

### Changed
- Consolidated color editing under the main *Icon & Text Colors* page, removing duplicate advanced settings.
- Kitty image preview protocol modernized to upload without displaying, swapping and cleaning only owned image IDs without full-screen clears on arrow navigation.
- Shell startup integration deduplicated across Bash, Zsh, and Fish; skips redundant managed fastfetch calls if an external startup call already exists.
- In small terminal viewports, color preview groups fall back to a contiguous, non-wrapping view with fixed semantic ordering.

### Fixed
- Fixed uninstallation for Fish shell: safely removes only the exact legacy Fish alias (`termfetch-studio --preview`) with automatic config backup, leaving personal functions and custom aliases intact.
- Corrected theme Inherit color preview to accurately match selected preset values.
- Quieted non-interactive shell and SSH/SCP sessions by ensuring startup traps and menus run strictly in interactive TTY sessions.
- Pinned and verified upstream fastfetch package on Debian/Ubuntu with SHA-256 validation, requiring fastfetch >= 2.42.0 for RGB configuration.
- Stabilized terminal menu input across resize events and deferred signal cleanup until user input reads complete.

## [1.0.0] - 2026-10-09

### Added
- Initial release of TermFetch Studio.
- 25 built-in themes and 8 popular themes (Cyberpunk, Matrix, Naruto, etc.).
- 6 fastfetch presets (Minimal, Classic, Modern, Hardware, Neofetch, Gaming).
- Image and ASCII logo support with auto-conversion and 16 built-in ASCII logos.
- Shell startup integration for Bash, Zsh, and Fish.
- Backup and restore manager with automatic backups before modifications.
