# TermFetch Studio v2 — Package Manager & Distribution Manifests

This directory contains the packaging manifests and configuration files for distributing TermFetch Studio across all major operating systems and package managers.

---

## Directory Structure

```
packaging/
├── aur/
│   ├── PKGBUILD             # Arch User Repository source build formula
│   ├── PKGBUILD-bin         # AUR prebuilt binary package (termfetch-studio-bin)
│   └── .SRCINFO             # Generated Arch package metadata
├── homebrew/
│   └── termfetch-studio.rb  # Homebrew tap formula for macOS & Linuxbrew
├── winget/
│   └── manifests/s/SinanSarikaya/TermFetchStudio/2.0.0/
│       ├── SinanSarikaya.TermFetchStudio.yaml
│       ├── SinanSarikaya.TermFetchStudio.installer.yaml
│       └── SinanSarikaya.TermFetchStudio.locale.en-US.yaml
├── debian/
│   └── control              # Debian/Ubuntu DEB control file
└── rpm/
    └── termfetch-studio.spec # Fedora/RHEL/openSUSE RPM spec file
```

---

## 1. Arch Linux (AUR)

### Package Variants
1. **`termfetch-studio`**: Builds from source using `cargo build --release --locked`. Ideal for Arch users who prefer compiling from source.
2. **`termfetch-studio-bin`**: Downloads the prebuilt release tarball from GitHub Releases. Installs instantaneously.

### Installation Commands for Users
```bash
# Using yay
yay -S termfetch-studio-bin

# Using paru
paru -S termfetch-studio-bin
```

### Dependencies
- **Runtime:** `fastfetch`, `gcc-libs`, `glibc`
- **Optional:** `kitty` (Kitty graphics), `chafa` (half-block fallback), `ttf-font-nerd` (icons)

---

## 2. macOS (Homebrew Tap)

### Setting up the Tap
Create a repository named `homebrew-tap` under `sinansarikaya`:
`https://github.com/sinansarikaya/homebrew-tap`
Place `termfetch-studio.rb` in `Formula/termfetch-studio.rb`.

### Installation Commands for Users
```bash
# Tap and install
brew tap sinansarikaya/tap
brew install termfetch-studio
```

### Features
- Native support for both Apple Silicon (`aarch64-apple-darwin`) and Intel (`x86_64-apple-darwin`).
- Automatic `depends_on "fastfetch"`.
- Installs bundled assets to `$(brew --prefix)/share/termfetch-studio/assets`.

---

## 3. Windows (WinGet)

### Manifest Submission
Manifests adhere to WinGet schema v1.6.0.
To submit to the official Windows Package Manager Community Repository:
1. Fork `https://github.com/microsoft/winget-pkgs`.
2. Copy `packaging/winget/manifests/s/SinanSarikaya/TermFetchStudio/2.0.0/` into `manifests/s/SinanSarikaya/TermFetchStudio/2.0.0/`.
3. Open a Pull Request.

### Installation Command for Users
```powershell
winget install SinanSarikaya.TermFetchStudio
```

### Features
- Portable zip extraction; automatically adds `termfetch-studio` alias to user PATH.
- Declares package dependency on `Fastfetch-cli.Fastfetch`.

---

## 4. Debian / Ubuntu (DEB)

Generated automatically by GitHub Actions release workflow or locally:
```bash
# User installation
sudo apt install ./termfetch-studio_2.0.0_amd64.deb
```
- Dependencies: `fastfetch`
- Recommends: `chafa`, `kitty`

---

## 5. Upgrade and Uninstall Behavior

- **Upgrade:**
  - Upgrading replaces `/usr/bin/termfetch-studio` and `/usr/share/termfetch-studio/assets`.
  - User configuration in `~/.config/termfetch-studio/` and `~/.config/fastfetch/config.jsonc` (or `%APPDATA%\fastfetch\config.jsonc`) is **always preserved**.
- **Uninstall:**
  - Removes application binaries and bundled assets.
  - User themes, custom backups, and settings are preserved in `~/.config/termfetch-studio/` to avoid data loss.
