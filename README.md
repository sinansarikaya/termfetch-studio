# TermFetch Studio v2

[![CI Matrix](https://github.com/sinansarikaya/termfetch-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/sinansarikaya/termfetch-studio/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Platforms](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-brightgreen.svg)](#installation)

Cross-platform terminal customization and Fastfetch theme studio — built in Rust with Ratatui and Crossterm. Browse, customize, and apply Fastfetch themes, presets, modules, and logos with a real-time, cell-accurate live preview.

---

## Installation

### 🛠️ Build from Source (Recommended for Development)
```bash
git clone https://github.com/sinansarikaya/termfetch-studio.git
cd termfetch-studio
cargo build --release
./target/release/termfetch-studio
```

Assets are discovered automatically at runtime (first match wins): `$TERMFETCH_STUDIO_ASSETS`,
`<exe dir>/assets`, `<exe dir>/../share/termfetch-studio/assets`,
`~/.local/share/termfetch-studio/assets`, `./assets`, then the source checkout.

### ⚡ Direct Binary Download (GitHub Releases)
Prebuilt standalone binaries and universal musl static archives are published on [GitHub Releases](https://github.com/sinansarikaya/termfetch-studio/releases):
- **Linux (Musl Static):** `termfetch-studio-v2.0.0-x86_64-unknown-linux-musl.tar.gz` (Zero runtime dependencies, runs on any Linux distribution)
- **Linux (Glibc):** `termfetch-studio-v2.0.0-x86_64-unknown-linux-gnu.tar.gz`
- **macOS (Universal):** `termfetch-studio-v2.0.0-apple-darwin-universal.tar.gz` (Native on Apple Silicon M-series and Intel x86_64)
- **Windows (x64):** `termfetch-studio-v2.0.0-x86_64-pc-windows-msvc.zip`

---

### 📦 Package Managers *(Active upon official v2.0.0 release)*

> [!NOTE]
> Package manager formulas, manifests, and PKGBUILDs are prepared in [`packaging/`](./packaging) and will be submitted to the respective repositories immediately upon official v2.0.0 release publication.

#### 🚀 Arch Linux (AUR) `[Upcoming / Release Gate]`
```bash
# Precompiled binary (recommended)
yay -S termfetch-studio-bin
# or with paru
paru -S termfetch-studio-bin

# Build from source
yay -S termfetch-studio
```

#### 🍎 macOS (Homebrew) `[Upcoming / Release Gate]`
```bash
brew tap sinansarikaya/tap
brew install termfetch-studio
```

#### 🪟 Windows (WinGet) `[Upcoming / Release Gate]`
```powershell
winget install SinanSarikaya.TermFetchStudio
```

#### 🐧 Debian / Ubuntu (.deb) `[Upcoming / Release Gate]`
```bash
# Download and install the verified .deb package from GitHub Releases
curl -sLO https://github.com/sinansarikaya/termfetch-studio/releases/download/v2.0.0/termfetch-studio_2.0.0_amd64.deb
sudo apt install ./termfetch-studio_2.0.0_amd64.deb
```

#### 🎩 Fedora / openSUSE (.rpm) `[Upcoming / Release Gate]`
```bash
# Download and install the verified .rpm package from GitHub Releases
curl -sLO https://github.com/sinansarikaya/termfetch-studio/releases/download/v2.0.0/termfetch-studio-2.0.0-1.x86_64.rpm
sudo dnf install ./termfetch-studio-2.0.0-1.x86_64.rpm  # Fedora
# or on openSUSE:
# sudo zypper install ./termfetch-studio-2.0.0-1.x86_64.rpm
```

## TUI

```bash
termfetch-studio            # uses the saved demo setting
termfetch-studio --no-demo  # start with real specs
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
| `d` | toggle privacy demo mode (persisted, config regenerated) |
| `s` | cycle image logo size (small / medium / large) |
| `o` | cycle the preview output (terminal preview / generated config) |
| `Space`, `J`/`K` | Modules: toggle / reorder modules |
| `←`/`→` | Settings: cycle colours and preferences · Preview Options: cycle values |
| `x` | Backup: delete selected backup · `r` Doctor: re-run checks |
| `?` | keyboard shortcuts overlay · mouse click and wheel supported everywhere |
| `q` / `Esc` | quit; unsaved changes are saved and applied to both fastfetch configs first |

## CLI

```bash
termfetch-studio apply --theme nord --preset developer --logo pikachu --logo-size large
termfetch-studio preview --theme dracula --preset minimal      # ANSI preview to stdout
termfetch-studio demo on|off|toggle|status
termfetch-studio list themes|presets|logos|modules|colors
termfetch-studio backup create|list|restore <name>|delete <name>
termfetch-studio run          # fastfetch with the studio config
termfetch-studio doctor       # also: --doctor
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

## License

MIT — Sinan Sarıkaya
