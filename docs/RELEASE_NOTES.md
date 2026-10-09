# Prepared release notes

**Status: prepared, not published.** No GitHub Release or tag has been created.

## Proposed release

- **Title:** TermFetch Studio 1.0.0 — existing application baseline
- **Tag:** `v1.0.0` (not yet created)
- **Target commit:** `8f26432b3563a0eed11b6316385c2aa880ef8489`
- **Release classification:** prerelease while known bugs and compatibility are being assessed.
- **Publication date:** assign when published; do not backdate to the commit date.

The target captures the latest existing application code. Documentation added afterward remains available on main.

---

## Release body

TermFetch Studio brings terminal themes, fastfetch presets and logo customization into an interactive Linux terminal interface.

This release packages the existing application baseline, which reports version **1.0.0**, including the untagged repository updates from October and December 2025. It does not introduce new bug fixes.

### Included

- Built-in terminal themes, custom colors and theme previews.
- fastfetch presets and custom module selection.
- Image and ASCII logo configuration.
- Bash, Zsh and Fish startup integration.
- Configuration preview, backup and restore commands.

### Install

Clone the repository at the release tag, inspect the installer and run it as your normal user:

```bash
git clone --branch v1.0.0 --depth 1 https://github.com/sinansarikaya/termfetch-studio.git
cd termfetch-studio
bash install.sh
```

**The tag command above will work only after this proposed release is published.** For current installation, use the instructions in the main-branch README.

### Requirements and limitations

Linux and Bash are required. Icon display needs an appropriately configured Nerd Font; image and color support vary by terminal. The installer may request sudo to install dependencies.

Known bugs are under review. Keep an independent copy of important settings before installation or restore operations. No new cross-terminal or distribution test matrix is claimed for this baseline.

### Feedback

Report bugs with your distribution, terminal, shell, fastfetch version, reproduction steps and relevant error output.

[Changelog](https://github.com/sinansarikaya/termfetch-studio/blob/main/CHANGELOG.md) · [User guide](https://github.com/sinansarikaya/termfetch-studio/blob/main/docs/USER_GUIDE.md) · [Issues](https://github.com/sinansarikaya/termfetch-studio/issues)

---

## Publication checklist

1. Confirm the target commit and the intended prerelease classification.
2. Create `v1.0.0` at the target commit and a GitHub prerelease using the body above.
3. Confirm generated source archive links and tagged installation instructions.
4. Update README release status and link to the published release.
5. Record the publication date in the changelog without changing historical commit dates.
