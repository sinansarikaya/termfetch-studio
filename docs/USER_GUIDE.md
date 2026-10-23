# User guide

[Back to README](../README.md)

Guidance for **TermFetch Studio 1.1.0**. See the [dated compatibility table](../README.md#compatibility-and-test-evidence-x86_64) for tested environments and [remaining manual checks](../tests/MANUAL_CHECKLIST.md) for limits. Docker results are not desktop validation.

## 🌟 Features

### 🎨 **Rich Theme Collection**
- **25 Built-in Themes**: Dark, Light, Colorful, Pastel, and Popular Culture options
- **8 Popular Themes**: Matrix 💚, Cyberpunk 🌆, Naruto 🍜, Pokemon ⚡, Spider-Man 🕷️, Doom 👹, Valorant 🔥, Minecraft ⛏️
- **3 Pastel Themes**: Sakura 🌸, Lavender Dream 💜, Candy 🍬
- **Custom Theme Creator**: Design your own color schemes
- **Persistent Colors**: Theme colors saved across terminal sessions
- **Live Preview**: See changes before applying

### 📊 **Fastfetch Integration**
- **5 Preset Configurations**: Full, Minimal, Focused, Developer, Gaming
- **🧩 Custom Preset Creator**: Build your own module combinations
- **🔄 Module Reordering**: Drag and drop style reordering with arrow keys
- **🎨 Advanced Color Customization**: Independent icon and text color control
- **🎯 Smart Color System**: Theme-based colors with custom override options
- **Clean Icon-Based Design**: Simple, elegant design with Nerd Font icons
- **Color-Coded Modules**: Each category has its own color for better readability
- **Tree-Style Layout**: Professional ├ └ tree characters for visual hierarchy
- **Beautiful Icons**: Every module has a matching Nerd Font icon

### 🖼️ **Advanced Logo System**
- **Multiple Logo Types**: Auto-detect distribution, custom images, ASCII art
- **Image Support**: Recorded native PNG checks in Kitty and Konsole; other terminal protocols require manual validation.
- **Format Conversion**: Auto-convert PNG, JPG, JPEG, GIF, BMP to optimal format with transparency preservation
- **Smart Resizing**: Automatically resize large images for better performance
- **ASCII Fallback**: Automatic conversion for unsupported terminals
- **16 Built-in ASCII Logos**:
  - Classics: Tux 🐧, Pikachu ⚡, Arch 📐
  - Cute: Cat 🐱, Neko 🎌, Bunny 🐰, Heart 💖, Sakura 🌸
  - Popular: Naruto 🍜, Pokeball ⚾, Spider 🕷️, Dragon 🐉, Creeper 💚, Star ⭐, Rocket 🚀

### 💾 **Backup & Restore**
- **Automatic Backups**: Before every change
- **Manual Backups**: Create anytime
- **Quick Restore**: Revert to any previous configuration
- **History Management**: Keep only recent backups

### ⚙️ **Smart Features**
- **Auto-Detection**: Identifies your terminal and capabilities
- **Setup Wizard**: Quick start for first-time users
- **Preview Mode**: Test configurations before applying
- **Shell Integration**: Bash, Zsh, Fish support
- **🛡️ Ubuntu Safety**: Automatic image logo blocking for unsupported terminals
- **🎭 Demo Mode**: Display fake system info for privacy-safe screenshots
- **🎨 Advanced Color Settings**: Customize every aspect of your fastfetch colors
- **📋 Interactive Guides**: Built-in help and navigation guides

---

## 📦 Installation

### Quick Install

```bash
git clone https://github.com/sinansarikaya/termfetch-studio.git
cd termfetch-studio
chmod +x install.sh
./install.sh
```

The installer will:
1. ✅ Check and install dependencies (fastfetch, optional tools)
2. ✅ Detect your distribution and terminal
3. ✅ Install themes and presets
4. ✅ Set up shell integration
5. ✅ Run the setup wizard

### Requirements

**Required:**
- Linux; see the tested distribution/version matrix in the README.
- Bash runtime; optional Bash/Zsh/Fish startup integration.
- fastfetch >= 2.42.0; the installer checks existing versions and offers installation where a supported source exists.

**Optional (for enhanced features):**
- `chafa` - Image to ASCII conversion
- `jp2a` - JPEG to ASCII conversion
- `imagemagick` - Image processing

### Terminal validation

- **Kitty:** recorded PNG, menu and theme checks in real VMs; latest UI/color follow-up was on Arch/Hyprland/Wayland, Kitty 0.49.2.
- **Konsole:** baseline KDE/Wayland VM checks include PNG via iTerm protocol, theme selection and persistence. Latest UI changes need a new regression here.
- **GNOME Terminal:** baseline GNOME/Wayland and separate X11 VM checks include ASCII fallback, theme selection and persistence. Latest UI changes need a new regression here.
- **WezTerm, Foot, iTerm2, Ghostty and other terminals:** no full graphical validation in this maintenance run. Do not infer application support from a terminal's protocol capabilities. This installer targets Linux; macOS installation was not tested.

Exact terminal versions, screenshots and test scope are in the [README matrix](../README.md#compatibility-and-test-evidence-x86_64).

---

## 🚀 Usage

### Interactive Menu

Simply run:
```bash
termfetch-studio
```

**Navigation:**
- Use **↑** and **↓** arrow keys to navigate
- Press **Enter** to select
- Press **Q** to go back/quit
- Intuitive, kitty-themes style interface

Navigate through the intuitive menu to:
- 🎨 Choose and apply themes
- 📊 Configure fastfetch presets
- 🖼️ Set up logos and images
- ⚙️ Adjust advanced settings
- 👁️ Preview your setup
- 💾 Backup and restore

### Command Line Options

```bash
termfetch-studio [OPTIONS]

Options:
  -h, --help       Show help message
  -v, --version    Show version information
  --preview        Quick preview of current setup
  --backup         Create a backup
  --restore        Restore from backup
```

### Quick Examples

```bash
# Start interactive menu (with arrow key navigation)
termfetch-studio

# Preview current configuration
termfetch-studio --preview

# Create a backup before experimenting
termfetch-studio --backup

# Show help
termfetch-studio --help
```

### First Run

On first run, TermFetch Studio will:
1. **Detect** your terminal and capabilities
2. **Run** a setup wizard
3. **Guide** you through choosing your first theme
4. **Apply** your configuration automatically

---

## 🎨 Themes

### Dark Themes

<table>
<tr>
<td width="50%">

**🦇 Dracula**
```
A dark theme with vibrant colors
Perfect for: All-around use
```

</td>
<td width="50%">

**❄️ Nord**
```
Arctic, north-bluish color palette
Perfect for: Clean, minimal look
```

</td>
</tr>
<tr>
<td width="50%">

**🍂 Gruvbox Dark**
```
Retro groove warm color scheme
Perfect for: Long coding sessions
```

</td>
<td width="50%">

**🌃 Tokyo Night**
```
Clean, elegant dark theme
Perfect for: Modern aesthetics
```

</td>
</tr>
<tr>
<td width="50%">

**🌑 One Dark**
```
Atom's iconic dark theme
Perfect for: Developers
```

</td>
<td width="50%">

**🌊 Oceanic Next**
```
Ocean-inspired blue theme
Perfect for: Calm environment
```

</td>
</tr>
<tr>
<td width="50%">

**⚫ Monochrome**
```
Pure black & white minimalism
Perfect for: Distraction-free work
```

</td>
<td width="50%">
</td>
</tr>
</table>

### Light Themes

<table>
<tr>
<td width="50%">

**☀️ Gruvbox Light**
```
Warm retro light variant
Perfect for: Daytime coding
```

</td>
<td width="50%">

**🌅 Solarized Light**
```
Precision colors for readability
Perfect for: Extended reading
```

</td>
</tr>
<tr>
<td width="50%">

**🌞 One Light**
```
Atom's light theme
Perfect for: Bright environments
```

</td>
<td width="50%">

**🏖️ Ayu Light**
```
Simple, bright and elegant
Perfect for: Minimalists
```

</td>
</tr>
</table>

### Colorful Themes

<table>
<tr>
<td width="50%">

**🌈 Synthwave**
```
Retro-futuristic neon colors
Perfect for: Standing out
```

</td>
<td width="50%">

**🎨 Monokai Pro**
```
Professional color scheme
Perfect for: Developers
```

</td>
</tr>
<tr>
<td width="50%">

**🔮 Palenight**
```
Material design inspired
Perfect for: Modern UI lovers
```

</td>
<td width="50%">
</td>
</tr>
</table>

### Pastel & Cute Themes 💕

<table>
<tr>
<td width="50%">

**🌸 Sakura**
```
Cherry blossom pink theme
Perfect for: Kawaii aesthetics
```

</td>
<td width="50%">

**💜 Lavender Dream**
```
Soft purple lavender tones
Perfect for: Elegant & calm
```

</td>
</tr>
<tr>
<td width="50%">

**🍬 Candy**
```
Cotton candy pastel colors
Perfect for: Sweet & playful
```

</td>
<td width="50%">
</td>
</tr>
</table>

### Popular Culture Themes 🎮🎬

<table>
<tr>
<td width="50%">

**💚 Matrix**
```
Green on black hacker aesthetic
Perfect for: Coding like Neo
```

</td>
<td width="50%">

**🌆 Cyberpunk 2077**
```
Neon yellow and cyan futuristic
Perfect for: Night City vibes
```

</td>
</tr>
<tr>
<td width="50%">

**🍜 Naruto**
```
Orange and blue ninja theme
Perfect for: Anime fans
```

</td>
<td width="50%">

**⚡ Pokemon**
```
Bright yellow Pikachu colors
Perfect for: Gotta catch 'em all
```

</td>
</tr>
<tr>
<td width="50%">

**🕷️ Spider-Man**
```
Red and blue superhero theme
Perfect for: Marvel fans
```

</td>
<td width="50%">

**👹 Doom**
```
Red and orange hellfire
Perfect for: Demon slaying
```

</td>
</tr>
<tr>
<td width="50%">

**🔥 Valorant**
```
Red and green tactical colors
Perfect for: FPS gamers
```

</td>
<td width="50%">

**⛏️ Minecraft**
```
Grass green blocky aesthetics
Perfect for: Crafting adventures
```

</td>
</tr>
</table>

---

## 📊 Fastfetch Presets

All presets feature a **modern grouped design** with:
- 🎨 **Color-coded groups** for better organization
- 🌳 **Tree-style icons** (├ └) for visual hierarchy
- 🎯 **Logical grouping** of related information
- ✨ **Nerd Font icons** for every module

### 1. 📋 Full Info
Complete system information organized in 5 groups:
- **OS Group** (Red Icons): System, Kernel, Packages, Shell
- **WM Group** (Green Icons): Window Manager, Desktop, Theme, Terminal
- **PC Group** (Yellow Icons): CPU, GPU, Memory, Disk, Battery
- **NET Group** (Blue Icons): Local/Public IP
- **TIME Group** (Magenta Icons): DateTime, Media Player, Uptime
- **🎨 Smart Text Colors**: All text uses consistent theme-based colors for readability

**Best for:** System administrators, showing off your setup

### 2. 📝 Minimal
Essential information in 1 clean group:
- **OS Group** (Red): System, Kernel, Shell, Uptime

**Best for:** Quick glances, clean look, slow systems

### 3. 🎯 Focused
Balanced display with 3 key groups:
- **OS Group** (Red): System, Kernel, Packages, Uptime
- **WM Group** (Green): Terminal, Shell
- **PC Group** (Yellow): CPU, Memory, Disk

**Best for:** Daily use, balanced information

### 4. 💻 Developer
Development-focused with 4 groups:
- **OS Group** (Red): System, Kernel, Shell
- **WM Group** (Green): Terminal, Git branch
- **PC Group** (Yellow): CPU, Memory, Disk
- **NET Group** (Blue): Local/Public IP

**Best for:** Developers, programmers

### 5. 🎮 Gaming
Performance-focused with 3 groups:
- **OS Group** (Red): System, Kernel
- **PC Group** (Green): CPU, GPU, Memory, Resolution, Swap, Battery
- **NET Group** (Yellow): Local IP

**Best for:** Gamers, performance monitoring

### 6. 🧩 Custom Preset
Create your own module combinations:
- **Choose any modules** from 25+ available options
- **Reorder modules** with drag-and-drop style interface
- **Custom colors** for keys, values, titles, and underlines
- **Live preview** of your custom configuration

**Best for:** Personal preferences, unique setups

---

## 🎨 Advanced Color Customization

### One live color editor

Open **Main Menu → Icon & Text Colors**. The duplicate Advanced Settings entry has been removed.

- **Icons / Text:** independent global module foregrounds.
- **OS, Desktop, Hardware, Network, Time:** group overrides color both the icons and values of that group's modules, wherever present in the chosen preset.
- **Inherit** retains preset/global colors, **Theme** uses the group's theme accent, **Default** uses the terminal foreground.
- Palette: 15 ANSI foregrounds and 16 named RGB shades; RGB needs a truecolor terminal. No external palette service is used.
- **↑↓** chooses a field; **←→** or Enter on a color field cycles its color. The inline sample updates without saving.
- **Save & Back** commits; **Q** cancels. **Reset to Theme** changes the draft until saved.

Preview order stays OS → Desktop → Hardware → Network → Time. All five appear where height permits; tiny windows show a contiguous ordered slice. Inline values are samples, not hardware probes. The full preview calls actual fastfetch. Titles and fastfetch's internal percentage colors are outside group overrides.

See [group-color evidence](../tests/group-colors-evidence/README.md) and [ordered-preview screenshots](../tests/ordered-colors-evidence/README.md).

---

## 🎭 Demo Mode

Demo Mode displays fake system information instead of real data - perfect for taking screenshots for GitHub, Reddit, or other platforms without exposing your actual system details.

### Enabling Demo Mode

1. **Navigate to Settings**:
   - Main Menu → `⚙️ Advanced Settings`

2. **Toggle Demo Mode**:
   - Select `🎭 Demo Mode: ❌ Off`
   - The toggle will switch to `✅ On`

3. **Preview Your Setup**:
   ```bash
   termfetch-studio --preview
   ```

### Demo Values Displayed

When Demo Mode is enabled, the following fake values are shown:

| Module | Demo Value |
|--------|------------|
| Title | `hacker@linuxbox` |
| OS | `Arch Linux x86_64` |
| Kernel | `6.12.1-arch1-1` |
| CPU | `AMD Ryzen 9 7950X3D (32) @ 5.7GHz` |
| GPU | `NVIDIA GeForce RTX 4090` |
| Memory | `32.00 GiB / 64.00 GiB (50%)` |
| Disk | `512.00 GiB / 2.00 TiB (25%)` |
| Local IP | `192.168.1.100` |
| Public IP | `203.0.113.42` |
| Uptime | `5 days, 12 hours, 34 mins` |
| WM | `Hyprland` |
| Terminal | `kitty` |
| Shell | `fish 3.7.0` |

### Supported Presets

Demo Mode works with the following presets:
- ✅ Full Info
- ✅ Minimal
- ✅ Focused
- ✅ Developer
- ✅ Gaming

> **Note:** Custom Preset currently doesn't support Demo Mode due to its dynamic module structure.

---

## 🧩 Custom Preset System

### Creating Custom Presets

1. **Navigate to Custom Preset**:
   - Main Menu → `📊 Fastfetch Presets` → `🧩 Custom Preset`

2. **Select Modules**:
   - Use **↑↓** arrow keys to navigate
   - Press **ENTER** to toggle selection
   - **✓** = Selected, **✗** = Not selected

3. **Reorder Modules**:
   - Press **R** to enter reorder mode
   - Use **←→** to select item to move
   - Use **↑↓** to move selected item
   - Press **ENTER** to save new order

4. **Save Configuration**:
   - Press **S** to save your custom preset
   - Press **Q** to quit without saving

### Available Modules

**System Information:**
- Operating System, Kernel, Packages, Shell
- Window Manager, Desktop Environment, WM Theme
- Icon Theme, Cursor Theme, Terminal, Terminal Font

**Hardware Information:**
- Host, CPU, GPU, Memory, Swap, Disk, Battery, Display

**Network Information:**
- Local IP, Public IP, WiFi

**Time Information:**
- Date & Time, Uptime, Media Player

### Reorder Mode Guide

```
📋 REORDER GUIDE:
  [←→] Select item to move
  [↑↓] Move selected item up/down
  [ENTER] Save new order
  [Q] Cancel and exit
```

**Tips:**
- Select the item you want to move with **←→**
- Move it up/down with **↑↓**
- The order you create is the order fastfetch will display
- You can reorder as many times as you want

---

## 🖼️ Screenshots

### Interactive Menu

![TermFetch Studio Menu](../menu.png)

*The main menu with arrow key navigation and live preview*

### Full Info Preset

![Full Info Preset](../full-info.png)

*Complete system information with color-coded groups*

### Custom Setups

Share your custom configurations! Open a PR with your setup.

---

## 🖼️ Adding Custom Images

### Prerequisites

For the best experience with custom images, you need:

1. **A terminal with recorded application image checks:** Kitty or Konsole in the linked baseline VM environments. WezTerm, Foot and iTerm2 require manual verification; macOS installation is untested.
2. **ImageMagick** (optional, for format conversion and resizing):
   ```bash
   # Arch/Manjaro
   sudo pacman -S imagemagick

   # Debian/Ubuntu
   sudo apt install imagemagick

   # Fedora
   sudo dnf install ImageMagick
   ```

### How to Add an Image

1. **Launch TermFetch Studio**:
   ```bash
   termfetch-studio
   ```

2. **Navigate to Logo Settings**:
   - Select `🖼️ Logo & Image Settings`
   - Choose `🖼️ Custom Image`

3. **Enter your image path**:
   ```bash
   /path/to/your/image.png
   ```

### Supported Image Formats

- ✅ **PNG** (recommended, best performance)
- ✅ **JPG/JPEG** (auto-converted to PNG)
- ✅ **GIF** (auto-converted to PNG)
- ✅ **BMP** (auto-converted to PNG)

### Image Optimization

TermFetch Studio automatically:
- **Converts** non-PNG formats to PNG for optimal performance
- **Resizes** images larger than 1000px width to 800px
- **Copies** processed images to `~/.config/termfetch-studio/images/`

### Tips for Best Results

1. **Image Size**: Use images around 500-800px for best display
2. **Aspect Ratio**: Square or portrait orientation works best
3. **File Location**: Keep original images in a permanent location
4. **Transparency**: PNG with transparency looks great with most themes

### Terminal-Specific Notes

Use the generated logo configuration rather than copying a protocol name manually.
Kitty and Konsole PNG, and GNOME Terminal/xterm ASCII fallback were recorded in
real baseline VMs. New UI rendering was subsequently checked in native Kitty on
Arch/Wayland; see the dated evidence matrix. WezTerm, Foot and macOS/iTerm2 require
manual verification. Install `chafa` for supported image-to-text conversion.

---

## 🔧 Configuration

### Configuration File Location

```
~/.config/termfetch-studio/config.ini
```

### Manual Configuration

You can manually edit the configuration file:

```ini
[colors]
theme=dracula

[fastfetch]
preset=full
icons=true
logo_type=auto
custom_image=
logo_size=medium

[general]
first_run=false
backup_enabled=true
auto_backup=true

[advanced]
custom_colors=false
animation_enabled=true
preview_before_apply=true
```

### Custom Theme Format

Create your own theme file in `~/.config/termfetch-studio/themes/`:

```ini
# My Custom Theme
name=My Theme
type=dark
author=Your Name

# Background and Foreground
background=#1e1e1e
foreground=#d4d4d4

# Normal Colors (0-7)
color0=#1e1e1e
color1=#f44747
color2=#4ec9b0
color3=#ffcc66
color4=#569cd6
color5=#c586c0
color6=#4fc1ff
color7=#d4d4d4

# Bright Colors (8-15)
color8=#5a5a5a
color9=#f44747
color10=#4ec9b0
color11=#ffcc66
color12=#569cd6
color13=#c586c0
color14=#4fc1ff
color15=#ffffff
```

---

## 🆘 Troubleshooting

### Changes not persisting after terminal restart

Make sure fastfetch is enabled in your shell configuration:

```bash
# Check your shell config file
# For Bash:
nano ~/.bashrc

# For Zsh:
nano ~/.zshrc

# For Fish:
nano ~/.config/fish/config.fish

# Make sure this line is uncommented:
fastfetch
```

The installer automatically adds this, but it may be commented out. Simply remove the `#` to enable it.

### Fastfetch not found

The installer should handle this automatically, but if needed:

```bash
# Arch/Manjaro
sudo pacman -S fastfetch

# Debian/Ubuntu
sudo apt install fastfetch

# Fedora
sudo dnf install fastfetch

# From source (if not in repos)
# The installer handles this automatically
```

### Menu navigation not working

If arrow keys don't work:
- Make sure your terminal supports ANSI escape sequences
- Try a different terminal emulator
- Check if `$TERM` is set correctly: `echo $TERM`

### Images not displaying

Check your terminal support:
```bash
termfetch-studio
# Go to: System Info → Check terminal capabilities
```

Recorded PNG checks: Kitty and Konsole in the baseline VM matrix. Other terminal/protocol combinations require manual verification; consult the dated README matrix.

For others, use ASCII art fallback:
```bash
sudo pacman -S chafa jp2a  # or your package manager
```

### Colors not applying

1. Make sure you're using a compatible terminal
2. Try restarting your terminal
3. Check if your terminal supports truecolor:
```bash
echo $COLORTERM
# Should output: truecolor or 24bit
```

### Permission denied

Don't run the installer as root:
```bash
./install.sh  # Not: sudo ./install.sh
```

The installer will ask for sudo when needed.

### Emoji/Icons not showing

1. Install a Nerd Font or Font Awesome compatible font
2. Configure your terminal to use the font
3. Toggle icons in settings if needed

---

## 🗑️ Uninstallation

### Uninstall application files

```bash
termfetch-studio uninstall
```

The uninstaller will ask if you want to keep your configuration files.

### Keep Configuration

If you want to reinstall later with your current settings:
- Answer **N** to `Remove config directory? (y/N):` (the question asks whether to remove it).
- Your themes and settings will be preserved

### Fish after uninstall

The uninstaller removes the exact legacy `alias fastfetch="termfetch-studio --preview"`
line and keeps a backup beside `config.fish`. Custom functions/compound aliases remain.
An already-open Fish keeps loaded functions; run:

```fish
functions --erase fastfetch
```

If you removed an older installation, also remove that exact stale alias line from
`~/.config/fish/config.fish` yourself before opening another terminal. There is no
need to delete your personal `clear` function. Review any custom autoload/plugin
wrapper separately. See [before/after evidence](../tests/ordered-colors-evidence/README.md).

### Installation paths

The current installer uses `~/.local/bin` for executables, `~/.local/share/termfetch-studio` for application files and `~/.config/termfetch-studio` for settings. Use `termfetch-studio uninstall` to handle removal and shell integration.

---

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

### Reporting Bugs

Open an issue with:
- Your distribution and version
- Terminal emulator and version
- Steps to reproduce
- Expected vs actual behavior

### Adding Themes

1. Create your theme file
2. Test it thoroughly
3. Submit a PR with:
   - Theme file
   - Screenshot
   - Description

### Feature Requests

Open an issue with:
- Feature description
- Use case
- Why it would be useful

### Code Contributions

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a PR

---

