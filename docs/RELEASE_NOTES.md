# Release Notes

## TermFetch Studio v1.1.0

- **Tag:** `v1.1.0`
- **Release classification:** Stable release
- **Publication date:** 2026-10-22

TermFetch Studio brings terminal themes, fastfetch presets, and logo customization into an interactive Linux terminal interface.

Version 1.1.0 introduces live draft color editing with semantic groups, persistent image-logo sizing, flicker-free off-screen menu rendering, enhanced multi-distribution compatibility, Fish shell uninstallation hardening, and shell startup deduplication.

### Key Highlights in v1.1.0

- **Live Color & Semantic Group Customization:** Interactive draft color editor with real-time preview for OS, Desktop, Hardware, Network, and Time semantic groups across all 6 presets.
- **Extended RGB Palette:** 16 named RGB colors added alongside Theme and Default handling, with per-field cancel, save, and reset options.
- **Persistent Logo Sizing:** Small, medium, and large image-logo sizing with bounded aspect-preserving dimensions.
- **Flicker-Free Rendering:** Single alternate-screen session across interactive submenus and off-screen frame preparation for smooth terminal navigation.
- **Enhanced Multi-Distro Support:** Distribution family recognition via `ID` and `ID_LIKE` (including openSUSE Tumbleweed, Debian 12/13, Ubuntu 24.04, Arch Linux, and Fedora).
- **Clean Shell Integration:** Startup deduplicated across Bash, Zsh, and Fish; skips redundant managed calls if an external startup call already exists.
- **Safe Fish Uninstallation:** Safely strips only the exact legacy Fish alias (`termfetch-studio --preview`) with automatic config backup, preserving personal functions.
- **Comprehensive Quality Gates:** 26 PTY regression tests, 4 Docker scenario suites (120 checks), ShellCheck validation, and native Kitty VM rendering verification.

### Installation

Clone the repository at the release tag and run the installer:

```bash
git clone --branch v1.1.0 --depth 1 https://github.com/sinansarikaya/termfetch-studio.git
cd termfetch-studio
bash install.sh
```

Run the installer as your normal user. Dependency installation may request sudo if packages are missing.

---

## TermFetch Studio v1.0.0

- **Tag:** `v1.0.0`
- **Publication date:** 2026-10-09

Initial baseline release of TermFetch Studio.

### Included

- 25 built-in terminal themes, custom colors and theme previews.
- 6 fastfetch presets and custom module selection.
- Image and ASCII logo configuration.
- Bash, Zsh and Fish startup integration.
- Configuration preview, backup and restore commands.
