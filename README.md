# TermFetch Studio

<div align="center">

```
╔═══════════════════════════════════════════════════╗
║         ████████╗███████╗██████╗ ███╗   ███╗     ║
║         ╚══██╔══╝██╔════╝██╔══██╗████╗ ████║     ║
║            ██║   █████╗  ██████╔╝██╔████╔██║     ║
║            ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║     ║
║            ██║   ███████╗██║  ██║██║ ╚═╝ ██║     ║
║            ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝     ║
║                                                   ║
║      Professional Terminal Theme Manager          ║
╚═══════════════════════════════════════════════════╝
```

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/sinansarikaya/termfetch-studio)
[![Linux](https://img.shields.io/badge/platform-Linux-green.svg)](https://www.linux.org/)

**Transform your terminal into a beautiful, informative workspace**

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [Themes](#themes) • [Screenshots](#screenshots)

</div>

---

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
- **Image Support**: Display custom images (Kitty, WezTerm, iTerm2, Foot)
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
- Linux (any distribution)
- Bash/Zsh/Fish shell
- Fastfetch (auto-installed if missing)

**Optional (for enhanced features):**
- `chafa` - Image to ASCII conversion
- `jp2a` - JPEG to ASCII conversion
- `imagemagick` - Image processing

### Supported Terminals

| Terminal | Color Support | Image Support | Theme Application | Status |
|----------|--------------|---------------|-------------------|--------|
| Kitty | ✅ Truecolor | ✅ Full | ✅ Auto | Full Support |
| Alacritty | ✅ Truecolor | ❌ No | ✅ Auto | Full Support |
| WezTerm | ✅ Truecolor | ✅ Full | ✅ Auto | Full Support |
| GNOME Terminal | ✅ Truecolor | ❌ No | ✅ Auto | Full Support |
| Konsole | ✅ Truecolor | ❌ No | ⚠️ Manual | Partial |
| Foot | ✅ Truecolor | ✅ Full | ⚠️ Manual | Partial |
| iTerm2 | ✅ Truecolor | ✅ Full | ⚠️ Manual | Partial |
| xterm/st | ⚠️ Basic | ❌ No | ❌ No | Basic |

**Theme Application:**
- ✅ Auto - Themes are automatically applied to terminal colors
- ⚠️ Manual - Requires manual configuration
- ❌ No - Not supported

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

### Color Settings Menu
Access through: `⚙️ Advanced Settings → 🎨 Color Settings`

**Available Color Categories:**
- **🔑 Icon Color (keyColor)**: Colors for module icons (OS, Kernel, etc.)
- **📝 Text Color (outputColor)**: Colors for the actual information text
- **🏷️ Title Color**: Colors for the main title/header
- **📏 Underline Color**: Colors for the separator line

**Color Modes:**
- **🎨 Theme Colors**: Automatic color selection based on current theme
- **🎯 Custom Colors**: Manual color selection for complete control
- **🔄 Smart Switching**: Easy toggle between theme and custom modes

**Color Options:**
- Blue, Red, Green, Yellow, Magenta, Cyan, White, Black
- Bright variants: Bright Blue, Bright Red, etc.
- Default option for value colors

**Features:**
- **🎨 Live Preview**: See changes before applying
- **🔄 Reset to Default**: One-click restore to theme colors
- **⚡ Auto-Apply**: Changes apply immediately
- **🎯 Independent Control**: Icon and text colors can be set separately

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

### Dark Theme Showcase

```
Coming soon - Add your own screenshots!
```

### Light Theme Showcase

```
Coming soon - Add your own screenshots!
```

### Custom Setups

Share your custom configurations! Open a PR with your setup.

---

## 🖼️ Adding Custom Images

### Prerequisites

For the best experience with custom images, you need:

1. **Terminal with image support**: Kitty, WezTerm, iTerm2, or Foot
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

#### Kitty
- Best overall support
- Use `type: "kitty"` protocol
- Supports all image formats

#### WezTerm
- Good support with recent versions
- Use PNG for best compatibility

#### iTerm2 (macOS)
- Full support on macOS
- Inline image protocol

#### Foot
- Sixel protocol support
- May have limited color depth

#### Unsupported Terminals
- Falls back to ASCII art automatically
- Install `chafa` for better ASCII art conversion:
  ```bash
  sudo pacman -S chafa  # Arch
  sudo apt install chafa  # Ubuntu
  ```

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

Supported terminals: Kitty, WezTerm, iTerm2, Foot

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

### Full Uninstall (removes everything)

```bash
sudo /usr/local/share/termfetch-studio/uninstall.sh
```

The uninstaller will ask if you want to keep your configuration files.

### Keep Configuration

If you want to reinstall later with your current settings:
- Answer "yes" when asked to keep configuration files
- Your themes and settings will be preserved

### Manual Cleanup

```bash
# Remove executable
sudo rm /usr/local/bin/termfetch-studio

# Remove installation directory
sudo rm -rf /usr/local/share/termfetch-studio

# Remove configuration (optional)
rm -rf ~/.config/termfetch-studio
```

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

## 📋 Roadmap

### Version 1.1 (Coming Soon)
- [ ] Theme store with community themes
- [ ] Logo pack downloads
- [ ] More fastfetch presets
- [ ] Terminal color scheme export
- [ ] GUI configuration editor

### Version 1.2
- [ ] Cloud backup sync
- [ ] Theme scheduling (day/night)
- [ ] Plugin system
- [ ] Advanced animation options

### Long-term
- [ ] Multi-monitor support
- [ ] Integration with other tools (neofetch, etc.)
- [ ] Mobile terminal support (Termux)
- [ ] Web-based theme editor

---

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2024 Sinan Sarıkaya

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
```

---

## 🙏 Acknowledgments

- **Fastfetch** - Fast system information tool
- **Dracula Theme** - Beautiful color scheme
- **Nord Theme** - Arctic color palette
- **Tokyo Night** - Clean dark theme
- All theme creators and contributors

---

## 📞 Contact & Support

- **GitHub**: [github.com/sinansarikaya/termfetch-studio](https://github.com/sinansarikaya/termfetch-studio)
- **Issues**: [Report a bug](https://github.com/sinansarikaya/termfetch-studio/issues)
- **Discussions**: [Community forum](https://github.com/sinansarikaya/termfetch-studio/discussions)

---

## ⭐ Star History

If you find this project useful, please consider giving it a star on GitHub!

---

<div align="center">

**Made with ❤️ by [Sinan Sarıkaya](https://github.com/sinansarikaya)**

*Transform your terminal, elevate your workflow*

</div>