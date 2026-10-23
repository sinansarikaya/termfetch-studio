# Release Notes

## TermFetch Studio v1.1.0

- **Tag:** `v1.1.0`
- **Release classification:** Stable release
- **Publication date:** 2026-10-22

TermFetch Studio brings terminal themes, fastfetch presets, and logo customization into an interactive Linux terminal interface.

Version 1.1.0 introduces live draft color editing with semantic groups, persistent image-logo sizing, off-screen menu rendering improvements, enhanced multi-distribution compatibility, Fish shell uninstallation hardening, and shell startup deduplication.

### Key Highlights in v1.1.0

- **Live Color & Semantic Group Customization:** Interactive draft color editor with real-time preview for OS, Desktop, Hardware, Network, and Time semantic groups across all 6 presets.
- **Extended RGB Palette:** 16 named RGB colors added alongside Theme and Default handling, with per-field cancel, save, and reset options.
- **Persistent Logo Sizing:** Small, medium, and large image-logo sizing with bounded aspect-preserving dimensions.
- **Menu Rendering Improvements:** Single alternate-screen session across interactive submenus and off-screen frame preparation for smooth terminal navigation.
- **Distribution Family Detection:** `ID` and `ID_LIKE` detection is distinct from validation of a particular distribution/version. See the dated matrix below.
- **Clean Shell Integration:** Startup deduplicated across Bash, Zsh, and Fish; skips redundant managed calls if an external startup call already exists.
- **Safe Fish Uninstallation:** Safely strips only the exact legacy Fish alias (`termfetch-studio --preview`) with automatic config backup, preserving personal functions.
- **Recorded Quality Gates:** separate dated runs include 26 PTY tests, a 4-scenario Ubuntu/Arch follow-up (120 checks), generated Bash/ShellCheck and real Kitty VM checks. These were not all rerun after the version-label update.

### Compatibility evidence and limitations

The [README matrix](../README.md#compatibility-and-test-evidence-x86_64) distinguishes:

- Baseline distribution tests: six x86_64 Docker images, two initial fastfetch states, 12 scenarios / 336 checks ([report](../tests/REPORT.md)). Docker does not test a desktop.
- Later UI/group-color tests: Ubuntu/Arch only, four scenarios / 120 checks ([report](../tests/group-colors-evidence/README.md)).
- Final ordered-preview and Fish uninstall follow-up: 26 PTY tests and targeted uninstall checks; actual Arch/Hyprland/Wayland/Kitty screenshots ([report](../tests/ordered-colors-evidence/README.md)).
- Baseline real graphical VM matrix: GNOME/KDE/Hyprland on Wayland plus a separate GNOME X11 comparison ([screenshots](../tests/screenshots/README.md)). These baseline checks do not validate every later UI change on every terminal.

Check [manual limits](../tests/MANUAL_CHECKLIST.md) for physical GPU/monitor/font coverage, other terminals and untested versions/architectures. Existing fastfetch < 2.42.0 is rejected without silently replacing it. An already-open Fish needs `functions --erase fastfetch` after uninstall if its old alias is still loaded.

The release tag remains the original code/documentation snapshot. Maintained documentation corrections dated 2026-10-23 are on the default branch; no release tags are moved.

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
