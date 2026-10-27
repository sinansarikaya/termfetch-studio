# TermFetch Studio v2

[![CI Matrix](https://github.com/sinansarikaya/termfetch-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/sinansarikaya/termfetch-studio/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Platforms](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-brightgreen.svg)](#installation)

Cross-platform terminal customization and Fastfetch theme studio — built in Rust with Ratatui and Crossterm. Browse, customize, and apply Fastfetch themes, presets, modules, and logos with a real-time, cell-accurate live preview.

---

## Installation

The verified public release is [v2.0.0](https://github.com/sinansarikaya/termfetch-studio/releases/tag/v2.0.0). This checkout is source version 2.1.1 and includes newer installer and branding work that is not in those downloads. The rest of this README describes current source unless a published release is named explicitly.

Keep the extracted `assets` directory beside the executable. Install Fastfetch separately for native output.

### 🪟 Windows

- **Published v2.0.0:** extract [the Windows x64 ZIP](https://github.com/sinansarikaya/termfetch-studio/releases/download/v2.0.0/termfetch-studio-v2.0.0-x86_64-pc-windows-msvc.zip), keep its bundled assets and run `termfetch-studio.exe`. This release does not include the newer `install.cmd` installer.
- **Current source:** after building, `termfetch-studio install` installs for the current user, adds the command to PATH and creates Start Menu shortcuts. It can install missing Fastfetch and Windows Terminal dependencies with winget. This installer is not yet part of a verified published app release.
- **Scoop** and **WinGet** channels are pending publication; see [`packaging/`](./packaging).

### 🍎 macOS

- **DMG (current source):** open `termfetch-studio-v<version>-macos-universal.dmg`, drag **TermFetch Studio** into **Applications** and open it from Launchpad or Spotlight. Fastfetch is bundled inside the app, so there is nothing else to install. The first start sets up the Fastfetch config, runs it in every new shell and puts `termfetch` on PATH. The app opens in the terminal you use most (iTerm2, Ghostty, kitty, WezTerm or Terminal.app) and adapts to it: images use the terminal's own protocol, and Terminal.app, which has none, gets block-drawn logos, 256 colours and a "TermFetch" profile with the theme colours. Terminal.app logos are drawn with the bundled TermFetch Blocks font, eight sub-cells per character. Until the app is notarized, macOS asks once: choose **System Settings → Privacy & Security → Open Anyway**. Build the DMG yourself with `scripts/build-macos-dmg.sh` (universal Apple Silicon + Intel). To uninstall, run `termfetch uninstall`, then drag the app to the Trash.
- **Homebrew:** the TermFetch tap/formula is pending publication; no TermFetch install command is available here yet.
- **Published v2.0.0:** extract [the universal macOS archive](https://github.com/sinansarikaya/termfetch-studio/releases/download/v2.0.0/termfetch-studio-v2.0.0-apple-darwin-universal.tar.gz), put `termfetch-studio` on PATH, and install the separate Fastfetch dependency with `brew install fastfetch`.

### 🐧 Linux

- **Arch Linux (AUR):** source and binary recipes are being prepared; registry publication is pending. Use the archives or build from source for now.
- **Debian / Ubuntu / Fedora / openSUSE:** `.deb` and `.rpm` packages are on [the v2.0.0 release page](https://github.com/sinansarikaya/termfetch-studio/releases/tag/v2.0.0).
- **Published v2.0.0 archives:** use `termfetch-studio-v2.0.0-x86_64-unknown-linux-musl.tar.gz` or the glibc archive from that release page. Put the executable on PATH and install Fastfetch from your package manager.

### 🛠️ Build from source

```bash
git clone https://github.com/sinansarikaya/termfetch-studio.git
cd termfetch-studio
cargo build --release
./target/release/termfetch-studio
```

In current source, first start sets up Fastfetch configs and platform shell/terminal integration. Run `termfetch` after installation; a menu launcher is added on Linux/macOS. These newer integration details should not be assumed to exist in the older v2.0.0 archives.

Assets are discovered automatically at runtime (first match wins): `$TERMFETCH_STUDIO_ASSETS`,
`<exe dir>/assets`, `<exe dir>/../share/termfetch-studio/assets`,
`~/.local/share/termfetch-studio/assets`, `./assets`, then the source checkout.

### 🗑️ Uninstall
- `termfetch uninstall` removes what TermFetch added to your shell startup, launcher and (on Windows) cmd prompt, Windows Terminal profile and colors. Your original Windows Terminal settings are kept in `settings.json.termfetch-backup`.
- On Windows, `termfetch uninstall` also removes the install folder, the PATH entry and the Start Menu shortcuts. On Linux and macOS, remove the program with the package manager you used.

---

## TUI

```bash
termfetch            # uses the saved demo setting
termfetch --no-demo  # start with real specs
```

Layout (the "Studio" design): a header bar with brand, fastfetch status,
distribution and local clock; a sidebar with the navigation (Dashboard, Themes,
Presets, Logos, Modules, Settings - Backup & Restore and Doctor live under
Settings) and context cards for the current theme, preset and config path; the
center stage; and on the right the **Live Preview** with a **Preview Options**
card (theme / preset / logo / output dropdowns and the *Apply Theme to System*
button). The Dashboard shows quick actions, recently applied themes with
procedural thumbnails, a system overview and tools. The chrome uses a fixed
navy/indigo palette by default; *Settings → Studio chrome* switches it to follow
the active theme. Every label is WCAG-checked on every surface in both modes.

The preview, painted in the theme background, reproduces fastfetch's layout
exactly (logo padding, `keyWidth`, separator, resolved colours); the *Output*
option switches it to the generated JSONC. Image logos are decoded once by a
background worker pool, so scrolling stays well above 60 FPS.

In terminals with the Kitty graphics protocol (kitty, Ghostty) the preview shows
image logos at full resolution: each logo is resized to the preview's exact
pixel box and PNG-encoded on a background thread, uploaded once, and then drawn
through Unicode placeholder cells, so switching logos only swaps an image id.
The whole gallery is pre-uploaded in the background; half-blocks are shown
until an upload lands and are used everywhere else (including tmux/screen).
Force a mode with `TERMFETCH_STUDIO_GRAPHICS=kitty` or `=halfblock`.

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | next / previous section: menu → center stage → preview options |
| `↑`/`↓`/`←`/`→`, `hjkl`, `PgUp`/`PgDn`, `g`/`G` | move (preview updates instantly) |
| `Enter` | select: open a quick action / tool, apply a card, open a dropdown |
| `a` / `A` | apply theme to system: save settings and write both fastfetch configs |
| `1`–`8` | jump to a screen (on the Dashboard stage `1`–`4` run the quick actions) |
| `p` | run the real fastfetch with the current, unsaved selection |
| `s` | cycle image logo size (small / medium / large) |
| `o` | cycle the preview output (terminal preview / generated config) |
| `Space`, `J`/`K` | Modules: toggle / reorder modules |
| `←`/`→` | Settings: cycle colours and preferences · Preview Options: cycle values |
| `x` | Backup: delete selected backup · `r` Doctor: re-run checks |
| `?` | keyboard shortcuts overlay · mouse click and wheel supported everywhere |
| `q` / `Esc` | quit; unsaved changes are saved and applied to both fastfetch configs first |

## CLI

`termfetch` is the short command; `termfetch-studio` keeps working as the full name. Packages install both, and a plain binary install adds the `termfetch` link next to itself on first start.

```bash
termfetch apply --theme nord --preset developer --logo pikachu --logo-size large
termfetch preview --theme dracula --preset minimal      # ANSI preview to stdout
termfetch demo on|off|toggle|status
termfetch list themes|presets|logos|modules|colors
termfetch backup create|list|restore <name>|delete <name>
termfetch run          # fastfetch with the studio config
termfetch settings     # open the Settings screen
termfetch setup        # write configs and the shell/system integration again
termfetch install      # Windows: install for this user (same as install.cmd)
termfetch uninstall    # remove what TermFetch added to this system
termfetch doctor       # also: --doctor
```

## Files

| Path | Purpose |
|---|---|
| `~/.config/termfetch-studio/settings.json` | saved choices |
| `~/.config/termfetch-studio/fastfetch.jsonc` | generated config (original installer path) |
| `~/.config/fastfetch/config.jsonc` | generated config (fastfetch default) |
| `~/.config/termfetch-studio/backups/` | backups; a hand-written fastfetch config is preserved here before the first overwrite |
| `~/.config/termfetch-studio/custom.png` | custom image logo |

Both configs are produced from typed `serde_json` structures and written
atomically (temp file + fsync + rename). `$XDG_CONFIG_HOME` is respected.

## Image sizing

The logo box is computed from the image header's pixel size and the terminal's
real cell size (`TIOCGWINSZ` via `crossterm::terminal::window_size`, falling
back to 1:2 cells), bounded by the size preset (20×14 / 36×22 / 52×32), a third
of the terminal width and its height − 4 — the same algorithm as the original
installer, including its "largest rectangle within 5% of the true ratio" search.
The preview uses the identical box, shrinking it with the same aspect ratio only
if the pane is smaller.

## Architecture

```
src/
├── main.rs              # CLI dispatch, terminal lifecycle, event loop, live fastfetch
├── paths.rs             # XDG/exe-relative path + asset discovery
├── cli/                 # clap definitions
├── app/                 # AppState, Action, reducer (+ effects)
├── domain/              # themes, presets/modules, logos, colours, settings, style resolution
├── imaging/             # fit_image_cells, half-block raster, Kitty graphics, decode pool
├── adapters/            # fastfetch config/runner, backup, doctor, atomic fs
└── tui/                 # layout, sidebar, preview, screens
```

## Website

The application website is maintained in [`website/`](./website), alongside the
Rust app so its interactive demo and shared brand stay in sync.

```bash
npm ci --prefix website
npm run dev --prefix website
npm run build --prefix website
npm run lint --prefix website
npm run check --prefix website  # headed browser regression after building
```

See the [website guide](./website/README.md), [shared brand](./docs/brand.md) and
[Docker / Dokploy deployment](./docs/website-deployment.md) for setup and maintenance.

## License

MIT — Sinan Sarıkaya
