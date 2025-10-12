#!/bin/bash

# TermFetch Studio - Professional Terminal Theme & Fastfetch Manager
# Author: Sinan Sarıkaya
# GitHub: https://github.com/sinansarikaya/termfetch-studio
# License: MIT

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Installation paths
INSTALL_DIR="$HOME/.local/share/termfetch-studio"
BIN_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/termfetch-studio"
BACKUP_DIR="$CONFIG_DIR/backups"
IMAGES_DIR="$CONFIG_DIR/images"
LOG_FILE="$CONFIG_DIR/debug.log"

# Logging functions
log_debug() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] DEBUG: $message" >> "$LOG_FILE"
}

log_error() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] ERROR: $message" >> "$LOG_FILE"
}

log_terminal_info() {
    log_debug "Terminal: $TERM"
    log_debug "Terminal Program: $TERM_PROGRAM"
    log_debug "Shell: $SHELL"
    # Use the dedicated function for consistency
    local image_support=$(check_terminal_image_support)
    log_debug "Image Support: $image_support"
    log_debug "Kitty Window ID: $KITTY_WINDOW_ID"
    log_debug "WezTerm Executable: $WEZTERM_EXECUTABLE"
    log_debug "Konsole Version: $KONSOLE_VERSION"
    log_debug "GNOME Terminal Screen: $GNOME_TERMINAL_SCREEN"
    log_debug "Wayland Display: $WAYLAND_DISPLAY"
    log_debug "Hyprland Instance: $HYPRLAND_INSTANCE_SIGNATURE"
    log_debug "X11 Display: $DISPLAY"
}

print_banner() {
    echo -e "${CYAN}${BOLD}"
    cat << "EOF"
╔═══════════════════════════════════════════════════╗
║                                                   ║
║         ████████╗███████╗██████╗ ███╗   ███╗      ║
║         ╚══██╔══╝██╔════╝██╔══██╗████╗ ████║      ║
║            ██║   █████╗  ██████╔╝██╔████╔██║      ║
║            ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║      ║
║            ██║   ███████╗██║  ██║██║ ╚═╝ ██║      ║
║            ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝      ║
║                                                   ║
║              TermFetch Studio v1.0                ║
║     Professional Terminal Theme Manager           ║
║                                                   ║
╚═══════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

print_info() {
    echo -e "${BLUE}${BOLD}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}${BOLD}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}${BOLD}[!]${NC} $1"
}

print_error() {
    echo -e "${RED}${BOLD}[✗]${NC} $1"
}

detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif [ -f /etc/lsb-release ]; then
        . /etc/lsb-release
        echo "$DISTRIB_ID" | tr '[:upper:]' '[:lower:]'
    else
        echo "unknown"
    fi
}

detect_distro_base() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        if [[ "$ID" == "fedora" ]] || [[ "$ID" == "rhel" ]] || [[ "$ID" == "centos" ]]; then
            echo "fedora"
        elif [[ "$ID" == "ubuntu" ]] || [[ "$ID" == "debian" ]] || [[ "$ID" == "linuxmint" ]] || [[ "$ID" == "pop" ]]; then
            echo "debian"
        elif [[ "$ID" == "arch" ]] || [[ "$ID" == "manjaro" ]] || [[ "$ID" == "endeavouros" ]] || [[ "$ID" == "cachyos" ]]; then
            echo "arch"
        elif [[ "$ID" == "opensuse" ]] || [[ "$ID" == "suse" ]]; then
            echo "suse"
        else
            echo "unknown"
        fi
    else
        echo "unknown"
    fi
}

# Gum is not used in this application - removed

install_fastfetch_from_source() {
    print_info "Installing fastfetch from GitHub releases..."
    
    local arch=$(uname -m)
    local url=""
    
    case $arch in
        x86_64)
            url="https://github.com/fastfetch-cli/fastfetch/releases/latest/download/fastfetch-linux-amd64.tar.gz"
            ;;
        aarch64|arm64)
            url="https://github.com/fastfetch-cli/fastfetch/releases/latest/download/fastfetch-linux-aarch64.tar.gz"
            ;;
        *)
            print_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
    
    cd /tmp
    wget -q "$url" -O fastfetch.tar.gz
    tar -xzf fastfetch.tar.gz
    cp fastfetch-*/usr/bin/fastfetch /usr/local/bin/
    chmod +x /usr/local/bin/fastfetch
    rm -rf fastfetch* 
    
    print_success "Fastfetch installed from source"
}

install_dependencies() {
    print_info "Checking and installing dependencies..."

    local distro_base=$(detect_distro_base)
    local missing_deps=()
    local optional_missing=()

    # Check required dependencies
    if ! command -v fastfetch &> /dev/null; then
        missing_deps+=("fastfetch")
    fi

    # Check optional dependencies
    if ! command -v chafa &> /dev/null; then
        optional_missing+=("chafa")
    fi

    if ! command -v convert &> /dev/null; then
        optional_missing+=("imagemagick")
    fi

    # Auto-install missing required dependencies
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_info "Installing required dependencies: ${missing_deps[*]}"
        case $distro_base in
            arch)
                sudo pacman -S --noconfirm fastfetch || {
                    print_error "Failed to install fastfetch"
                    print_info "Please install manually: sudo pacman -S fastfetch"
                    return 1
                }
                ;;
            debian)
                # For Debian/Ubuntu/Mint, fastfetch is not in default repos
                # Download and install from GitHub releases
                print_info "Downloading fastfetch from GitHub..."

                local arch=$(uname -m)
                local deb_arch="amd64"
                [ "$arch" = "aarch64" ] && deb_arch="arm64"

                local tmp_dir=$(mktemp -d)
                local latest_url="https://github.com/fastfetch-cli/fastfetch/releases/latest/download/fastfetch-linux-${deb_arch}.deb"

                if wget -q "$latest_url" -O "$tmp_dir/fastfetch.deb"; then
                    print_info "Installing fastfetch..."
                    sudo dpkg -i "$tmp_dir/fastfetch.deb" || {
                        print_info "Fixing dependencies..."
                        sudo apt-get install -f -y
                    }
                    rm -rf "$tmp_dir"

                    if command -v fastfetch &> /dev/null; then
                        print_success "Fastfetch installed successfully"
                    else
                        print_error "Failed to install fastfetch"
                        return 1
                    fi
                else
                    print_error "Failed to download fastfetch"
                    print_info "Please install manually from: https://github.com/fastfetch-cli/fastfetch/releases"
                    rm -rf "$tmp_dir"
                    return 1
                fi
                ;;
            fedora)
                sudo dnf install -y fastfetch || {
                    print_error "Failed to install fastfetch"
                    print_info "Please install manually: sudo dnf install fastfetch"
                    return 1
                }
                ;;
            suse)
                sudo zypper install -y fastfetch || {
                    print_error "Failed to install fastfetch"
                    print_info "Please install manually: sudo zypper install fastfetch"
                    return 1
                }
                ;;
            *)
                print_error "Unsupported distribution. Please install fastfetch manually"
                print_info "Check: https://github.com/fastfetch-cli/fastfetch"
                return 1
                ;;
        esac
    fi
    
    # Auto-install optional dependencies
    if [ ${#optional_missing[@]} -ne 0 ]; then
        print_info "Installing optional dependencies: ${optional_missing[*]}"
        case $distro_base in
            arch)
                sudo pacman -S --noconfirm chafa imagemagick 2>/dev/null || print_warning "Some optional dependencies failed to install"
                ;;
            debian)
                sudo apt install -y chafa imagemagick 2>/dev/null || print_warning "Some optional dependencies failed to install"
                ;;
            fedora)
                sudo dnf install -y chafa ImageMagick 2>/dev/null || print_warning "Some optional dependencies failed to install"
                ;;
            suse)
                sudo zypper install -y chafa ImageMagick 2>/dev/null || print_warning "Some optional dependencies failed to install"
                ;;
        esac
        print_success "Optional dependencies installed"
    fi
    
    # Verify installation
    if command -v fastfetch &> /dev/null; then
        print_success "All required dependencies are installed"
    else
        print_error "Failed to install fastfetch"
        exit 1
    fi
}

install_nerd_fonts() {
    print_info "Checking for Nerd Fonts..."
    
    # Check if any Nerd Font is installed
    local font_installed=false
    if fc-list 2>/dev/null | grep -i "nerd" &> /dev/null; then
        font_installed=true
    fi
    
    if [ "$font_installed" = false ]; then
        echo ""
        print_warning "Nerd Fonts not detected. These are required for icons to display properly."
        echo ""
        read -p "Install Nerd Fonts? (Y/n) " -n 1 -r
        echo

        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            local distro_base=$(detect_distro_base)
            case $distro_base in
                arch)
                    print_info "Installing Nerd Fonts via pacman..."
                    sudo pacman -S --noconfirm ttf-jetbrains-mono-nerd 2>/dev/null || true
                    ;;
                debian|fedora|suse)
                    print_info "Installing Fontconfig utilities..."
                    case $distro_base in
                        debian) sudo apt install -y fontconfig curl unzip 2>/dev/null || true ;;
                        fedora) sudo dnf install -y fontconfig curl unzip 2>/dev/null || true ;;
                        suse) sudo zypper install -y fontconfig curl unzip 2>/dev/null || true ;;
                    esac
                    
                    print_info "Downloading and installing JetBrainsMono Nerd Font..."
                    mkdir -p "$HOME/.local/share/fonts"
                    cd /tmp
                    curl -L -o JetBrainsMono.zip "https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.zip"
                    unzip -o JetBrainsMono.zip -d "$HOME/.local/share/fonts/JetBrainsMono"
                    fc-cache -fv "$HOME/.local/share/fonts/JetBrainsMono"
                    rm -f JetBrainsMono.zip
                    print_success "JetBrainsMono Nerd Font installed"
                    ;;
            esac
            
            echo ""
            print_warning "Please configure your terminal to use the installed Nerd Font"
        fi
    else
        print_success "Nerd Fonts already installed"
    fi
}

create_directories() {
    print_info "Creating directories..."

    # Create user directories (no sudo needed)
    mkdir -p "$CONFIG_DIR"/{themes,backups,images,logos}
    mkdir -p "$BACKUP_DIR"
    mkdir -p "$IMAGES_DIR"

    # Create local installation directory in user's home
    mkdir -p "$HOME/.local/share/termfetch-studio"/{lib,themes,presets,scripts,assets,logos}

    # Create bin directory for executable
    mkdir -p "$BIN_DIR"

    print_success "Directories created"
}

install_ascii_logos() {
    print_info "Installing ASCII logos..."
    
    # Pikachu ASCII
    tee "$INSTALL_DIR/logos/pikachu.txt" > /dev/null << 'PIKACHU'
        ╭─────────────────╮
       ╱                 ╲
      ╱    ╭─────────╮    ╲
     ╱    ╱           ╲    ╲
    ╱    ╱  ╭─────╮    ╲    ╲
   ╱    ╱  ╱       ╲    ╲    ╲
  ╱    ╱  ╱  ╭─╮   ╲    ╲    ╲
 ╱    ╱  ╱  ╱   ╲   ╲    ╲    ╲
╱    ╱  ╱  ╱  ╭─╮  ╲    ╲    ╲
╲    ╲  ╲  ╲  ╰─╯  ╱    ╱    ╱
 ╲    ╲  ╲  ╲     ╱    ╱    ╱
  ╲    ╲  ╲  ╰───╯    ╱    ╱
   ╲    ╲  ╲         ╱    ╱
    ╲    ╲  ╰───────╯    ╱
     ╲    ╲             ╱
      ╲    ╰───────────╯
       ╲                 ╱
        ╰─────────────────╯
PIKACHU

    # Tux ASCII
    tee "$INSTALL_DIR/logos/tux.txt" > /dev/null << 'TUX'
       .---.
      /     \
      \.@-@./
      /`\_/`\
     //  _  \\
    | \     )|_
   /`\_`>  <_/ \
   \__/'---'\__/
TUX

    # Arch Logo
    tee "$INSTALL_DIR/logos/arch.txt" > /dev/null << 'ARCHLOGO'
                   -`
                  .o+`
                 `ooo/
                `+oooo:
               `+oooooo:
               -+oooooo+:
             `/:-:++oooo+:
            `/++++/+++++++:
           `/++++++++++++++:
          `/+++ooooooooooooo/`
         ./ooosssso++osssssso+`
        .oossssso-````/ossssss+`
       -osssssso.      :ssssssso.
      :osssssss/        osssso+++.
     /ossssssss/        +ssssooo/-
   `/ossssso+/:-        -:/+osssso+-
  `+sso+:-`                 `.-/+oso:
 `++:.                           `-/+/
 .`                                 `/
ARCHLOGO

    # Cat ASCII
    tee "$INSTALL_DIR/logos/cat.txt" > /dev/null << 'CAT'
     /\_/\
    ( o.o )
     > ^ <
CAT

    # Neko ASCII
    tee "$INSTALL_DIR/logos/neko.txt" > /dev/null << 'NEKO'
  ∧＿∧
 ( ･ω･)
 ＿|  ⊃／(＿＿_
／ └-(＿＿＿／
￣￣￣￣￣￣￣
NEKO

    # Bunny ASCII
    tee "$INSTALL_DIR/logos/bunny.txt" > /dev/null << 'BUNNY'
 /\ /\
( ^.^ )
 (")_(")
BUNNY

    # Heart ASCII
    tee "$INSTALL_DIR/logos/heart.txt" > /dev/null << 'HEART'
  ♥♥♥♥  ♥♥♥♥
♥♥♥♥♥♥♥♥♥♥♥♥♥
♥♥♥♥♥♥♥♥♥♥♥♥♥
 ♥♥♥♥♥♥♥♥♥♥♥
  ♥♥♥♥♥♥♥♥♥
   ♥♥♥♥♥♥♥
    ♥♥♥♥♥
     ♥♥♥
      ♥
HEART

    # Sakura ASCII
    tee "$INSTALL_DIR/logos/sakura.txt" > /dev/null << 'SAKURA'
    *
   ***
  *****
   ***
    *
SAKURA

    # Naruto ASCII
    tee "$INSTALL_DIR/logos/naruto.txt" > /dev/null << 'NARUTO'
    ____
   /    \
  | () () |
   \  ~  /
    |||||
NARUTO

    # Pokeball ASCII
    tee "$INSTALL_DIR/logos/pokeball.txt" > /dev/null << 'POKEBALL'
     ___
   /     \
  |  ()   |
  |_______|
   \     /
     ---
POKEBALL

    # Spider ASCII
    tee "$INSTALL_DIR/logos/spider.txt" > /dev/null << 'SPIDER'
  /\_/\
 ( o o )
  =\_/=
   / \
SPIDER

    # Dragon ASCII
    tee "$INSTALL_DIR/logos/dragon.txt" > /dev/null << 'DRAGON'
    ______________
   /              \
  |  ()      ()   |
   \    \/\/     /
     \_________/
        | |
DRAGON

    # Creeper ASCII (Minecraft)
    tee "$INSTALL_DIR/logos/creeper.txt" > /dev/null << 'CREEPER'
  ________
 |  []  []|
 |        |
 |  ____  |
 | |    | |
  -------
CREEPER

    # Star ASCII
    tee "$INSTALL_DIR/logos/star.txt" > /dev/null << 'STAR'
    *
   ***
  *****
 *******
  *****
   ***
    *
STAR

    # Rocket ASCII
    tee "$INSTALL_DIR/logos/rocket.txt" > /dev/null << 'ROCKET'
    /\
   /  \
  |    |
  |    |
 /|    |\
/ |    | \
  |____|
  |    |
  |____|
ROCKET

    print_success "ASCII logos installed"
}

install_preset_images() {
    print_info "Installing preset images..."

    local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local preset_logos_source="$script_dir/preset-logos"

    # Check if preset-logos directory exists in source
    if [ -d "$preset_logos_source" ]; then
        mkdir -p "$INSTALL_DIR/preset-logos"
        cp -r "$preset_logos_source"/* "$INSTALL_DIR/preset-logos/" 2>/dev/null || true
        print_success "Preset images installed ($(ls -1 "$preset_logos_source" 2>/dev/null | wc -l) files)"
    else
        print_warning "preset-logos directory not found, skipping preset images"
    fi
}

install_main_script() {
    print_info "Installing main executable..."
    
    tee "$BIN_DIR/termfetch-studio" > /dev/null << 'MAINSCRIPT'
#!/bin/bash

# TermFetch Studio Main Script
VERSION="1.0.0"
INSTALL_DIR="$HOME/.local/share/termfetch-studio"
CONFIG_DIR="$HOME/.config/termfetch-studio"

# Load libraries
for lib in "$INSTALL_DIR/lib"/*.sh; do
    [ -f "$lib" ] && source "$lib"
done

# Ensure core library is loaded
if ! declare -f init_config >/dev/null 2>&1; then
    echo "Error: Core library not loaded properly. Please reinstall TermFetch Studio."
    exit 1
fi

# Parse arguments
case "${1:-}" in
    --version|-v)
        echo "TermFetch Studio v$VERSION"
        exit 0
        ;;
    --help|-h)
        cat << 'EOF'
 TermFetch Studio - Professional Terminal Theme Manager

Usage: termfetch-studio [OPTIONS]

Options:
  -h, --help       Show this help message
  -v, --version    Show version information
  --preview        Preview current configuration
  --backup         Create a backup
  --restore        Restore from backup
  uninstall        Uninstall TermFetch Studio

Interactive mode (default):
  Simply run 'termfetch-studio' to enter interactive menu

Navigation:
  ↑↓    - Move up/down
  Enter - Select
  Q     - Back/Quit

Examples:
  termfetch-studio              # Start interactive menu
  termfetch-studio --preview    # Quick preview
  termfetch-studio --backup     # Create backup
  termfetch-studio uninstall    # Uninstall application

GitHub: https://github.com/sinansarikaya/termfetch-studio
EOF
        exit 0
        ;;
    --preview)
        init_config
        clear
        if [ -x "$HOME/.local/bin/tfs-fastfetch" ]; then
            "$HOME/.local/bin/tfs-fastfetch"
        else
            fastfetch -c "$CONFIG_DIR/fastfetch.jsonc"
        fi
        exit 0
        ;;
    --backup)
        init_config
        create_backup
        exit 0
        ;;
    --restore)
        init_config
        restore_backup
        exit 0
        ;;
    uninstall)
        bash "$INSTALL_DIR/uninstall.sh"
        exit 0
        ;;
    "")
        # Interactive mode
        init_config
        first_run_wizard
        main_menu
        ;;
    *)
        echo "Unknown option: $1"
        echo "Run 'termfetch-studio --help' for usage information"
        exit 1
        ;;
esac
MAINSCRIPT
    
    chmod +x "$BIN_DIR/termfetch-studio"
    print_success "Main executable installed"
}

install_fastfetch_wrapper() {
    print_info "Installing fastfetch runtime wrapper..."
    
    tee "$BIN_DIR/tfs-fastfetch" > /dev/null << 'WRAPPER'
#!/bin/bash
# TermFetch Studio runtime fastfetch wrapper
# Generates terminal-aware logo settings before running fastfetch

set -e

INSTALL_DIR="$HOME/.local/share/termfetch-studio"
CONFIG_DIR="$HOME/.config/termfetch-studio"
JSON="$CONFIG_DIR/fastfetch.jsonc"
TMP="$CONFIG_DIR/.fastfetch.current.jsonc"

# If base config missing, run fastfetch normally
if [ ! -f "$JSON" ]; then
  exec fastfetch "$@"
fi

# Load libs (core + fastfetch)
for lib in "$INSTALL_DIR/lib"/*.sh; do
  [ -f "$lib" ] && source "$lib"
done

# Guard: if helper functions not found, run normal
if ! declare -f get_logo_type >/dev/null 2>&1 || ! declare -f get_logo_source >/dev/null 2>&1; then
  exec fastfetch --config "$JSON" "$@"
fi

# Read user preferences
logo_type=$(get_config_value "fastfetch" "logo_type")
custom_image=$(get_config_value "fastfetch" "custom_image")
[ -z "$logo_type" ] && logo_type="auto"

# Compute runtime logo according to current terminal
logo_source=$(get_logo_source "$logo_type" "$custom_image")
logo_actual_type=$(get_logo_type "$logo_type" "$custom_image")

# Prepare temp config with adjusted logo fields
cp "$JSON" "$TMP"

# Update logo.type and logo.source inside the JSON safely with awk
awk -v t="$logo_actual_type" -v s="$logo_source" '
  BEGIN{ inlogo=0; has_source=0; has_type=0 }
  {
    # Detect logo block start
    if ($0 ~ /"logo"\s*:\s*\{/){ 
      inlogo=1
      has_source=0
      has_type=0
      print
      next
    }
    
    # Detect logo block end
    if (inlogo==1 && $0 ~ /^\s*}\s*,?\s*$/){
      # before closing, if s is non-empty and no source line present, insert one
      if (s != "" && has_source==0){
        print "        \"source\": \"" s "\",";
      }
      # if type is non-empty and no type line present, insert one
      if (t != "" && has_type==0){
        print "        \"type\": \"" t "\",";
      }
      inlogo=0
      has_source=0
      has_type=0
      print
      next
    }
    
    # Inside logo block - handle type field
    if (inlogo==1 && $0 ~ /"type"\s*:/){
      has_type=1
      if (t == "") {
        # drop type line entirely when empty (for ASCII files)
        next
      } else {
        sub(/"type"\s*:\s*"[^"]*"/, "\"type\": \"" t "\"")
        print
        next
      }
    }
    
    # Inside logo block - handle source field
    if (inlogo==1 && $0 ~ /"source"\s*:/){
      has_source=1
      if (s == "") {
        # drop source line entirely
        next
      } else {
        sub(/"source"\s*:\s*"[^"]*"/, "\"source\": \"" s "\"")
        print
        next
      }
    }
    
    # Default: print the line as-is
    print
  }
' "$JSON" > "$TMP"

# Run fastfetch with the temp config
exec fastfetch --config "$TMP" "$@"
WRAPPER
    
    chmod +x "$BIN_DIR/tfs-fastfetch"
    print_success "Fastfetch wrapper installed"
}

install_core_lib() {
    print_info "Installing core library..."
    
    tee "$INSTALL_DIR/lib/core.sh" > /dev/null << 'CORELIB'
#!/bin/bash

CONFIG_FILE="$CONFIG_DIR/config.ini"
IMAGES_DIR="$CONFIG_DIR/images"
LOGOS_DIR="$CONFIG_DIR/logos"
BACKUP_DIR="$CONFIG_DIR/backups"

show_header() {
    clear
    echo -e "\033[1;36m"
    cat << "EOF"
╔═══════════════════════════════════════════════════╗
║            TermFetch Studio v1.0                  ║
║     Professional Terminal Theme Manager           ║
╚═══════════════════════════════════════════════════╝
EOF
    echo -e "\033[0m"
}

select_option() {
    local prompt="$1"
    shift
    local options=("$@")
    local selected=0
    local total=${#options[@]}
    local preview_callback=""

    # Check if last argument is a preview callback function
    if [ $# -gt 0 ] && declare -f "${!#}" &>/dev/null; then
        preview_callback="${!#}"
        unset 'options[-1]'
        ((total--))
    fi

    # Helper function to draw menu
    draw_menu() {
        clear
        show_header
        echo ""
        echo -e "\033[1;33m$prompt\033[0m"
        echo ""

        for i in "${!options[@]}"; do
            if [ $i -eq $selected ]; then
                echo -e "  \033[1;32m▶\033[0m \033[7m${options[$i]}\033[0m"
            else
                echo -e "    ${options[$i]}"
            fi
        done

        # Show preview if callback provided with fixed height container
        if [ -n "$preview_callback" ]; then
            echo ""
            echo -e "\033[1;36m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m"
            echo -e "\033[1;33m👁️  Önizleme:\033[0m"
            echo ""

            # Capture preview output and pad to fixed height (15 lines)
            local preview_output=$($preview_callback $selected 2>&1)
            local preview_lines=$(echo "$preview_output" | wc -l)

            # Display preview
            echo "$preview_output"

            # Add empty lines to maintain fixed height (prevents scrollbar jumping)
            local padding=$((15 - preview_lines))
            if [ $padding -gt 0 ]; then
                for ((i=0; i<padding; i++)); do
                    echo ""
                done
            fi
        fi
    }

    tput civis 2>/dev/null
    draw_menu

    while true; do
        read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            read -rsn2 key
            case $key in
                '[A')
                    ((selected--))
                    [ $selected -lt 0 ] && selected=$((total - 1))
                    draw_menu
                    ;;
                '[B')
                    ((selected++))
                    [ $selected -ge $total ] && selected=0
                    draw_menu
                    ;;
            esac
        elif [[ $key == "" ]]; then
            tput cnorm 2>/dev/null
            clear
            return $selected
        elif [[ $key == "q" ]] || [[ $key == "Q" ]]; then
            tput cnorm 2>/dev/null
            clear
            return 255
        fi
    done
}

select_option_2col() {
    local prompt="$1"
    shift
    local options=("$@")
    local selected=0
    local total=${#options[@]}
    local cols=2
    local rows=$(( (total + cols - 1) / cols ))

    tput civis 2>/dev/null

    while true; do
        tput cup 0 0 2>/dev/null
        show_header
        echo ""
        echo -e "\033[1;33m$prompt\033[0m"
        echo ""

        # Display items in 2 columns
        for ((row=0; row<rows; row++)); do
            local left_idx=$row
            local right_idx=$((rows + row))

            # Left column
            if [ $left_idx -lt $total ]; then
                if [ $left_idx -eq $selected ]; then
                    printf "  \033[1;32m▶\033[0m \033[7m%-28s\033[0m" "${options[$left_idx]}"
                else
                    printf "    %-28s" "${options[$left_idx]}"
                fi
            else
                printf "%-30s" ""
            fi

            # Right column
            if [ $right_idx -lt $total ]; then
                if [ $right_idx -eq $selected ]; then
                    printf "  \033[1;32m▶\033[0m \033[7m%s\033[0m\n" "${options[$right_idx]}"
                else
                    printf "    %s\n" "${options[$right_idx]}"
                fi
            else
                echo ""
            fi
        done

        read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            read -rsn2 key
            case $key in
                '[A')  # Up arrow
                    if [ $selected -lt $rows ]; then
                        # In left column
                        if [ $selected -gt 0 ]; then
                            selected=$((selected - 1))
                        else
                            # Wrap to bottom of left column
                            selected=$((rows - 1))
                        fi
                    else
                        # In right column
                        if [ $selected -gt $rows ]; then
                            selected=$((selected - 1))
                        else
                            # Wrap to bottom of right column
                            selected=$((total - 1))
                        fi
                    fi
                    ;;
                '[B')  # Down arrow
                    if [ $selected -lt $rows ]; then
                        # In left column
                        if [ $selected -lt $((rows - 1)) ]; then
                            selected=$((selected + 1))
                        else
                            # Wrap to top of left column
                            selected=0
                        fi
                    else
                        # In right column
                        if [ $selected -lt $((total - 1)) ]; then
                            selected=$((selected + 1))
                        else
                            # Wrap to top of right column
                            selected=$rows
                        fi
                    fi
                    ;;
                '[C')  # Right arrow
                    if [ $selected -lt $rows ] && [ $((selected + rows)) -lt $total ]; then
                        selected=$((selected + rows))
                    fi
                    ;;
                '[D')  # Left arrow
                    if [ $selected -ge $rows ]; then
                        selected=$((selected - rows))
                    fi
                    ;;
            esac
        elif [[ $key == "" ]]; then
            tput cnorm 2>/dev/null
            return $selected
        elif [[ $key == "q" ]] || [[ $key == "Q" ]]; then
            tput cnorm 2>/dev/null
            return 255
        fi
    done
}

select_option_2col_with_preview() {
    local prompt="$1"
    shift
    local options=("$@")
    local preview_callback=""
    
    # Check if last argument is a preview callback function
    if [ $# -gt 0 ] && declare -f "${!#}" &>/dev/null; then
        preview_callback="${!#}"
        unset 'options[-1]'
    fi
    
    local selected=0
    local total=${#options[@]}
    local cols=2
    local rows=$(( (total + cols - 1) / cols ))
    local last_selected=-1

    tput civis 2>/dev/null

    while true; do
        tput cup 0 0 2>/dev/null
        show_header
        echo ""
        echo -e "\033[1;33m$prompt\033[0m"
        echo ""

        # Display items in 2 columns
        for ((row=0; row<rows; row++)); do
            local left_idx=$row
            local right_idx=$((rows + row))

            # Left column
            if [ $left_idx -lt $total ]; then
                if [ $left_idx -eq $selected ]; then
                    printf "  \033[1;32m▶\033[0m \033[7m%-28s\033[0m" "${options[$left_idx]}"
                else
                    printf "    %-28s" "${options[$left_idx]}"
                fi
            else
                printf "%-30s" ""
            fi

            # Right column
            if [ $right_idx -lt $total ]; then
                if [ $right_idx -eq $selected ]; then
                    printf "  \033[1;32m▶\033[0m \033[7m%s\033[0m\n" "${options[$right_idx]}"
                else
                    printf "    %s\n" "${options[$right_idx]}"
                fi
            else
                echo ""
            fi
        done

        # Show theme preview if callback provided and selection changed
        if [ -n "$preview_callback" ] && [ $selected -ne $last_selected ]; then
            echo ""
            echo -e "\033[1;36m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\033[0m"
            echo -e "\033[1;33m🎨 Theme Preview:\033[0m"
            echo ""
            
            # Apply theme preview
            $preview_callback $selected
            
            last_selected=$selected
        fi

        read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            read -rsn2 key
            case $key in
                '[A')  # Up arrow
                    if [ $selected -lt $rows ]; then
                        # In left column
                        if [ $selected -gt 0 ]; then
                            selected=$((selected - 1))
                        else
                            # Wrap to bottom of left column
                            selected=$((rows - 1))
                        fi
                    else
                        # In right column
                        if [ $selected -gt $rows ]; then
                            selected=$((selected - 1))
                        else
                            # Wrap to bottom of right column
                            selected=$((total - 1))
                        fi
                    fi
                    ;;
                '[B')  # Down arrow
                    if [ $selected -lt $rows ]; then
                        # In left column
                        if [ $selected -lt $((rows - 1)) ]; then
                            selected=$((selected + 1))
                        else
                            # Wrap to top of left column
                            selected=0
                        fi
                    else
                        # In right column
                        if [ $selected -lt $((total - 1)) ]; then
                            selected=$((selected + 1))
                        else
                            # Wrap to top of right column
                            selected=$rows
                        fi
                    fi
                    ;;
                '[C')  # Right arrow
                    if [ $selected -lt $rows ] && [ $((selected + rows)) -lt $total ]; then
                        selected=$((selected + rows))
                    fi
                    ;;
                '[D')  # Left arrow
                    if [ $selected -ge $rows ]; then
                        selected=$((selected - rows))
                    fi
                    ;;
            esac
        elif [[ $key == "" ]]; then
            tput cnorm 2>/dev/null
            return $selected
        elif [[ $key == "q" ]] || [[ $key == "Q" ]]; then
            tput cnorm 2>/dev/null
            return 255
        fi
    done
}

# Preview fastfetch preset function
preview_fastfetch_preset() {
    local selection=$1
    
    # Preset mapping (same order as in fastfetch_menu)
    local presets=(full minimal focused developer gaming custom)
    
    # Don't preview if selection is "Back" or invalid
    if [ $selection -ge ${#presets[@]} ] || [ $selection -lt 0 ]; then
        return
    fi
    
    local preset="${presets[$selection]}"
    local current_theme=$(get_config_value "colors" "theme")
    [ -z "$current_theme" ] && current_theme="dracula"
    
    # Show a preview of the selected preset
    if command -v fastfetch &> /dev/null; then
        # Create a temporary config for preview
        local temp_config="/tmp/fastfetch_preview.jsonc"
        
        # Generate config based on preset
        case "$preset" in
            "full")
                create_full_config "auto" "" "$current_theme" > "$temp_config" 2>/dev/null
                ;;
            "minimal")
                create_minimal_config "auto" "" "$current_theme" > "$temp_config" 2>/dev/null
                ;;
            "focused")
                create_focused_config "auto" "" "$current_theme" > "$temp_config" 2>/dev/null
                ;;
            "developer")
                create_developer_config "auto" "" "$current_theme" > "$temp_config" 2>/dev/null
                ;;
            "gaming")
                create_gaming_config "auto" "" "$current_theme" > "$temp_config" 2>/dev/null
                ;;
            "custom")
                create_custom_config "auto" "" "$current_theme" > "$temp_config" 2>/dev/null
                ;;
        esac
        
        # Show preview (suppress errors)
        if [ -f "$temp_config" ]; then
            fastfetch --config "$temp_config" 2>/dev/null | head -15 || true
        fi
        
        # Clean up
        rm -f "$temp_config" 2>/dev/null
    fi
}

# Preview theme function for theme menu
preview_theme() {
    local selection=$1
    
    # Theme mapping (same order as in theme_menu)
    local themes=(dracula nord gruvbox-dark tokyo-night one-dark oceanic-next monochrome
                  gruvbox-light solarized-light one-light ayu-light synthwave monokai-pro palenight sakura lavender candy
                  matrix cyberpunk naruto pokemon spiderman doom valorant minecraft)
    
    # Don't preview if selection is "Back" or invalid
    if [ $selection -ge ${#themes[@]} ] || [ $selection -lt 0 ]; then
        # Back button - no theme application, just return
        return
    fi
    
    local theme="${themes[$selection]}"
    
    # Apply theme colors temporarily for preview
    apply_terminal_colors "$theme" 2>/dev/null || true
    
    # Show a small fastfetch preview with the theme
    if command -v fastfetch &> /dev/null; then
        # Create a temporary config for preview
        local temp_config="/tmp/termfetch_preview.jsonc"
        
        # Generate a minimal config for preview
        cat > "$temp_config" << EOF
{
    "logo": {
        "type": "none"
    },
    "display": {
        "separator": " ",
        "padding": [0, 1],
        "linePrefix": "├─ "
    },
    "modules": [
        {
            "type": "title",
            "key": "title",
            "format": "TermFetch Studio Preview"
        },
        {
            "type": "os",
            "key": "os",
            "format": "{3} {5}"
        },
        {
            "type": "theme",
            "key": "theme",
            "format": "Theme: $theme"
        }
    ]
}
EOF
        
        # Show preview (suppress errors)
        fastfetch --config "$temp_config" 2>/dev/null | head -10 || true
        
        # Clean up
        rm -f "$temp_config" 2>/dev/null
    fi
}

# Toggle show/hide textual labels near icons in Fastfetch
toggle_labels() {
    local val=$(get_config_value "fastfetch" "labels")
    if [ "$val" = "true" ]; then
        set_config_value "fastfetch" "labels" "false"
        echo ""
        echo -e "\033[1;33m🏷️  Labels disabled (icons only)\033[0m"
    else
        set_config_value "fastfetch" "labels" "true"
        echo ""
        echo -e "\033[1;33m🏷️  Labels enabled (icons + text)\033[0m"
    fi
    # Re-apply current preset to reflect label change
    local preset=$(get_config_value "fastfetch" "preset")
    [ -z "$preset" ] && preset="full"
    apply_preset "$preset"
}

check_terminal_image_support() {
    # Enhanced terminal detection with better support for various terminals
    # Check specific terminal environment variables first
    if [ -n "$KITTY_WINDOW_ID" ] || [ "$TERM" = "xterm-kitty" ]; then
        echo "kitty"
    elif [ -n "$GHOSTTY_RESOURCES_DIR" ] || [ "$TERM_PROGRAM" = "ghostty" ]; then
        echo "kitty"  # Ghostty supports kitty graphics protocol
    elif [ -n "$WEZTERM_EXECUTABLE" ]; then
        echo "wezterm"
    elif [ "$TERM_PROGRAM" = "iTerm.app" ]; then
        echo "iterm2"
    elif [ -n "$KONSOLE_VERSION" ] || [ -n "$KONSOLE_DBUS_SERVICE" ]; then
        echo "konsole"
    elif [ "$TERM_PROGRAM" = "gnome-terminal" ] || [ "$TERM" = "gnome-256color" ] || [ "$TERM" = "gnome" ]; then
        echo "none"
    elif [ "$TERM_PROGRAM" = "terminator" ]; then
        echo "none"
    elif [ "$TERM_PROGRAM" = "xterm" ] || [ "$TERM" = "xterm-256color" ]; then
        echo "none"
    elif [ "$TERM_PROGRAM" = "foot" ]; then
        echo "foot"  # Foot supports sixel protocol
    elif [ "$TERM_PROGRAM" = "alacritty" ] || [ "$TERM" = "alacritty" ]; then
        echo "none"
    elif [ -n "$GNOME_TERMINAL_SCREEN" ]; then
        echo "none"
    elif [ "$TERM" = "dumb" ] || [ -z "$TERM_PROGRAM" ]; then
        # For dumb terminals or when TERM_PROGRAM is not set, try to detect the actual terminal
        # Check if we're in a known terminal emulator
        if command -v kitty &> /dev/null && pgrep -x kitty > /dev/null; then
            echo "kitty"
        elif command -v konsole &> /dev/null && pgrep -x konsole > /dev/null; then
            echo "konsole"
        elif command -v wezterm &> /dev/null && pgrep -x wezterm > /dev/null; then
            echo "wezterm"
        elif command -v foot &> /dev/null && pgrep -x foot > /dev/null; then
            echo "foot"  # Foot supports sixel protocol
        elif command -v alacritty &> /dev/null && pgrep -x alacritty > /dev/null; then
            echo "none"  # alacritty doesn't support image protocols used by this script
        else
            # Check for Wayland environment and common terminals
            if [ -n "$WAYLAND_DISPLAY" ] || [ -n "$HYPRLAND_INSTANCE_SIGNATURE" ]; then
                # We're in Wayland/Hyprland, try to detect terminal by process
                local parent_pid=$(ps -o ppid= -p $$ 2>/dev/null | tr -d ' ')
                local parent_cmd=$(ps -o cmd= -p $parent_pid 2>/dev/null)
                
                if echo "$parent_cmd" | grep -q "kitty"; then
                    echo "kitty"
                elif echo "$parent_cmd" | grep -q "wezterm"; then
                    echo "wezterm"
                elif echo "$parent_cmd" | grep -q "foot"; then
                    echo "foot"
                elif echo "$parent_cmd" | grep -q "konsole"; then
                    echo "konsole"
                else
                    echo "none"
                fi
            else
                # Default to none for unknown terminals
                echo "none"
            fi
        fi
    else
        echo "none"
    fi
}

apply_terminal_colors() {
    local theme=$1

    case $theme in
        dracula)
            # Set terminal colors for Dracula
            printf '\033]10;#f8f8f2\007'  # foreground
            printf '\033]11;#282a36\007'  # background
            printf '\033]4;0;#000000\007' # black
            printf '\033]4;1;#ff5555\007' # red
            printf '\033]4;2;#50fa7b\007' # green
            printf '\033]4;3;#f1fa8c\007' # yellow
            printf '\033]4;4;#bd93f9\007' # blue
            printf '\033]4;5;#ff79c6\007' # magenta
            printf '\033]4;6;#8be9fd\007' # cyan
            printf '\033]4;7;#bbbbbb\007' # white
            printf '\033]4;8;#44475a\007' # bright black
            printf '\033]4;9;#ff6e67\007' # bright red
            printf '\033]4;10;#5af78e\007' # bright green
            printf '\033]4;11;#f4f99d\007' # bright yellow
            printf '\033]4;12;#caa9fa\007' # bright blue
            printf '\033]4;13;#ff92d0\007' # bright magenta
            printf '\033]4;14;#9aedfe\007' # bright cyan
            printf '\033]4;15;#ffffff\007' # bright white
            ;;
        nord)
            printf '\033]10;#d8dee9\007'
            printf '\033]11;#2e3440\007'
            printf '\033]4;0;#3b4252\007'
            printf '\033]4;1;#bf616a\007'
            printf '\033]4;2;#a3be8c\007'
            printf '\033]4;3;#ebcb8b\007'
            printf '\033]4;4;#81a1c1\007'
            printf '\033]4;5;#b48ead\007'
            printf '\033]4;6;#88c0d0\007'
            printf '\033]4;7;#e5e9f0\007'
            printf '\033]4;8;#4c566a\007'
            printf '\033]4;9;#bf616a\007'
            printf '\033]4;10;#a3be8c\007'
            printf '\033]4;11;#ebcb8b\007'
            printf '\033]4;12;#81a1c1\007'
            printf '\033]4;13;#b48ead\007'
            printf '\033]4;14;#8fbcbb\007'
            printf '\033]4;15;#eceff4\007'
            ;;
        gruvbox-dark)
            printf '\033]10;#f0e6c8\007'  
            printf '\033]11;#33302f\007'  
            printf '\033]4;0;#3a3735\007' 
            printf '\033]4;1;#d65c5c\007'  
            printf '\033]4;2;#a7b86c\007'  
            printf '\033]4;3;#e3b65f\007'  
            printf '\033]4;4;#6d9cae\007'  
            printf '\033]4;5;#c97ea7\007'  
            printf '\033]4;6;#8dbb9e\007'  
            printf '\033]4;7;#c8bfae\007'  
            printf '\033]4;8;#7a7368\007'  
            printf '\033]4;9;#e07a70\007'  
            printf '\033]4;10;#c3d37a\007' 
            printf '\033]4;11;#f3cf83\007' 
            printf '\033]4;12;#84b5c7\007' 
            printf '\033]4;13;#e6a5c7\007' 
            printf '\033]4;14;#a8d7b8\007' 
            printf '\033]4;15;#f5ecda\007' 
            ;;
        tokyo-night)
            printf '\033]10;#c0caf5\007'
            printf '\033]11;#1a1b26\007'
            printf '\033]4;0;#15161e\007'
            printf '\033]4;1;#f7768e\007'
            printf '\033]4;2;#9ece6a\007'
            printf '\033]4;3;#e0af68\007'
            printf '\033]4;4;#7aa2f7\007'
            printf '\033]4;5;#bb9af7\007'
            printf '\033]4;6;#7dcfff\007'
            printf '\033]4;7;#a9b1d6\007'
            printf '\033]4;8;#414868\007'
            printf '\033]4;9;#f7768e\007'
            printf '\033]4;10;#9ece6a\007'
            printf '\033]4;11;#e0af68\007'
            printf '\033]4;12;#7aa2f7\007'
            printf '\033]4;13;#bb9af7\007'
            printf '\033]4;14;#7dcfff\007'
            printf '\033]4;15;#c0caf5\007'
            ;;
        one-dark)
            printf '\033]10;#abb2bf\007'
            printf '\033]11;#282c34\007'
            printf '\033]4;0;#282c34\007'
            printf '\033]4;1;#e06c75\007'
            printf '\033]4;2;#98c379\007'
            printf '\033]4;3;#e5c07b\007'
            printf '\033]4;4;#61afef\007'
            printf '\033]4;5;#c678dd\007'
            printf '\033]4;6;#56b6c2\007'
            printf '\033]4;7;#abb2bf\007'
            printf '\033]4;8;#5c6370\007'
            printf '\033]4;9;#e06c75\007'
            printf '\033]4;10;#98c379\007'
            printf '\033]4;11;#e5c07b\007'
            printf '\033]4;12;#61afef\007'
            printf '\033]4;13;#c678dd\007'
            printf '\033]4;14;#56b6c2\007'
            printf '\033]4;15;#ffffff\007'
            ;;
        synthwave)
            printf '\033]10;#ffffff\007'
            printf '\033]11;#2a2139\007'
            printf '\033]4;0;#241b2f\007'
            printf '\033]4;1;#fe4450\007'
            printf '\033]4;2;#72f1b8\007'
            printf '\033]4;3;#fede5d\007'
            printf '\033]4;4;#36f9f6\007'
            printf '\033]4;5;#ff7edb\007'
            printf '\033]4;6;#6e44ff\007'
            printf '\033]4;7;#f97e72\007'
            printf '\033]4;8;#554971\007'
            printf '\033]4;9;#fe4450\007'
            printf '\033]4;10;#72f1b8\007'
            printf '\033]4;11;#fede5d\007'
            printf '\033]4;12;#36f9f6\007'
            printf '\033]4;13;#ff7edb\007'
            printf '\033]4;14;#6e44ff\007'
            printf '\033]4;15;#ffffff\007'
            ;;
        oceanic-next)
            printf '\033]10;#c0c5ce\007'
            printf '\033]11;#1b2b34\007'
            printf '\033]4;0;#1b2b34\007'
            printf '\033]4;1;#ec5f67\007'
            printf '\033]4;2;#99c794\007'
            printf '\033]4;3;#fac863\007'
            printf '\033]4;4;#6699cc\007'
            printf '\033]4;5;#c594c5\007'
            printf '\033]4;6;#5fb3b3\007'
            printf '\033]4;7;#c0c5ce\007'
            printf '\033]4;8;#65737e\007'
            printf '\033]4;9;#ec5f67\007'
            printf '\033]4;10;#99c794\007'
            printf '\033]4;11;#fac863\007'
            printf '\033]4;12;#6699cc\007'
            printf '\033]4;13;#c594c5\007'
            printf '\033]4;14;#5fb3b3\007'
            printf '\033]4;15;#d8dee9\007'
            ;;
        monochrome)
            printf '\033]10;#ffffff\007'
            printf '\033]11;#000000\007'
            printf '\033]4;0;#000000\007'
            printf '\033]4;1;#7c7c7c\007'
            printf '\033]4;2;#8e8e8e\007'
            printf '\033]4;3;#a0a0a0\007'
            printf '\033]4;4;#686868\007'
            printf '\033]4;5;#747474\007'
            printf '\033]4;6;#868686\007'
            printf '\033]4;7;#b9b9b9\007'
            printf '\033]4;8;#525252\007'
            printf '\033]4;9;#7c7c7c\007'
            printf '\033]4;10;#8e8e8e\007'
            printf '\033]4;11;#a0a0a0\007'
            printf '\033]4;12;#686868\007'
            printf '\033]4;13;#747474\007'
            printf '\033]4;14;#868686\007'
            printf '\033]4;15;#ffffff\007'
            ;;
       gruvbox-light)
            printf '\033]10;#1a1a1a\007'
            printf '\033]11;#f6f3ec\007'
            printf '\033]4;0;#f6f3ec\007'
            printf '\033]4;1;#d85f5f\007'
            printf '\033]4;2;#9fb46b\007'
            printf '\033]4;3;#e0b35a\007'
            printf '\033]4;4;#6c9fb4\007'
            printf '\033]4;5;#c77ea8\007'
            printf '\033]4;6;#7ebda0\007'
            printf '\033]4;7;#c8beb2\007'
            printf '\033]4;8;#a89f96\007'
            printf '\033]4;9;#e2766a\007'
            printf '\033]4;10;#b8c672\007'
            printf '\033]4;11;#f0c97a\007'
            printf '\033]4;12;#8db3c1\007'
            printf '\033]4;13;#d799b9\007'
            printf '\033]4;14;#98cbb0\007'
            printf '\033]4;15;#1a1a1a\007'
            ;;
        solarized-light)
            printf '\033]10;#1a1a1a\007'
            printf '\033]11;#f8f6f2\007'
            printf '\033]4;0;#f8f6f2\007'
            printf '\033]4;1;#e17c86\007'
            printf '\033]4;2;#9cc9a5\007'
            printf '\033]4;3;#f2c87d\007'
            printf '\033]4;4;#7aa6d1\007'
            printf '\033]4;5;#c5a5e0\007'
            printf '\033]4;6;#90c5b9\007'
            printf '\033]4;7;#dcd3c3\007'
            printf '\033]4;8;#bcb4a8\007'
            printf '\033]4;9;#e68b8e\007'
            printf '\033]4;10;#aad7b2\007'
            printf '\033]4;11;#f5d899\007'
            printf '\033]4;12;#8ebde3\007'
            printf '\033]4;13;#d0b2eb\007'
            printf '\033]4;14;#9dd5c0\007'
            printf '\033]4;15;#1a1a1a\007'
            ;;
        one-light)
            printf '\033]10;#1a1a1a\007'
            printf '\033]11;#f0f8ff\007'
            printf '\033]4;0;#f0f8ff\007'
            printf '\033]4;1;#dc3545\007'
            printf '\033]4;2;#28a745\007'
            printf '\033]4;3;#ffc107\007'
            printf '\033]4;4;#007bff\007'
            printf '\033]4;5;#6f42c1\007'
            printf '\033]4;6;#17a2b8\007'
            printf '\033]4;7;#6c757d\007'
            printf '\033]4;8;#6c757d\007'
            printf '\033]4;9;#dc3545\007'
            printf '\033]4;10;#28a745\007'
            printf '\033]4;11;#ffc107\007'
            printf '\033]4;12;#007bff\007'
            printf '\033]4;13;#6f42c1\007'
            printf '\033]4;14;#17a2b8\007'
            printf '\033]4;15;#1a1a1a\007'
            ;;
        ayu-light)
            printf '\033]10;#1a1a1a\007'
            printf '\033]11;#f0fff0\007'
            printf '\033]4;0;#f0fff0\007'
            printf '\033]4;1;#e74c3c\007'
            printf '\033]4;2;#27ae60\007'
            printf '\033]4;3;#f39c12\007'
            printf '\033]4;4;#3498db\007'
            printf '\033]4;5;#9b59b6\007'
            printf '\033]4;6;#1abc9c\007'
            printf '\033]4;7;#95a5a6\007'
            printf '\033]4;8;#95a5a6\007'
            printf '\033]4;9;#e74c3c\007'
            printf '\033]4;10;#27ae60\007'
            printf '\033]4;11;#f39c12\007'
            printf '\033]4;12;#3498db\007'
            printf '\033]4;13;#9b59b6\007'
            printf '\033]4;14;#1abc9c\007'
            printf '\033]4;15;#1a1a1a\007'
            ;;
        monokai-pro)
            printf '\033]10;#fcfcfa\007'
            printf '\033]11;#2d2a2e\007'
            printf '\033]4;0;#2d2a2e\007'
            printf '\033]4;1;#ff6188\007'
            printf '\033]4;2;#a9dc76\007'
            printf '\033]4;3;#ffd866\007'
            printf '\033]4;4;#fc9867\007'
            printf '\033]4;5;#ab9df2\007'
            printf '\033]4;6;#78dce8\007'
            printf '\033]4;7;#fcfcfa\007'
            printf '\033]4;8;#727072\007'
            printf '\033]4;9;#ff6188\007'
            printf '\033]4;10;#a9dc76\007'
            printf '\033]4;11;#ffd866\007'
            printf '\033]4;12;#fc9867\007'
            printf '\033]4;13;#ab9df2\007'
            printf '\033]4;14;#78dce8\007'
            printf '\033]4;15;#fcfcfa\007'
            ;;
        palenight)
            printf '\033]10;#c5c9e0\007'
            printf '\033]11;#262626\007'
            printf '\033]4;0;#2c2c34\007'
            printf '\033]4;1;#ff7b86\007'
            printf '\033]4;2;#b4e88d\007'
            printf '\033]4;3;#ffd76b\007'
            printf '\033]4;4;#82b0ff\007'
            printf '\033]4;5;#c792ea\007'
            printf '\033]4;6;#8fdfff\007'
            printf '\033]4;7;#a9b1d6\007'
            printf '\033]4;8;#50586e\007'
            printf '\033]4;9;#ff99a1\007'
            printf '\033]4;10;#c6f0a0\007'
            printf '\033]4;11;#ffe37a\007'
            printf '\033]4;12;#a2c2ff\007'
            printf '\033]4;13;#d3a7ff\007'
            printf '\033]4;14;#a7ecff\007'
            printf '\033]4;15;#ffffff\007'
            ;;
        sakura)
            printf '\033]10;#ffe9f2\007'
            printf '\033]11;#262626\007'
            printf '\033]4;0;#332b30\007'
            printf '\033]4;1;#ff8fa3\007'
            printf '\033]4;2;#ffc1d9\007'
            printf '\033]4;3;#ffd8e2\007'
            printf '\033]4;4;#ffb8da\007'
            printf '\033]4;5;#ff99c8\007'
            printf '\033]4;6;#ffdaeb\007'
            printf '\033]4;7;#ffd6e8\007'
            printf '\033]4;8;#ffadc6\007'
            printf '\033]4;9;#ff6fae\007'
            printf '\033]4;10;#ffa7c9\007'
            printf '\033]4;11;#ffbfd9\007'
            printf '\033]4;12;#ff9dc8\007'
            printf '\033]4;13;#ff80b8\007'
            printf '\033]4;14;#ffd9ec\007'
            printf '\033]4;15;#fff0f5\007'
            ;;
        lavender)
            printf '\033]10;#e0dcf4\007'
            printf '\033]11;#262626\007'
            printf '\033]4;0;#302b38\007'
            printf '\033]4;1;#caa3e5\007'
            printf '\033]4;2;#d6b3f0\007'
            printf '\033]4;3;#e4c8fa\007'
            printf '\033]4;4;#bfa2e3\007'
            printf '\033]4;5;#d3b8f5\007'
            printf '\033]4;6;#e4d2fa\007'
            printf '\033]4;7;#f0e9ff\007'
            printf '\033]4;8;#a993cf\007'
            printf '\033]4;9;#b78ee1\007'
            printf '\033]4;10;#cba8f0\007'
            printf '\033]4;11;#dec0fa\007'
            printf '\033]4;12;#c39eed\007'
            printf '\033]4;13;#d8b2f8\007'
            printf '\033]4;14;#e9d9ff\007'
            printf '\033]4;15;#f6f3ff\007'
            ;;
        candy)
            printf '\033]10;#ffe3f2\007'
            printf '\033]11;#262626\007'
            printf '\033]4;0;#322730\007'
            printf '\033]4;1;#ff8abf\007'
            printf '\033]4;2;#ffb6de\007'
            printf '\033]4;3;#ffc9e8\007'
            printf '\033]4;4;#ffaee0\007'
            printf '\033]4;5;#ff9ad4\007'
            printf '\033]4;6;#ffd6f2\007'
            printf '\033]4;7;#ffeaf7\007'
            printf '\033]4;8;#ffb3da\007'
            printf '\033]4;9;#ff7ac4\007'
            printf '\033]4;10;#ff99d2\007'
            printf '\033]4;11;#ffb8e6\007'
            printf '\033]4;12;#ffa3dd\007'
            printf '\033]4;13;#ff8cd3\007'
            printf '\033]4;14;#ffd9ec\007'
            printf '\033]4;15;#fff0fa\007'
            ;;
        strawberry)
            printf '\033]10;#000000\007'  # pure black text for maximum contrast
            printf '\033]11;#fff0f5\007'  # strawberry cream background
            printf '\033]4;0;#fff0f5\007' # strawberry cream
            printf '\033]4;1;#ff69b4\007' # hot pink
            printf '\033]4;2;#ffb6c1\007' # light pink
            printf '\033]4;3;#ffa0b4\007' # strawberry pink
            printf '\033]4;4;#ffc0cb\007' # pink
            printf '\033]4;5;#ff91a4\007' # deep pink
            printf '\033]4;6;#ffb3d1\007' # light rose
            printf '\033]4;7;#ffe4e1\007' # misty rose
            printf '\033]4;8;#d4a5a5\007' # light coral
            printf '\033]4;9;#ff1493\007' # deep pink
            printf '\033]4;10;#ffadd2\007' # ultra pink
            printf '\033]4;11;#ffb3c1\007' # light pink
            printf '\033]4;12;#ffc0d1\007' # pink lace
            printf '\033]4;13;#ff91b4\007' # rose pink
            printf '\033]4;14;#ffb3d1\007' # light rose
            printf '\033]4;15;#000000\007' # pure black for maximum contrast
            ;;
        matrix)
            printf '\033]10;#00ff00\007'  # bright green text
            printf '\033]11;#000000\007'  # black background
            printf '\033]4;0;#000000\007' # black
            printf '\033]4;1;#003300\007' # dark green
            printf '\033]4;2;#00ff00\007' # bright green
            printf '\033]4;3;#00cc00\007' # green
            printf '\033]4;4;#00aa00\007' # medium green
            printf '\033]4;5;#00ff00\007' # neon green
            printf '\033]4;6;#00ee00\007' # lime
            printf '\033]4;7;#00dd00\007' # green
            printf '\033]4;8;#003300\007' # dark green
            printf '\033]4;9;#00ff00\007' # bright green
            printf '\033]4;10;#00ff00\007' # neon green
            printf '\033]4;11;#00ff00\007' # bright green
            printf '\033]4;12;#00cc00\007' # green
            printf '\033]4;13;#00ff00\007' # neon green
            printf '\033]4;14;#00ee00\007' # lime green
            printf '\033]4;15;#00ff00\007' # bright green
            ;;
        cyberpunk)
            printf '\033]10;#00ffff\007'  # cyan text
            printf '\033]11;#0a0e27\007'  # dark blue background
            printf '\033]4;0;#0a0e27\007' # dark blue
            printf '\033]4;1;#ff003c\007' # neon pink
            printf '\033]4;2;#00ff9f\007' # neon green
            printf '\033]4;3;#ffff00\007' # yellow
            printf '\033]4;4;#00b8ff\007' # electric blue
            printf '\033]4;5;#ff00ff\007' # magenta
            printf '\033]4;6;#00ffff\007' # cyan
            printf '\033]4;7;#e0e0e0\007' # white
            printf '\033]4;8;#1a1e3f\007' # dark
            printf '\033]4;9;#ff0055\007' # hot pink
            printf '\033]4;10;#00ff00\007' # bright green
            printf '\033]4;11;#ffff00\007' # bright yellow
            printf '\033]4;12;#00d4ff\007' # bright blue
            printf '\033]4;13;#ff00ff\007' # bright magenta
            printf '\033]4;14;#00ffff\007' # bright cyan
            printf '\033]4;15;#ffffff\007' # white
            ;;
        naruto)
            printf '\033]10;#2c2416\007'  # dark brown text
            printf '\033]11;#ffefd5\007'  # papaya whip background
            printf '\033]4;0;#2c2416\007' # black
            printf '\033]4;1;#ff4500\007' # orange red
            printf '\033]4;2;#ff8c00\007' # dark orange
            printf '\033]4;3;#ffa500\007' # orange
            printf '\033]4;4;#4169e1\007' # royal blue
            printf '\033]4;5;#ff6347\007' # tomato
            printf '\033]4;6;#ff7f50\007' # coral
            printf '\033]4;7;#2c2416\007' # dark brown
            printf '\033]4;8;#654321\007' # dark brown
            printf '\033]4;9;#ff6347\007' # tomato red
            printf '\033]4;10;#ff8c00\007' # dark orange
            printf '\033]4;11;#ffa500\007' # orange
            printf '\033]4;12;#4682b4\007' # steel blue
            printf '\033]4;13;#ff7f50\007' # coral
            printf '\033]4;14;#ffa07a\007' # light salmon
            printf '\033]4;15;#1a1410\007' # very dark brown
            ;;
        pokemon)
            printf '\033]10;#2c2c2c\007'  # dark text
            printf '\033]11;#ffde00\007'  # pikachu yellow background
            printf '\033]4;0;#000000\007' # black
            printf '\033]4;1;#ff0000\007' # pokeball red
            printf '\033]4;2;#00ff00\007' # grass green
            printf '\033]4;3;#ffde00\007' # electric yellow
            printf '\033]4;4;#0066cc\007' # water blue
            printf '\033]4;5;#cc0066\007' # fairy pink
            printf '\033]4;6;#00cccc\007' # ice cyan
            printf '\033]4;7;#c0c0c0\007' # steel gray
            printf '\033]4;8;#666666\007' # dark gray
            printf '\033]4;9;#ff3333\007' # bright red
            printf '\033]4;10;#33ff33\007' # bright green
            printf '\033]4;11;#ffff00\007' # bright yellow
            printf '\033]4;12;#3399ff\007' # bright blue
            printf '\033]4;13;#ff33cc\007' # bright pink
            printf '\033]4;14;#33ffff\007' # bright cyan
            printf '\033]4;15;#ffffff\007' # white
            ;;
        spiderman)
            printf '\033]10;#e8e8e8\007'  # light gray text
            printf '\033]11;#c41e3a\007'  # red background
            printf '\033]4;0;#000000\007' # black
            printf '\033]4;1;#c41e3a\007' # red
            printf '\033]4;2;#0051ba\007' # blue
            printf '\033]4;3;#ff0000\007' # bright red
            printf '\033]4;4;#0066ff\007' # bright blue
            printf '\033]4;5;#990000\007' # dark red
            printf '\033]4;6;#003399\007' # navy blue
            printf '\033]4;7;#e8e8e8\007' # light gray
            printf '\033]4;8;#4d4d4d\007' # dark gray
            printf '\033]4;9;#ff0000\007' # bright red
            printf '\033]4;10;#3366ff\007' # bright blue
            printf '\033]4;11;#ff3333\007' # light red
            printf '\033]4;12;#0080ff\007' # light blue
            printf '\033]4;13;#cc0000\007' # red
            printf '\033]4;14;#0044cc\007' # blue
            printf '\033]4;15;#ffffff\007' # white
            ;;
        doom)
            printf '\033]10;#ff0000\007'  # red text
            printf '\033]11;#000000\007'  # black background
            printf '\033]4;0;#000000\007' # black
            printf '\033]4;1;#ff0000\007' # red
            printf '\033]4;2;#8b4513\007' # saddle brown
            printf '\033]4;3;#ff4500\007' # orange red
            printf '\033]4;4;#8b0000\007' # dark red
            printf '\033]4;5;#ff6347\007' # tomato
            printf '\033]4;6;#ff8c00\007' # dark orange
            printf '\033]4;7;#a52a2a\007' # brown
            printf '\033]4;8;#330000\007' # very dark red
            printf '\033]4;9;#ff0000\007' # bright red
            printf '\033]4;10;#b8860b\007' # dark goldenrod
            printf '\033]4;11;#ff6600\007' # orange
            printf '\033]4;12;#990000\007' # red
            printf '\033]4;13;#ff4500\007' # orange red
            printf '\033]4;14;#ffa500\007' # orange
            printf '\033]4;15;#ff0000\007' # red
            ;;
        valorant)
            printf '\033]10;#ece8e1\007'  # light text
            printf '\033]11;#0f1923\007'  # dark background
            printf '\033]4;0;#0f1923\007' # black
            printf '\033]4;1;#ff4655\007' # red
            printf '\033]4;2;#53fca8\007' # green
            printf '\033]4;3;#fad663\007' # yellow
            printf '\033]4;4;#3fa8c9\007' # blue
            printf '\033]4;5;#ff4655\007' # magenta
            printf '\033]4;6;#53fca8\007' # cyan
            printf '\033]4;7;#ece8e1\007' # white
            printf '\033]4;8;#1f2933\007' # bright black
            printf '\033]4;9;#ff4655\007' # bright red
            printf '\033]4;10;#53fca8\007' # bright green
            printf '\033]4;11;#fad663\007' # bright yellow
            printf '\033]4;12;#3fa8c9\007' # bright blue
            printf '\033]4;13;#ff4655\007' # bright magenta
            printf '\033]4;14;#53fca8\007' # bright cyan
            printf '\033]4;15;#ece8e1\007' # bright white
            ;;
        minecraft)
            printf '\033]10;#2c2c2c\007'  # dark text
            printf '\033]11;#7bc043\007'  # grass green background
            printf '\033]4;0;#3c2415\007' # dirt brown
            printf '\033]4;1;#8b0000\007' # red
            printf '\033]4;2;#7bc043\007' # grass green
            printf '\033]4;3;#d4af37\007' # gold
            printf '\033]4;4;#4682b4\007' # water blue
            printf '\033]4;5;#8a2be2\007' # purple
            printf '\033]4;6;#00bfff\007' # diamond cyan
            printf '\033]4;7;#dcdcdc\007' # light gray
            printf '\033]4;8;#696969\007' # dark gray
            printf '\033]4;9;#ff0000\007' # bright red
            printf '\033]4;10;#90ee90\007' # light green
            printf '\033]4;11;#ffd700\007' # gold
            printf '\033]4;12;#87ceeb\007' # sky blue
            printf '\033]4;13;#ba55d3\007' # orchid
            printf '\033]4;14;#00ffff\007' # cyan
            printf '\033]4;15;#ffffff\007' # white
            ;;
    esac
    
    # Apply theme to current terminal session
    export TERMFETCH_THEME="$theme"
}

get_theme_color() {
    local theme=$1
    local color_type=$2
    
    case $theme in
        dracula)
            case $color_type in
                primary) echo "#bd93f9" ;;
                secondary) echo "#ff79c6" ;;
                accent) echo "#50fa7b" ;;
                text) echo "#f8f8f2" ;;
                background) echo "#282a36" ;;
                *) echo "#bd93f9" ;;
            esac
            ;;
        nord)
            case $color_type in
                primary) echo "#81a1c1" ;;
                secondary) echo "#88c0d0" ;;
                accent) echo "#a3be8c" ;;
                text) echo "#d8dee9" ;;
                background) echo "#2e3440" ;;
                *) echo "#81a1c1" ;;
            esac
            ;;
        gruvbox-dark)
            case $color_type in
                primary) echo "#d79921" ;;
                secondary) echo "#458588" ;;
                accent) echo "#98971a" ;;
                text) echo="#ebdbb2" ;;
                background) echo="#282828" ;;
                *) echo="#d79921" ;;
            esac
            ;;
        tokyo-night)
            case $color_type in
                primary) echo="#7aa2f7" ;;
                secondary) echo="#bb9af7" ;;
                accent) echo="#9ece6a" ;;
                text) echo="#c0caf5" ;;
                background) echo="#1a1b26" ;;
                *) echo="#7aa2f7" ;;
            esac
            ;;
        one-dark)
            case $color_type in
                primary) echo="#61afef" ;;
                secondary) echo="#c678dd" ;;
                accent) echo="#98c379" ;;
                text) echo="#abb2bf" ;;
                background) echo="#282c34" ;;
                *) echo="#61afef" ;;
            esac
            ;;
        synthwave)
            case $color_type in
                primary) echo="#36f9f6" ;;
                secondary) echo="#ff7edb" ;;
                accent) echo="#72f1b8" ;;
                text) echo="#ffffff" ;;
                background) echo="#2a2139" ;;
                *) echo="#36f9f6" ;;
            esac
            ;;
        oceanic-next)
            case $color_type in
                primary) echo="#6699cc" ;;
                secondary) echo="#5fb3b3" ;;
                accent) echo="#99c794" ;;
                text) echo="#c0c5ce" ;;
                background) echo="#1b2b34" ;;
                *) echo="#6699cc" ;;
            esac
            ;;
        monochrome)
            case $color_type in
                primary) echo="#a0a0a0" ;;
                secondary) echo="#8e8e8e" ;;
                accent) echo="#7c7c7c" ;;
                text) echo="#ffffff" ;;
                background) echo="#000000" ;;
                *) echo="#a0a0a0" ;;
            esac
            ;;
        gruvbox-light)
            case $color_type in
                primary) echo="#ff69b4" ;;    # hot pink
                secondary) echo="#dda0dd" ;;  # plum
                accent) echo="#98fb98" ;;     # pale green
                text) echo="#000000" ;;       # pure black for maximum contrast
                background) echo="#fff0f5" ;; # lavender blush
                *) echo="#ff69b4" ;;
            esac
            ;;
        solarized-light)
            case $color_type in
                primary) echo="#ff85ad" ;;    # soft pink
                secondary) echo="#d9b3ff" ;;  # light purple
                accent) echo="#a8e6cf" ;;     # mint green
                text) echo="#000000" ;;       # pure black for maximum contrast
                background) echo="#f8f4ff" ;; # very light lavender
                *) echo="#ff85ad" ;;
            esac
            ;;
        one-light)
            case $color_type in
                primary) echo="#ff9ecf" ;;    # rose pink
                secondary) echo="#e6b3ff" ;;  # light violet
                accent) echo="#b3ffb3" ;;     # light green
                text) echo="#000000" ;;       # pure black for maximum contrast
                background) echo="#fff5f8" ;; # very light pink
                *) echo="#ff9ecf" ;;
            esac
            ;;
        ayu-light)
            case $color_type in
                primary) echo="#ff8fa3" ;;    # soft pink
                secondary) echo="#e6a8e6" ;;  # light pink
                accent) echo="#a8e6a8" ;;     # light mint green
                text) echo="#000000" ;;       # pure black for maximum contrast
                background) echo="#fef7ff" ;; # very light lavender
                *) echo="#ff8fa3" ;;
            esac
            ;;
        monokai-pro)
            case $color_type in
                primary) echo="#fc9867" ;;
                secondary) echo="#ab9df2" ;;
                accent) echo="#a9dc76" ;;
                text) echo="#fcfcfa" ;;
                background) echo="#2d2a2e" ;;
                *) echo="#fc9867" ;;
            esac
            ;;
        palenight)
            case $color_type in
                primary) echo="#82aaff" ;;
                secondary) echo="#c792ea" ;;
                accent) echo="#c3e88d" ;;
                text) echo="#a6accd" ;;
                background) echo="#292d3e" ;;
                *) echo="#82aaff" ;;
            esac
            ;;
        sakura)
            case $color_type in
                primary) echo="#ff69b4" ;;
                secondary) echo="#ff1493" ;;
                accent) echo="#98d8c8" ;;
                text) echo="#2d2a2e" ;;
                background) echo="#fff5f7" ;;
                *) echo="#ff69b4" ;;
            esac
            ;;
        lavender)
            case $color_type in
                primary) echo="#9370db" ;;
                secondary) echo="#ba55d3" ;;
                accent) echo="#b19cd9" ;;
                text) echo="#2d2a2e" ;;
                background) echo="#f5f3ff" ;;
                *) echo="#9370db" ;;
            esac
            ;;
        candy)
            case $color_type in
                primary) echo="#ff85ad" ;;
                secondary) echo="#d9b3ff" ;;
                accent) echo="#c2f0c2" ;;
                text) echo="#5d4e6d" ;;
                background) echo="#ffe6f0" ;;
                *) echo="#ff85ad" ;;
            esac
            ;;
        strawberry)
            case $color_type in
                primary) echo="#ff69b4" ;;    # hot pink
                secondary) echo="#ffa0b4" ;;  # strawberry pink
                accent) echo="#ffb6c1" ;;     # light pink
                text) echo="#000000" ;;       # pure black for maximum contrast
                background) echo="#fff0f5" ;; # strawberry cream
                *) echo="#ff69b4" ;;
            esac
            ;;
        *)
            echo="#569cd6" ;;
    esac
}

get_theme_ansi_color() {
    local theme=$1
    local position=$2  # primary, secondary, accent, highlight, info

    case $theme in
        dracula)
            case $position in
                primary) echo "35" ;;      # magenta/purple
                secondary) echo "36" ;;    # cyan
                accent) echo "32" ;;       # green
                highlight) echo "33" ;;    # yellow
                info) echo "34" ;;         # blue
                *) echo "35" ;;
            esac
            ;;
        nord)
            case $position in
                primary) echo "34" ;;      # blue
                secondary) echo "36" ;;    # cyan
                accent) echo "32" ;;       # green
                highlight) echo "33" ;;    # yellow
                info) echo "35" ;;         # magenta
                *) echo "34" ;;
            esac
            ;;
        gruvbox-dark)
            case $position in
                primary) echo "33" ;;      # yellow
                secondary) echo "34" ;;    # blue
                accent) echo "32" ;;       # green
                highlight) echo "31" ;;    # red
                info) echo "35" ;;         # magenta
                *) echo "33" ;;
            esac
            ;;
        tokyo-night)
            case $position in
                primary) echo "34" ;;      # blue
                secondary) echo "35" ;;    # magenta
                accent) echo "32" ;;       # green
                highlight) echo "33" ;;    # yellow
                info) echo "36" ;;         # cyan
                *) echo "34" ;;
            esac
            ;;
        one-dark)
            case $position in
                primary) echo "34" ;;      # blue
                secondary) echo "35" ;;    # magenta
                accent) echo "32" ;;       # green
                highlight) echo "33" ;;    # yellow
                info) echo "36" ;;         # cyan
                *) echo "34" ;;
            esac
            ;;
        synthwave)
            case $position in
                primary) echo "36" ;;      # cyan
                secondary) echo "35" ;;    # magenta
                accent) echo "32" ;;       # green
                highlight) echo "33" ;;    # yellow
                info) echo "34" ;;         # blue
                *) echo "36" ;;
            esac
            ;;
        oceanic-next)
            case $position in
                primary) echo "34" ;;      # blue
                secondary) echo "36" ;;    # cyan
                accent) echo "32" ;;       # green
                highlight) echo "33" ;;    # yellow
                info) echo "35" ;;         # magenta
                *) echo "34" ;;
            esac
            ;;
        monochrome)
            case $position in
                primary) echo "37" ;;      # white
                secondary) echo "37" ;;    # white
                accent) echo "37" ;;       # white
                highlight) echo "37" ;;    # white
                info) echo "37" ;;         # white
                *) echo "37" ;;
            esac
            ;;
        gruvbox-light|solarized-light|one-light|ayu-light)
            case $position in
                primary) echo "33" ;;      # yellow
                secondary) echo "34" ;;    # blue
                accent) echo "32" ;;       # green
                highlight) echo "31" ;;    # red
                info) echo "36" ;;         # cyan
                *) echo "33" ;;
            esac
            ;;
        monokai-pro)
            case $position in
                primary) echo "35" ;;      # magenta
                secondary) echo "36" ;;    # cyan
                accent) echo "32" ;;       # green
                highlight) echo "33" ;;    # yellow
                info) echo "34" ;;         # blue
                *) echo "35" ;;
            esac
            ;;
        cyberpunk)
            case $position in
                primary) echo "36" ;;      # cyan
                secondary) echo "35" ;;    # magenta
                accent) echo "33" ;;       # yellow
                highlight) echo "31" ;;    # red
                info) echo "34" ;;         # blue
                *) echo "36" ;;
            esac
            ;;
        material)
            case $position in
                primary) echo "34" ;;      # blue
                secondary) echo "36" ;;    # cyan
                accent) echo "32" ;;       # green
                highlight) echo "33" ;;    # yellow
                info) echo "35" ;;         # magenta
                *) echo "34" ;;
            esac
            ;;
        sakura|lavender|candy|strawberry)
            case $position in
                primary) echo "35" ;;      # magenta
                secondary) echo "36" ;;    # cyan
                accent) echo "32" ;;       # green
                highlight) echo "33" ;;    # yellow
                info) echo "34" ;;         # blue
                *) echo "35" ;;
            esac
            ;;
        matrix)
            case $position in
                primary) echo "32" ;;      # green
                secondary) echo "32" ;;    # green
                accent) echo "32" ;;       # green
                highlight) echo "32" ;;    # green
                info) echo "32" ;;         # green
                *) echo "32" ;;
            esac
            ;;
        naruto)
            case $position in
                primary) echo "33" ;;      # orange/yellow
                secondary) echo "34" ;;    # blue
                accent) echo "31" ;;       # red
                highlight) echo "33" ;;    # yellow
                info) echo "34" ;;         # blue
                *) echo "33" ;;
            esac
            ;;
        pokemon)
            case $position in
                primary) echo "33" ;;      # yellow
                secondary) echo "31" ;;    # red
                accent) echo "34" ;;       # blue
                highlight) echo "32" ;;    # green
                info) echo "35" ;;         # magenta
                *) echo "33" ;;
            esac
            ;;
        spiderman)
            case $position in
                primary) echo "31" ;;      # red
                secondary) echo "34" ;;    # blue
                accent) echo "37" ;;       # white
                highlight) echo "31" ;;    # red
                info) echo "34" ;;         # blue
                *) echo "31" ;;
            esac
            ;;
        doom)
            case $position in
                primary) echo "31" ;;      # red
                secondary) echo "33" ;;    # yellow/orange
                accent) echo "31" ;;       # red
                highlight) echo "31" ;;    # red
                info) echo "33" ;;         # yellow
                *) echo "31" ;;
            esac
            ;;
        valorant)
            case $position in
                primary) echo "31" ;;      # red
                secondary) echo "32" ;;    # green
                accent) echo "36" ;;       # cyan
                highlight) echo "33" ;;    # yellow
                info) echo "34" ;;         # blue
                *) echo "31" ;;
            esac
            ;;
        minecraft)
            case $position in
                primary) echo "32" ;;      # green
                secondary) echo "33" ;;    # gold/yellow
                accent) echo "34" ;;       # blue
                highlight) echo "36" ;;    # cyan
                info) echo "35" ;;         # magenta
                *) echo "32" ;;
            esac
            ;;
        palenight)
            case $position in
                primary) echo "34" ;;      # blue
                secondary) echo "35" ;;    # magenta
                accent) echo "32" ;;       # green
                highlight) echo "33" ;;    # yellow
                info) echo "36" ;;         # cyan
                *) echo "34" ;;
            esac
            ;;
        *)
            # Default colors for unknown themes
            case $position in
                primary) echo "34" ;;      # blue
                secondary) echo "36" ;;    # cyan
                accent) echo "32" ;;       # green
                highlight) echo "33" ;;    # yellow
                info) echo "35" ;;         # magenta
                *) echo "34" ;;
            esac
            ;;
    esac
}

main_menu() {
    while true; do
        local current_theme=$(get_config_value "colors" "theme")
        local current_preset=$(get_config_value "fastfetch" "preset")
        local icons_enabled=$(get_config_value "fastfetch" "icons")
        local logo_type=$(get_config_value "fastfetch" "logo_type")
        
        local options=(
            "🎨 Color Themes"
            "📊 Fastfetch Presets"
            "🖼️ Logo & Image Settings"
            "🎨 Icon & Text Colors"
            "⚙️ Advanced Settings"
            "👁️ Preview Setup"
            "💾 Backup & Restore"
            "ℹ️ System Info"
            "❓ Help & Info"
            "🔧 Run Setup Wizard"
            "🚪 Exit"
        )
        
        select_option "📋 Main Menu (Theme: $current_theme | Preset: $current_preset | Logo: $logo_type) [↑↓ Navigate, Enter to Select, Q to Exit]" "${options[@]}"
        local choice=$?
        
        [ $choice -eq 255 ] && choice=10

        case $choice in
            0) theme_menu ;;
            1) fastfetch_menu ;;
            2) logo_menu ;;
            3) color_settings_menu ;;
            4) settings_menu ;;
            5) preview_setup ;;
            6) backup_menu ;;
            7) system_info ;;
            8) help_info_menu ;;
            9) setup_wizard ;;
            10)
                clear
                # Run preview to show current configuration
                if command -v termfetch-studio &> /dev/null; then
                    termfetch-studio --preview
                else
                    # Fallback if command not found
                    echo -e "\n\033[1;32m✨ Thank you for using TermFetch Studio! ✨\033[0m\n"
                fi
                exit 0
                ;;
        esac
    done
}

color_settings_menu() {
    while true; do
        local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
        [ -z "$use_theme_colors" ] && use_theme_colors="true"
        local key_color=$(get_config_value "fastfetch" "key_color")
        [ -z "$key_color" ] && key_color="34"
        local value_color=$(get_config_value "fastfetch" "value_color")
        [ -z "$value_color" ] && value_color="default"
        
        local options=(
            "🎨 Use Theme Colors: $use_theme_colors"
            "🔑 Icon Color: $key_color"
            "📝 Text Color: $value_color"
            "🔄 Reset to Theme Colors"
            "← Back"
        )
        
        select_option "🎨 Icon & Text Colors (Theme Colors: $use_theme_colors) [↑↓ Navigate, Enter to Select, Q to Back]" "${options[@]}"
        local choice=$?
        
        [ $choice -eq 4 ] || [ $choice -eq 255 ] && return
        
        case $choice in
            0) toggle_theme_colors ;;
            1) set_icon_color ;;
            2) set_text_color ;;
            3) reset_to_theme_colors ;;
        esac
    done
}

toggle_theme_colors() {
    local current=$(get_config_value "fastfetch" "use_theme_colors")
    if [ "$current" = "true" ]; then
        set_config_value "fastfetch" "use_theme_colors" "false"
        # Set default custom colors if not already set
        local key_color=$(get_config_value "fastfetch" "key_color")
        local value_color=$(get_config_value "fastfetch" "value_color")
        [ -z "$key_color" ] && set_config_value "fastfetch" "key_color" "34"
        [ -z "$value_color" ] && set_config_value "fastfetch" "value_color" "37"
        echo -e "\n\033[1;33m🎨 Using custom colors (Icon: 34, Text: 37)\033[0m"
    else
        set_config_value "fastfetch" "use_theme_colors" "true"
        echo -e "\n\033[1;33m🎨 Using theme colors\033[0m"
    fi
    
    # Re-apply current preset
    local preset=$(get_config_value "fastfetch" "preset")
    apply_preset "$preset"
}

set_icon_color() {
    show_header
    echo -e "\033[1;33m🔑 Icon Color\033[0m"
    echo ""
    echo -e "Available colors:"
    echo "  31=Red, 32=Green, 33=Yellow, 34=Blue, 35=Magenta, 36=Cyan, 37=White"
    echo ""
    echo -ne "Enter color code (31-37): "
    read -r color_code
    
    if [[ "$color_code" =~ ^[3][1-7]$ ]]; then
        set_config_value "fastfetch" "key_color" "$color_code"
        set_config_value "fastfetch" "use_theme_colors" "false"
        
        # Re-apply current preset
        local preset=$(get_config_value "fastfetch" "preset")
        apply_preset "$preset"
        echo -e "\n\033[1;32m✓ Icon color updated!\033[0m"
    else
        echo -e "\n\033[1;31m✗ Invalid color code!\033[0m"
    fi
    sleep 2
}

set_text_color() {
    show_header
    echo -e "\033[1;33m📝 Text Color\033[0m"
    echo ""
    echo -e "Available options:"
    echo "  default = Use theme colors"
    echo "  31=Red, 32=Green, 33=Yellow, 34=Blue, 35=Magenta, 36=Cyan, 37=White"
    echo ""
    echo -ne "Enter color (default or 31-37): "
    read -r color_code
    
    if [[ "$color_code" = "default" ]] || [[ "$color_code" =~ ^[3][1-7]$ ]]; then
        set_config_value "fastfetch" "value_color" "$color_code"
        set_config_value "fastfetch" "use_theme_colors" "false"
        
        # Re-apply current preset
        local preset=$(get_config_value "fastfetch" "preset")
        apply_preset "$preset"
        echo -e "\n\033[1;32m✓ Text color updated!\033[0m"
    else
        echo -e "\n\033[1;31m✗ Invalid color!\033[0m"
    fi
    sleep 2
}

reset_to_theme_colors() {
    set_config_value "fastfetch" "use_theme_colors" "true"
    set_config_value "fastfetch" "key_color" "34"
    set_config_value "fastfetch" "value_color" "default"
    
    # Re-apply current preset
    local preset=$(get_config_value "fastfetch" "preset")
    apply_preset "$preset"
    echo -e "\n\033[1;32m✓ Reset to theme colors!\033[0m"
}

# Preview callback for ASCII logos
preview_ascii_logo() {
    local selected=$1
    local logo_types=("pikachu" "tux" "arch" "cat" "neko" "bunny" "heart" "sakura" "naruto" "pokeball" "spider" "dragon" "creeper" "star" "rocket" "custom" "back")
    local logo_type="${logo_types[$selected]}"

    # Skip preview for custom and back options
    if [ "$logo_type" = "custom" ] || [ "$logo_type" = "back" ]; then
        return
    fi

    local logo_path="$HOME/.local/share/termfetch-studio/logos/${logo_type}.txt"

    if [ -f "$logo_path" ]; then
        # Display first 8 lines of the ASCII logo (limited height and width)
        head -n 8 "$logo_path" 2>/dev/null | cut -c1-40 || echo -e "\033[1;31mÖnizleme yüklenemedi\033[0m"
    else
        echo -e "\033[1;33mLogo dosyası bulunamadı: $logo_path\033[0m"
    fi
}

ascii_logo_submenu() {
    while true; do
        local current_logo_type=$(get_config_value "fastfetch" "logo_type")
        local logo_types=(pikachu tux arch cat neko bunny heart sakura naruto pokeball spider dragon creeper star rocket custom)
        
        local options=(
            "⚡ Pikachu ASCII"
            "🐧 Tux ASCII"
            "📐 Arch Logo ASCII"
            "🐱 Cat ASCII"
            "🎌 Neko ASCII"
            "🐰 Bunny ASCII"
            "💖 Heart ASCII"
            "🌸 Sakura ASCII"
            "🍜 Naruto ASCII"
            "⚾ Pokeball ASCII"
            "🕷️ Spider ASCII"
            "🐉 Dragon ASCII"
            "💚 Creeper ASCII"
            "⭐ Star ASCII"
            "🚀 Rocket ASCII"
            "🎨 Custom ASCII File"
            "← Back"
        )
        
        # Add indicators for current logo
        for i in "${!options[@]}"; do
            if [ $i -lt ${#logo_types[@]} ]; then
                if [ "${logo_types[$i]}" = "$current_logo_type" ]; then
                    options[$i]="${options[$i]} ✓"
                fi
            fi
        done

        select_option "📝 ASCII Logos [↑↓ Navigate, Enter to Apply, Q to Back]" "${options[@]}" preview_ascii_logo
        local choice=$?

        [ $choice -eq 16 ] || [ $choice -eq 255 ] && return

        case $choice in
            0)
                set_config_value "fastfetch" "logo_type" "pikachu"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            1)
                set_config_value "fastfetch" "logo_type" "tux"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            2)
                set_config_value "fastfetch" "logo_type" "arch"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            3)
                set_config_value "fastfetch" "logo_type" "cat"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            4)
                set_config_value "fastfetch" "logo_type" "neko"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            5)
                set_config_value "fastfetch" "logo_type" "bunny"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            6)
                set_config_value "fastfetch" "logo_type" "heart"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            7)
                set_config_value "fastfetch" "logo_type" "sakura"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            8)
                set_config_value "fastfetch" "logo_type" "naruto"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            9)
                set_config_value "fastfetch" "logo_type" "pokeball"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            10)
                set_config_value "fastfetch" "logo_type" "spider"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            11)
                set_config_value "fastfetch" "logo_type" "dragon"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            12)
                set_config_value "fastfetch" "logo_type" "creeper"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            13)
                set_config_value "fastfetch" "logo_type" "star"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            14)
                set_config_value "fastfetch" "logo_type" "rocket"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            15)
                show_header
                echo -e "\033[1;33m🎨 Custom ASCII File\033[0m"
                echo ""
                echo -ne "\033[1;36mEnter full path to ASCII file:\033[0m "
                read -r ascii_path
                if [ -f "$ascii_path" ]; then
                    cp "$ascii_path" "$LOGOS_DIR/custom.txt"
                    set_config_value "fastfetch" "logo_type" "custom"
                    set_config_value "fastfetch" "custom_image" ""
                    apply_logo_config
                    echo -e "\n\033[1;32m✓ Custom ASCII loaded!\033[0m"
                else
                    echo -e "\n\033[1;31m✗ File not found!\033[0m"
                fi
                sleep 2
                ;;
        esac
    done
}

# Preview callback for image logos
preview_image_logo() {
    local selected=$1

    # Access the preset_images array from parent scope
    local num_preset=${#PREVIEW_PRESET_IMAGES[@]}

    # Check if selected is a preset image
    if [ $selected -lt $num_preset ]; then
        local image_path="${PREVIEW_PRESET_IMAGES[$selected]}"

        if [ -f "$image_path" ]; then
            # Check terminal support
            local terminal_support=$(check_terminal_image_support)

            if [ "$terminal_support" = "kitty" ]; then
                # Use chafa for consistent, non-overlapping preview with smaller size
                if command -v chafa &>/dev/null; then
                    chafa -s 30x8 "$image_path" </dev/null 2>/dev/null || echo -e "\033[1;31mÖnizleme gösterilemiyor\033[0m"
                else
                    # Just show image info if chafa not available
                    echo -e "\033[1;36mDosya:\033[0m $(basename "$image_path")"
                    if command -v identify &>/dev/null; then
                        local dimensions=$(identify -format "%wx%h" "$image_path" 2>/dev/null)
                        [ -n "$dimensions" ] && echo -e "\033[1;36mBoyut:\033[0m $dimensions"
                    fi
                    local size=$(du -h "$image_path" 2>/dev/null | cut -f1)
                    [ -n "$size" ] && echo -e "\033[1;36mDosya Boyutu:\033[0m $size"
                fi
            elif command -v chafa &>/dev/null; then
                # Use chafa as fallback for ASCII art preview with smaller size
                chafa -s 30x8 "$image_path" </dev/null 2>/dev/null || echo -e "\033[1;31mÖnizleme gösterilemiyor\033[0m"
            else
                # Just show image info
                echo -e "\033[1;36mDosya:\033[0m $(basename "$image_path")"
                if command -v identify &>/dev/null; then
                    local dimensions=$(identify -format "%wx%h" "$image_path" 2>/dev/null)
                    [ -n "$dimensions" ] && echo -e "\033[1;36mBoyut:\033[0m $dimensions"
                fi
                local size=$(du -h "$image_path" 2>/dev/null | cut -f1)
                [ -n "$size" ] && echo -e "\033[1;36mDosya Boyutu:\033[0m $size"
            fi
        else
            echo -e "\033[1;31mDosya bulunamadı\033[0m"
        fi
    else
        # Custom image or other options - no preview
        return
    fi
}

image_logo_submenu() {
    while true; do
        local terminal_support=$(check_terminal_image_support)
        local preset_logos_dir="$HOME/.local/share/termfetch-studio/preset-logos"

        # Special check for Ubuntu - completely block image logos
        if [ "$terminal_support" = "none" ]; then
            log_debug "Image logos blocked - terminal not supported"
            log_terminal_info

            show_header
            echo -e "\033[1;31m🚫 IMAGE LOGOS COMPLETELY BLOCKED\033[0m"
            echo ""
            echo -e "\033[1;33mYour terminal: $TERM\033[0m"
            echo -e "\033[1;31mImage logos are DISABLED for your safety.\033[0m"
            echo ""
            echo -e "\033[1;32m✅ Using safe ASCII fallback instead\033[0m"
            echo ""
            echo -e "\033[1;36m💡 To use image logos, switch to:\033[0m"
            echo -e "  • Kitty Terminal"
            echo -e "  • WezTerm"
            echo ""
            echo -e "\033[1;33mPress any key to continue...\033[0m"
            read -n 1 -s

            # Force ASCII fallback
            set_config_value "fastfetch" "logo_type" "auto"
            set_config_value "fastfetch" "custom_image" ""
            apply_logo_config
            log_debug "ASCII fallback applied"
            return
        fi

        # Build options array starting with preset images
        local options=()
        declare -g -a PREVIEW_PRESET_IMAGES=()

        # Scan for preset images if directory exists
        if [ -d "$preset_logos_dir" ]; then
            while IFS= read -r -d '' image; do
                local basename=$(basename "$image")
                PREVIEW_PRESET_IMAGES+=("$image")
                options+=("🖼️  $basename")
            done < <(find "$preset_logos_dir" -type f \( -iname "*.png" -o -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.gif" -o -iname "*.bmp" \) -print0 2>/dev/null | sort -z)
        fi

        # Add custom image option
        if [ "$terminal_support" != "none" ]; then
            options+=("🎨 Custom Image (Terminal Supports)")
        else
            options+=("🎨 Custom Image (Not Supported)")
        fi

        # Add safe fallback options for unsupported terminals
        if [ "$terminal_support" = "none" ]; then
            options+=("🛡️  Safe ASCII Fallback")
            options+=("🔄 Auto Fallback (Recommended)")
        fi

        options+=("← Back")

        select_option "🖼️  Image Logos (Terminal: $terminal_support) [↑↓ Navigate, Enter to Apply, Q to Back]" "${options[@]}" preview_image_logo
        local choice=$?

        local num_preset=${#PREVIEW_PRESET_IMAGES[@]}
        local num_custom=1
        local num_fallback=0
        if [ "$terminal_support" = "none" ]; then
            num_fallback=2
        fi
        local total_options=$((num_preset + num_custom + num_fallback))
        local back_index=$total_options

        [ $choice -eq $back_index ] || [ $choice -eq 255 ] && return

        # Handle preset image selection
        if [ $choice -lt $num_preset ]; then
            # Enhanced warning for terminals that don't support images
            if [ "$terminal_support" = "none" ]; then
                show_header
                echo -e "\033[1;31m🚫 IMAGE LOGOS BLOCKED - TERMINAL NOT SUPPORTED\033[0m"
                echo ""
                echo -e "\033[1;33mYour terminal: $TERM\033[0m"
                echo -e "\033[1;31mYour terminal does NOT support image protocols.\033[0m"
                echo ""
                echo -e "\033[1;31m❌ Image logos are DISABLED for your safety:\033[0m"
                echo -e "  • Prevents terminal freezing"
                echo -e "  • Prevents blank screen issues"
                echo -e "  • Prevents need for Ctrl+C recovery"
                echo ""
                echo -e "\033[1;32m✅ Using safe ASCII fallback instead\033[0m"
                echo ""
                echo -e "\033[1;36m💡 To use image logos, switch to:\033[0m"
                echo -e "  • Kitty Terminal"
                echo -e "  • WezTerm"
                echo ""
                echo -e "\033[1;33mPress any key to continue...\033[0m"
                read -n 1 -s

                # Force ASCII fallback
                set_config_value "fastfetch" "logo_type" "auto"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                continue
            fi

            local selected_image="${PREVIEW_PRESET_IMAGES[$choice]}"
            # Create unique target name per selection to avoid Konsole caching same name
            local stamp=$(date +%s)
            local target_image="$IMAGES_DIR/preset_logo_${stamp}.png"

            # Clean old preset_logo_* files to keep only latest
            rm -f "$IMAGES_DIR"/preset_logo_*.png 2>/dev/null || true

            # Get file extension
            local ext="${selected_image##*.}"
            local ext_lower=$(echo "$ext" | tr '[:upper:]' '[:lower:]')

            # Convert to PNG if needed, preserving transparency
            if [ "$ext_lower" != "png" ]; then
                if command -v convert &> /dev/null; then
                    convert "$selected_image" PNG32:"$target_image" 2>/dev/null
                else
                    cp "$selected_image" "$IMAGES_DIR/preset_logo.$ext_lower"
                    target_image="$IMAGES_DIR/preset_logo.$ext_lower"
                fi
            else
                cp "$selected_image" "$target_image"
            fi

            # Resize if too large
            if command -v convert &> /dev/null && [ -f "$target_image" ]; then
                local img_width=$(identify -format "%w" "$target_image" 2>/dev/null)
                if [ -n "$img_width" ] && [ "$img_width" -gt 800 ]; then
                    convert "$target_image" -resize 800x PNG32:"$target_image" 2>/dev/null
                fi
            fi

            set_config_value "fastfetch" "logo_type" "image"
            set_config_value "fastfetch" "custom_image" "$target_image"
            apply_logo_config

        # Handle custom image
        elif [ $choice -eq $num_preset ]; then
            if [ "$terminal_support" = "none" ]; then
                # Block custom image for unsupported terminals
                show_header
                echo -e "\033[1;31m🚫 IMAGE LOGOS BLOCKED - TERMINAL NOT SUPPORTED\033[0m"
                echo ""
                echo -e "\033[1;33mYour terminal: $TERM\033[0m"
                echo -e "\033[1;31mCustom image logos are DISABLED for your safety.\033[0m"
                echo ""
                echo -e "\033[1;32m✅ Using safe ASCII fallback instead\033[0m"
                echo ""
                echo -e "\033[1;33mPress any key to continue...\033[0m"
                read -n 1 -s
                
                # Force ASCII fallback
                set_config_value "fastfetch" "logo_type" "auto"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                continue
            elif [ "$terminal_support" != "none" ]; then
                show_header
                echo -e "\033[1;33m🖼️  Custom Image\033[0m"
                echo ""
                echo -e "\033[1;36mSupported formats: PNG, JPG, JPEG, GIF, BMP\033[0m"
                echo -ne "\033[1;36mEnter full path to image file:\033[0m "
                read -r image_path

                # Expand tilde to home directory
                image_path="${image_path/#\~/$HOME}"

                if [ -f "$image_path" ]; then
                    # Get file extension
                    ext="${image_path##*.}"
                    ext_lower=$(echo "$ext" | tr '[:upper:]' '[:lower:]')

                    # Convert to PNG preserving transparency
                    if [ "$ext_lower" != "png" ]; then
                        if command -v convert &> /dev/null; then
                            echo -e "\033[1;33m🔄 Converting $ext_lower to PNG...\033[0m"
                            convert "$image_path" PNG32:"$IMAGES_DIR/custom_logo.png" 2>/dev/null
                            if [ $? -eq 0 ]; then
                                echo -e "\033[1;32m✓ Image converted successfully!\033[0m"
                            else
                                echo -e "\033[1;31m✗ Conversion failed, copying original...\033[0m"
                                cp "$image_path" "$IMAGES_DIR/custom_logo.$ext_lower"
                                set_config_value "fastfetch" "logo_type" "image"
                                set_config_value "fastfetch" "custom_image" "$IMAGES_DIR/custom_logo.$ext_lower"
                                apply_logo_config
                                continue
                            fi
                        else
                            echo -e "\033[1;33m⚠️  ImageMagick not found, copying as-is...\033[0m"
                            cp "$image_path" "$IMAGES_DIR/custom_logo.$ext_lower"
                            set_config_value "fastfetch" "logo_type" "image"
                            set_config_value "fastfetch" "custom_image" "$IMAGES_DIR/custom_logo.$ext_lower"
                            apply_logo_config
                            continue
                        fi
                    else
                        echo -e "\033[1;36mCopying PNG image...\033[0m"
                        # Just copy the PNG as-is to preserve transparency
                        cp "$image_path" "$IMAGES_DIR/custom_logo.png"
                    fi

                    # Resize if image is too large (preserve transparency)
                    if command -v convert &> /dev/null && [ -f "$IMAGES_DIR/custom_logo.png" ]; then
                        # Get image dimensions
                        img_width=$(identify -format "%w" "$IMAGES_DIR/custom_logo.png" 2>/dev/null)
                        if [ -n "$img_width" ] && [ "$img_width" -gt 800 ]; then
                            echo -e "\033[1;33m📐 Resizing large image...\033[0m"
                            convert "$IMAGES_DIR/custom_logo.png" -resize 800x PNG32:"$IMAGES_DIR/custom_logo.png" 2>/dev/null
                        fi
                    fi

                    set_config_value "fastfetch" "logo_type" "image"
                    set_config_value "fastfetch" "custom_image" "$IMAGES_DIR/custom_logo.png"
                    apply_logo_config
                    echo -e "\n\033[1;32m✓ Custom image loaded!\033[0m"
                else
                    echo -e "\n\033[1;31m✗ File not found: $image_path\033[0m"
                    echo -e "\033[1;33mPlease check the path and try again.\033[0m"
                fi
            else
                echo -e "\n\033[1;33m⚠️  Your terminal doesn't support images\033[0m"
                echo -e "ASCII art will be used instead"
                echo ""
                echo -e "\033[1;36mTip: Install 'chafa' for ASCII art conversion:\033[0m"
                echo -e "  \033[1;32msudo pacman -S chafa\033[0m   # Arch"
                echo -e "  \033[1;32msudo apt install chafa\033[0m # Debian/Ubuntu"
            fi
            sleep 2
        # Handle fallback options for unsupported terminals
        elif [ $choice -eq $((num_preset + num_custom)) ] && [ "$terminal_support" = "none" ]; then
            # Safe ASCII Fallback
            set_config_value "fastfetch" "logo_type" "auto"
            set_config_value "fastfetch" "custom_image" ""
            apply_logo_config
            echo -e "\033[1;32m✓ Safe ASCII fallback enabled!\033[0m"
        elif [ $choice -eq $((num_preset + num_custom + 1)) ] && [ "$terminal_support" = "none" ]; then
            # Auto Fallback (Recommended)
            set_config_value "fastfetch" "logo_type" "auto"
            set_config_value "fastfetch" "custom_image" ""
            set_config_value "fastfetch" "logo_fallback" "auto"
            apply_logo_config
            echo -e "\033[1;32m✓ Auto fallback enabled (recommended)!\033[0m"
        fi
    done
}

logo_menu() {
    while true; do
        local current=$(get_config_value "fastfetch" "logo_type")
        local custom_image=$(get_config_value "fastfetch" "custom_image")
        local terminal_support=$(check_terminal_image_support)
        local fallback=$(get_config_value "fastfetch" "logo_fallback")
        [ -z "$fallback" ] && fallback="auto"

        local options=(
            "🐧 Auto (Distribution Logo)"
            "📝 ASCII Logos"
            "🖼️ Image Logos"
            "🔄 Image Fallback (Unsupported Terminals): $fallback"
            "❌ No Logo"
            "← Back"
        )

        select_option "🖼️  Logo Settings (Current: $current | Terminal: $terminal_support)" "${options[@]}"
        local choice=$?

        [ $choice -eq 5 ] || [ $choice -eq 255 ] && return

        case $choice in
            0)
                set_config_value "fastfetch" "logo_type" "auto"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
            1)
                ascii_logo_submenu
                ;;
            2)
                image_logo_submenu
                ;;
            3)
                # Image fallback submenu
                local fallback_options=(
                    "🐧 Auto (Distribution Logo)"
                    "📝 ASCII Logo"
                    "❌ No Logo"
                    "← Back"
                )
                select_option "🔄 Fallback for Unsupported Terminals" "${fallback_options[@]}"
                local fb_choice=$?

                case $fb_choice in
                    0) set_config_value "fastfetch" "logo_fallback" "auto" ;;
                    1) set_config_value "fastfetch" "logo_fallback" "ascii" ;;
                    2) set_config_value "fastfetch" "logo_fallback" "none" ;;
                esac
                ;;
            4)
                set_config_value "fastfetch" "logo_type" "none"
                set_config_value "fastfetch" "custom_image" ""
                apply_logo_config
                ;;
        esac
    done
}

apply_logo_config() {
    local logo_type=$(get_config_value "fastfetch" "logo_type")
    local custom_image=$(get_config_value "fastfetch" "custom_image")
    
    echo ""
    echo -e "\033[1;36m🖼️  Applying logo configuration...\033[0m"
    
    # Regenerate fastfetch config with new logo
    local preset=$(get_config_value "fastfetch" "preset")
    local theme=$(get_config_value "colors" "theme")
    
    case $preset in
        "full") create_full_config "$logo_type" "$custom_image" "$theme" ;;
        "minimal") create_minimal_config "$logo_type" "$custom_image" "$theme" ;;
        "focused") create_focused_config "$logo_type" "$custom_image" "$theme" ;;
        "developer") create_developer_config "$logo_type" "$custom_image" "$theme" ;;
        "gaming") create_gaming_config "$logo_type" "$custom_image" "$theme" ;;
        *) create_full_config "$logo_type" "$custom_image" "$theme" ;;
    esac
    
    # Cleanup: remove invalid/empty source fields for auto/builtin
    if [ -f "$CONFIG_DIR/fastfetch.jsonc" ]; then
        sed -i -E '/"source"\s*:\s*""/d; /"source"\s*:\s*"auto"/d' "$CONFIG_DIR/fastfetch.jsonc" 2>/dev/null || true
    fi

    echo -e "\033[1;32m✓ Logo applied!\033[0m"
}

theme_menu() {
    while true; do
        local current=$(get_config_value "colors" "theme")
        
        # Always apply current theme when entering menu
        apply_terminal_colors "$current" 2>/dev/null || true

        local options=(
            "🦇 Dracula"
            "❄️ Nord"
            "🍂 Gruvbox Dark"
            "🌃 Tokyo Night"
            "🌑 One Dark"
            "🌊 Oceanic Next"
            "⚫ Monochrome"
            "☀️ Gruvbox Light"
            "🌅 Solarized Light"
            "🌞 One Light"
            "🏖️ Ayu Light"
            "🌈 Synthwave"
            "🎨 Monokai Pro"
            "🔮 Palenight"
            "🌸 Sakura"
            "💜 Lavender Dream"
            "🍬 Candy"
            "💚 Matrix"
            "🌆 Cyberpunk 2077"
            "🍜 Naruto"
            "⚡ Pokemon"
            "🕷️ Spider-Man"
            "👹 Doom"
            "🔥 Valorant"
            "⛏️ Minecraft"
            "← Back"
        )

        # Add indicators for current theme
        local themes=(dracula nord gruvbox-dark tokyo-night one-dark oceanic-next monochrome
                      gruvbox-light solarized-light one-light ayu-light synthwave monokai-pro palenight sakura lavender candy
                      matrix cyberpunk naruto pokemon spiderman doom valorant minecraft)

        # Add visual indicators to options
        for i in "${!options[@]}"; do
            if [ $i -lt ${#themes[@]} ]; then
                if [ "${themes[$i]}" = "$current" ]; then
                    options[$i]="${options[$i]} ✓"
                fi
            fi
        done

        select_option_2col_with_preview "🎨 Color Themes (Current: $current) [↑↓←→ Navigate, Enter to Apply, Q to Back]" "${options[@]}" preview_theme
        local choice=$?

        if [ $choice -eq 25 ] || [ $choice -eq 255 ]; then
            # Back button - apply current theme and return to main menu
            apply_terminal_colors "$current" 2>/dev/null || true
            return
        fi

        if [ $choice -ge 0 ] && [ $choice -le 25 ]; then
            apply_theme "${themes[$choice]}"
        fi
    done
}

fastfetch_menu() {
    while true; do
        local current=$(get_config_value "fastfetch" "preset")
        local icons=$(get_config_value "fastfetch" "icons")
        local labels=$(get_config_value "fastfetch" "labels")
        
        local presets=(full minimal focused developer gaming custom)
        
        local options=(
            "📋 Full Info"
            "📝 Minimal"
            "🎯 Focused"
            "💻 Developer"
            "🎮 Gaming"
            "🧩 Custom Preset"
            "🛠️ Configure Custom Preset"
            "🔄 Reset to Default"
            "← Back"
        )
        
        # Add indicators for current preset (only for the first 6 options which are presets)
        for i in "${!options[@]}"; do
            if [ $i -lt 6 ] && [ $i -lt ${#presets[@]} ]; then
                if [ "${presets[$i]}" = "$current" ]; then
                    options[$i]="${options[$i]} ✓"
                fi
            fi
        done

        select_option "📊 Fastfetch Presets (Current: $current | Icons: $icons) [↑↓ Navigate, Enter to Apply, Q to Back]" "${options[@]}" preview_fastfetch_preset
        local choice=$?
        
        [ $choice -eq 8 ] || [ $choice -eq 255 ] && return

        case $choice in
            0) apply_preset "full" ;;
            1) apply_preset "minimal" ;;
            2) apply_preset "focused" ;;
            3) apply_preset "developer" ;;
            4) apply_preset "gaming" ;;
            5) apply_preset "custom" ;;
            6) configure_custom_preset ;;
            7) reset_fastfetch ;;
        esac
    done
}

settings_menu() {
    while true; do
        local options=(
            "🎨 Color Settings"
            "🔧 Edit Config File"
            "🗑️ Reset All Settings"
            "← Back"
        )

        select_option "⚙️  Advanced Settings" "${options[@]}"
        local choice=$?

        [ $choice -eq 3 ] || [ $choice -eq 255 ] && return

        case $choice in
            0) color_settings_menu ;;
            1) edit_config_file ;;
            2) reset_all_settings ;;
        esac
    done
}

# Color settings menu for customizing fastfetch colors
color_settings_menu() {
    while true; do
        local current_key_color=$(get_config_value "fastfetch" "key_color")
        local current_value_color=$(get_config_value "fastfetch" "value_color")
        local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")

        [ -z "$use_theme_colors" ] && use_theme_colors="true"
        [ -z "$current_key_color" ] && current_key_color="theme"
        [ -z "$current_value_color" ] && current_value_color="theme"

        local mode_text="Theme Colors (Auto)"
        if [ "$use_theme_colors" = "false" ]; then
            mode_text="Custom Colors"
        fi

        local options=(
            "🌈 Color Mode: $mode_text"
            "🔑 Icon Color (keyColor): $current_key_color"
            "📝 Text Color (outputColor): $current_value_color"
            "🎨 Preview Colors"
            "🔄 Reset to Theme Colors"
            "← Back"
        )

        select_option "🎨 Icon & Text Color Settings" "${options[@]}"
        local choice=$?

        [ $choice -eq 5 ] || [ $choice -eq 255 ] && return

        case $choice in
            0) toggle_color_mode ;;
            1) set_icon_color ;;
            2) set_text_color ;;
            3) preview_colors ;;
            4) reset_to_theme_colors ;;
        esac
    done
}

# Toggle between theme colors and custom colors
toggle_color_mode() {
    local current=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$current" ] && current="true"

    if [ "$current" = "true" ]; then
        set_config_value "fastfetch" "use_theme_colors" "false"
        echo -e "\033[1;33m🎨 Switched to Custom Colors mode\033[0m"
        echo -e "\033[1;36mYou can now set custom icon and text colors\033[0m"
    else
        set_config_value "fastfetch" "use_theme_colors" "true"
        set_config_value "fastfetch" "key_color" "theme"
        set_config_value "fastfetch" "value_color" "theme"
        apply_color_changes
        echo -e "\033[1;33m🌈 Switched to Theme Colors mode\033[0m"
        echo -e "\033[1;36mColors will adapt to your selected theme\033[0m"
    fi
    sleep 2
}

# Set icon color (keyColor in fastfetch)
set_icon_color() {
    local use_theme=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme" ] && use_theme="true"

    if [ "$use_theme" = "true" ]; then
        echo -e "\033[1;33m⚠️  Theme Colors mode is active\033[0m"
        echo -e "\033[1;36mSwitch to Custom Colors mode first to set custom icon colors\033[0m"
        sleep 2
        return
    fi

    local colors=("31" "32" "33" "34" "35" "36" "37" "90" "91" "92" "93" "94" "95" "96" "97")
    local color_names=("Red" "Green" "Yellow" "Blue" "Magenta" "Cyan" "White" "Bright Black" "Bright Red" "Bright Green" "Bright Yellow" "Bright Blue" "Bright Magenta" "Bright Cyan" "Bright White")

    local options=()
    for i in "${!colors[@]}"; do
        options+=("\033[1;${colors[$i]}m● ${color_names[$i]}\033[0m")
    done
    options+=("← Back")

    select_option "🔑 Select Icon Color" "${options[@]}"
    local choice=$?

    [ $choice -eq ${#colors[@]} ] || [ $choice -eq 255 ] && return

    set_config_value "fastfetch" "key_color" "${colors[$choice]}"
    apply_color_changes
    echo -e "\033[1;32m✓ Icon color set to ${color_names[$choice]}\033[0m"
}

# Set text color (outputColor in fastfetch)
set_text_color() {
    local use_theme=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme" ] && use_theme="true"

    if [ "$use_theme" = "true" ]; then
        echo -e "\033[1;33m⚠️  Theme Colors mode is active\033[0m"
        echo -e "\033[1;36mSwitch to Custom Colors mode first to set custom text colors\033[0m"
        sleep 2
        return
    fi

    local colors=("default" "31" "32" "33" "34" "35" "36" "37" "90" "91" "92" "93" "94" "95" "96" "97")
    local color_names=("Default" "Red" "Green" "Yellow" "Blue" "Magenta" "Cyan" "White" "Bright Black" "Bright Red" "Bright Green" "Bright Yellow" "Bright Blue" "Bright Magenta" "Bright Cyan" "Bright White")

    local options=()
    for i in "${!colors[@]}"; do
        if [ "${colors[$i]}" = "default" ]; then
            options+=("● ${color_names[$i]}")
        else
            options+=("\033[1;${colors[$i]}m● ${color_names[$i]}\033[0m")
        fi
    done
    options+=("← Back")

    select_option "📝 Select Text Color" "${options[@]}"
    local choice=$?

    [ $choice -eq ${#colors[@]} ] || [ $choice -eq 255 ] && return

    set_config_value "fastfetch" "value_color" "${colors[$choice]}"
    apply_color_changes
    echo -e "\033[1;32m✓ Text color set to ${color_names[$choice]}\033[0m"
}

# Reset to theme colors
reset_to_theme_colors() {
    set_config_value "fastfetch" "use_theme_colors" "true"
    set_config_value "fastfetch" "key_color" "theme"
    set_config_value "fastfetch" "value_color" "theme"
    apply_color_changes
    echo -e "\033[1;32m✓ Reset to theme-adaptive colors!\033[0m"
}

# Preview colors
preview_colors() {
    clear
    echo ""
    echo -e "\033[1;36m🎨 Previewing current fastfetch configuration...\033[0m"
    echo ""

    # Show actual fastfetch preview
    fastfetch -c "$CONFIG_DIR/fastfetch.jsonc" 2>/dev/null

    echo ""
    echo -e "\033[1;33mPress any key to continue...\033[0m"
    read -n 1 -s
}

# Apply color changes to current configuration
apply_color_changes() {
    local preset=$(get_config_value "fastfetch" "preset")
    [ -z "$preset" ] && preset="full"
    
    case $preset in
        "full") create_full_config "$(get_config_value fastfetch logo_type)" "$(get_config_value fastfetch custom_image)" "$(get_config_value colors theme)" ;;
        "minimal") create_minimal_config "$(get_config_value fastfetch logo_type)" "$(get_config_value fastfetch custom_image)" "$(get_config_value colors theme)" ;;
        "focused") create_focused_config "$(get_config_value fastfetch logo_type)" "$(get_config_value fastfetch custom_image)" "$(get_config_value colors theme)" ;;
        "developer") create_developer_config "$(get_config_value fastfetch logo_type)" "$(get_config_value fastfetch custom_image)" "$(get_config_value colors theme)" ;;
        "gaming") create_gaming_config "$(get_config_value fastfetch logo_type)" "$(get_config_value fastfetch custom_image)" "$(get_config_value colors theme)" ;;
        "custom") create_custom_config "$(get_config_value fastfetch logo_type)" "$(get_config_value fastfetch custom_image)" "$(get_config_value colors theme)" ;;
    esac
}

backup_menu() {
    while true; do
        local options=(
            "💾 Create Backup"
            "📂 Restore Backup"
            "📋 List Backups"
            "← Back"
        )
        
        select_option "💾 Backup & Restore" "${options[@]}"
        local choice=$?
        
        [ $choice -eq 3 ] || [ $choice -eq 255 ] && return
        
        case $choice in
            0) create_backup ;;
            1) restore_backup ;;
            2) list_backups ;;
        esac
    done
}

apply_theme() {
    local theme=$1

    # Save theme to config immediately
    set_config_value "colors" "theme" "$theme"

    # Regenerate fastfetch config with new theme
    local preset=$(get_config_value "fastfetch" "preset")
    local logo_type=$(get_config_value "fastfetch" "logo_type")
    local custom_image=$(get_config_value "fastfetch" "custom_image")

    # Set defaults if values are empty
    [ -z "$preset" ] && preset="full"
    [ -z "$logo_type" ] && logo_type="auto"

    # Save preset to config if it was not set
    set_config_value "fastfetch" "preset" "$preset"
    set_config_value "fastfetch" "logo_type" "$logo_type"

    case $preset in
        "full") create_full_config "$logo_type" "$custom_image" "$theme" ;;
        "minimal") create_minimal_config "$logo_type" "$custom_image" "$theme" ;;
        "focused") create_focused_config "$logo_type" "$custom_image" "$theme" ;;
        "developer") create_developer_config "$logo_type" "$custom_image" "$theme" ;;
        "gaming") create_gaming_config "$logo_type" "$custom_image" "$theme" ;;
        "custom") create_custom_config "$logo_type" "$custom_image" "$theme" ;;
        *) create_full_config "$logo_type" "$custom_image" "$theme" ;;
    esac

    # Save theme to a sourced file for persistence
    mkdir -p "$CONFIG_DIR"
    cat > "$CONFIG_DIR/theme.sh" << THEMESCRIPT
#!/bin/bash
# TermFetch Studio Theme - Auto-generated
# Theme: $theme
# This file is sourced on terminal startup

# Apply terminal colors (if supported by your terminal)
THEMESCRIPT

    # Append the theme color function (for terminals that support it)
    declare -f apply_terminal_colors >> "$CONFIG_DIR/theme.sh"
    echo "apply_terminal_colors '$theme' 2>/dev/null || true" >> "$CONFIG_DIR/theme.sh"

    # Try to apply terminal colors (will silently fail on unsupported terminals)
    apply_terminal_colors "$theme" 2>/dev/null || true

    # Clear screen and show new theme
    clear

    # Re-run fastfetch to show the new theme immediately
    if command -v fastfetch &> /dev/null; then
        fastfetch --config "$CONFIG_DIR/fastfetch.jsonc" 2>/dev/null || true
    fi
}

apply_preset() {
    local preset=$1
    echo ""
    echo -e "\033[1;36m📊 Applying $preset preset...\033[0m"
    
    local logo_type=$(get_config_value "fastfetch" "logo_type")
    local custom_image=$(get_config_value "fastfetch" "custom_image")
    local theme=$(get_config_value "colors" "theme")
    
    case $preset in
        "full") create_full_config "$logo_type" "$custom_image" "$theme" ;;
        "minimal") create_minimal_config "$logo_type" "$custom_image" "$theme" ;;
        "focused") create_focused_config "$logo_type" "$custom_image" "$theme" ;;
        "developer") create_developer_config "$logo_type" "$custom_image" "$theme" ;;
        "gaming") create_gaming_config "$logo_type" "$custom_image" "$theme" ;;
        "custom") create_custom_config "$logo_type" "$custom_image" "$theme" ;;
    esac
    
    set_config_value "fastfetch" "preset" "$preset"
    
    echo -e "\033[1;32m✓ Preset applied!\033[0m"
}

toggle_icons() {
    local current=$(get_config_value "fastfetch" "icons")
    if [ "$current" = "true" ]; then
        set_config_value "fastfetch" "icons" "false"
        echo ""
        echo -e "\033[1;31m🎭 Icons disabled\033[0m"
    else
        set_config_value "fastfetch" "icons" "true"
        echo ""
        echo -e "\033[1;32m🎭 Icons enabled\033[0m"
    fi
    
    # Re-apply current preset to update icons
    local preset=$(get_config_value "fastfetch" "preset")
    apply_preset "$preset"
}

reset_fastfetch() {
    echo ""
    echo -e "\033[1;33m🔄 Resetting fastfetch configuration...\033[0m"
    
    set_config_value "fastfetch" "preset" "full"
    set_config_value "fastfetch" "icons" "true"
    set_config_value "fastfetch" "logo_type" "auto"
    set_config_value "fastfetch" "custom_image" ""
    
    local theme=$(get_config_value "colors" "theme")
    create_full_config "auto" "" "$theme"
    
    echo -e "\033[1;32m✓ Fastfetch reset to defaults!\033[0m"
    sleep 1
}

reset_all_settings() {
    echo ""
    echo -e "\033[1;31m⚠️  This will reset ALL settings to defaults!\033[0m"
    echo -ne "\033[1;33mAre you sure? (y/N): \033[0m"
    read -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        echo -e "\033[1;33m🗑️  Resetting all settings...\033[0m"
        
        rm -f "$CONFIG_FILE"
        rm -f "$CONFIG_DIR/fastfetch.jsonc"
        rm -rf "$IMAGES_DIR"
        rm -rf "$LOGOS_DIR"
        
        mkdir -p "$IMAGES_DIR"
        mkdir -p "$LOGOS_DIR"
        
        init_config
        
        echo -e "\033[1;32m✓ All settings reset!\033[0m"
        sleep 1
    fi
}

edit_config_file() {
    if command -v nano &> /dev/null; then
        nano "$CONFIG_FILE"
    elif command -v vim &> /dev/null; then
        vim "$CONFIG_FILE"
    elif command -v vi &> /dev/null; then
        vi "$CONFIG_FILE"
    else
        echo -e "\033[1;31m✗ No text editor found!\033[0m"
        sleep 1
    fi
}

preview_setup() {
    echo ""
    echo -e "\033[1;36m👁️  Previewing current setup...\033[0m"
    echo ""
    
    # Apply current theme colors
    local current_theme=$(get_config_value "colors" "theme")
    apply_terminal_colors "$current_theme"
    
    # Show fastfetch
    fastfetch -c "$CONFIG_DIR/fastfetch.jsonc"
    
    echo ""
    echo -e "\033[1;33mPress any key to continue...\033[0m"
    read -n 1 -s
}

help_info_menu() {
    show_header
    echo -e "\033[1;33m❓ Help & Information\033[0m"
    echo ""

    echo -e "\033[1;36m📚 Quick Commands:\033[0m"
    echo -e "  \033[1;32mtermfetch-studio\033[0m           # Start interactive menu"
    echo -e "  \033[1;32mtermfetch-studio --preview\033[0m # Quick preview current setup"
    echo -e "  \033[1;32mtermfetch-studio --help\033[0m    # Show help message"
    echo -e "  \033[1;32mtermfetch-studio --backup\033[0m  # Create configuration backup"
    echo -e "  \033[1;32mtermfetch-studio --restore\033[0m # Restore from backup"
    echo -e "  \033[1;32mtermfetch-studio uninstall\033[0m # Uninstall application"
    echo ""

    echo -e "\033[1;36m🎨 Features:\033[0m"
    echo -e "  \033[1;34m•\033[0m 25+ color themes with theme-adaptive colors"
    echo -e "  \033[1;34m•\033[0m 6 fastfetch presets (Full, Minimal, Focused, Developer, Gaming, Custom)"
    echo -e "  \033[1;34m•\033[0m Custom logo support (ASCII, Images, None)"
    echo -e "  \033[1;34m•\033[0m Customizable icon and text colors"
    echo -e "  \033[1;34m•\033[0m Live preview and easy configuration"
    echo -e "  \033[1;34m•\033[0m Backup & restore system"
    echo ""

    echo -e "\033[1;36m📍 File Locations:\033[0m"
    echo -e "  \033[1;34mConfig:\033[0m ~/.config/termfetch-studio/"
    echo -e "  \033[1;34mData:\033[0m ~/.local/share/termfetch-studio/"
    echo -e "  \033[1;34mBinary:\033[0m ~/.local/bin/termfetch-studio"
    echo ""

    echo -e "\033[1;36m⌨️  Navigation:\033[0m"
    echo -e "  \033[1;34m↑↓\033[0m      Move up/down in menus"
    echo -e "  \033[1;34mEnter\033[0m   Select option"
    echo -e "  \033[1;34mQ\033[0m       Go back / Quit"
    echo ""

    echo -e "\033[1;36m🎨 Color Settings:\033[0m"
    echo -e "  \033[1;34m•\033[0m Theme Colors: Icons and text adapt to selected theme"
    echo -e "  \033[1;34m•\033[0m Custom Colors: Set your own icon and text colors"
    echo -e "  \033[1;34m•\033[0m Toggle between modes in Advanced Settings → Color Settings"
    echo ""

    echo -e "\033[1;36m🐛 Troubleshooting:\033[0m"
    echo -e "  \033[1;34m•\033[0m Command not found? Add ~/.local/bin to PATH:"
    echo -e "    \033[1;32mexport PATH=\"\$HOME/.local/bin:\$PATH\"\033[0m"
    echo -e "  \033[1;34m•\033[0m Icons not showing? Install a Nerd Font"
    echo -e "  \033[1;34m•\033[0m Colors broken? Check terminal ANSI color support"
    echo ""

    echo -e "\033[1;36m🔗 Resources:\033[0m"
    echo -e "  \033[1;34mGitHub:\033[0m https://github.com/sinansarikaya/termfetch-studio"
    echo -e "  \033[1;34mVersion:\033[0m 1.0.0"
    echo ""

    echo -e "\033[1;33mPress any key to return to menu...\033[0m"
    read -n 1 -s
}

system_info() {
    show_header
    echo -e "\033[1;33mℹ️  System Information\033[0m"
    echo ""
    
    echo -e "\033[1;36mSystem:\033[0m"
    echo -e "  \033[1;34mOS:\033[0m $(lsb_release -d 2>/dev/null | cut -f2 || uname -o)"
    echo -e "  \033[1;34mKernel:\033[0m $(uname -r)"
    echo -e "  \033[1;34mArchitecture:\033[0m $(uname -m)"
    echo -e "  \033[1;34mHostname:\033[0m $(hostname)"
    
    echo ""
    echo -e "\033[1;36mTerminal:\033[0m"
    echo -e "  \033[1;34mTerminal:\033[0m $TERM"
    echo -e "  \033[1;34mTerminal Program:\033[0m $TERM_PROGRAM"
    echo -e "  \033[1;34mShell:\033[0m $SHELL"
    echo -e "  \033[1;34mImage Support:\033[0m $(check_terminal_image_support)"
    
    echo ""
    echo -e "\033[1;36mEnvironment:\033[0m"
    echo -e "  \033[1;34mDisplay:\033[0m $DISPLAY"
    echo -e "  \033[1;34mWayland Display:\033[0m $WAYLAND_DISPLAY"
    echo -e "  \033[1;34mHyprland Instance:\033[0m $HYPRLAND_INSTANCE_SIGNATURE"
    
    echo ""
    echo -e "\033[1;36mTermFetch Studio:\033[0m"
    echo -e "  \033[1;34mConfig Dir:\033[0m $CONFIG_DIR"
    echo -e "  \033[1;34mInstall Dir:\033[0m $INSTALL_DIR"
    echo -e "  \033[1;34mCurrent Theme:\033[0m $(get_config_value "colors" "theme")"
    echo -e "  \033[1;34mCurrent Preset:\033[0m $(get_config_value "fastfetch" "preset")"
    
    echo ""
    echo -e "\033[1;36mImage Diagnostics:\033[0m"
    local terminal_support=$(check_terminal_image_support)
    case $terminal_support in
        "kitty")
            echo -e "  \033[1;32m✓ Kitty detected - Full image support available\033[0m"
            ;;
        "wezterm")
            echo -e "  \033[1;32m✓ WezTerm detected - Full image support available\033[0m"
            ;;
        "foot")
            echo -e "  \033[1;33m⚠ Foot detected - Limited image support (sixel)\033[0m"
            ;;
        "konsole")
            echo -e "  \033[1;33m⚠ Konsole detected - Limited image support\033[0m"
            ;;
        "iterm2")
            echo -e "  \033[1;32m✓ iTerm2 detected - Full image support available\033[0m"
            ;;
        "none")
            echo -e "  \033[1;31m✗ No image support detected\033[0m"
            echo -e "  \033[1;33m💡 Try using Kitty, WezTerm, or Foot for image support\033[0m"
            if [ -n "$WAYLAND_DISPLAY" ] || [ -n "$HYPRLAND_INSTANCE_SIGNATURE" ]; then
                echo -e "  \033[1;33m💡 You're in Wayland/Hyprland - ensure your terminal supports image protocols\033[0m"
            fi
            ;;
    esac
    
    echo ""
    echo -e "\033[1;33mPress any key to continue...\033[0m"
    read -n 1 -s
}

setup_wizard() {
    show_header
    echo -e "\033[1;32m🎉 Welcome to TermFetch Studio Setup Wizard! 🎉\033[0m"
    echo ""
    
    local options=(
        "🌙 Dark & Modern (Dracula + Full)"
        "☀️ Light & Clean (Solarized + Minimal)"
        "🎨 Colorful & Vibrant (Synthwave + Full)"
        "⚫ Minimal & Simple (Monochrome + Minimal)"
        "🎮 Gaming Setup (One Dark + Gaming)"
        "💻 Developer Setup (Nord + Developer)"
        "⏭️ Skip Setup"
    )
    
    select_option "🎨 Choose your preferred setup style:" "${options[@]}"
    local choice=$?
    
    [ $choice -eq 255 ] && choice=6
    
    case $choice in
        0)
            apply_theme "dracula"
            apply_preset "full"
            ;;
        1)
            apply_theme "solarized-light"
            apply_preset "minimal"
            ;;
        2)
            apply_theme "synthwave"
            apply_preset "full"
            ;;
        3)
            apply_theme "monochrome"
            apply_preset "minimal"
            ;;
        4)
            apply_theme "one-dark"
            apply_preset "gaming"
            ;;
        5)
            apply_theme "nord"
            apply_preset "developer"
            ;;
        6)
            return
            ;;
    esac
    
    echo ""
    echo -e "\033[1;32m✨ Setup complete! Your terminal is now customized. ✨\033[0m"
    echo -e "\033[1;33mYou can always run 'termfetch-studio' to change settings.\033[0m"
    echo ""
    sleep 3
}

first_run_wizard() {
    if [ ! -f "$CONFIG_FILE" ]; then
        show_header
        echo -e "\033[1;32m🎉 Welcome to TermFetch Studio! 🎉\033[0m"
        echo ""
        echo -e "\033[1;36mLet's set up your terminal experience...\033[0m"
        echo ""
        
        setup_wizard
        set_config_value "general" "first_run" "false"
    fi
}

init_config() {
    if [ ! -f "$CONFIG_FILE" ]; then
        mkdir -p "$(dirname "$CONFIG_FILE")"

        cat > "$CONFIG_FILE" << EOF
[colors]
theme=dracula

[fastfetch]
preset=full
icons=true
logo_type=auto
custom_image=
use_theme_colors=true
key_color=theme
value_color=theme
labels=false

[general]
first_run=true
backup_enabled=true

[terminal]
font=JetBrainsMono Nerd Font
font_size=12
EOF
    else
        # Repair config if sections are missing
        if ! grep -q "^\[fastfetch\]" "$CONFIG_FILE"; then
            echo "" >> "$CONFIG_FILE"
            echo "[fastfetch]" >> "$CONFIG_FILE"
            echo "preset=full" >> "$CONFIG_FILE"
            echo "icons=true" >> "$CONFIG_FILE"
            echo "logo_type=auto" >> "$CONFIG_FILE"
            echo "custom_image=" >> "$CONFIG_FILE"
            echo "use_theme_colors=true" >> "$CONFIG_FILE"
            echo "key_color=theme" >> "$CONFIG_FILE"
            echo "value_color=theme" >> "$CONFIG_FILE"
            echo "labels=false" >> "$CONFIG_FILE"
        fi

        if ! grep -q "^\[general\]" "$CONFIG_FILE"; then
            echo "" >> "$CONFIG_FILE"
            echo "[general]" >> "$CONFIG_FILE"
            echo "first_run=false" >> "$CONFIG_FILE"
            echo "backup_enabled=true" >> "$CONFIG_FILE"
        fi

        if ! grep -q "^\[terminal\]" "$CONFIG_FILE"; then
            echo "" >> "$CONFIG_FILE"
            echo "[terminal]" >> "$CONFIG_FILE"
            echo "font=JetBrainsMono Nerd Font" >> "$CONFIG_FILE"
            echo "font_size=12" >> "$CONFIG_FILE"
        fi
    fi

    # Create default fastfetch config
    if [ ! -f "$CONFIG_DIR/fastfetch.jsonc" ]; then
        local theme=$(get_config_value "colors" "theme")
        [ -z "$theme" ] && theme="dracula"
        create_full_config "auto" "" "$theme"
    fi
}

get_config_value() {
    local section=$1
    local key=$2
    awk -F '=' -v section="$section" -v key="$key" '
        /^\[/ { current_section = substr($0, 2, length($0)-2) }
        current_section == section && $1 == key { print $2 }
    ' "$CONFIG_FILE" 2>/dev/null
}

set_config_value() {
    local section=$1
    local key=$2
    local value=$3

    if [ ! -f "$CONFIG_FILE" ]; then
        init_config
    fi

    # Escape special characters for sed
    local escaped_value=$(printf '%s\n' "$value" | sed 's/[\/&]/\\&/g')

    if ! grep -q "^\[$section\]" "$CONFIG_FILE"; then
        echo "" >> "$CONFIG_FILE"
        echo "[$section]" >> "$CONFIG_FILE"
        echo "$key=$value" >> "$CONFIG_FILE"
    elif grep -q "^\[$section\]" "$CONFIG_FILE"; then
        if grep -q "^$key=" "$CONFIG_FILE"; then
            # Use a temporary file for safer sed operation
            local tmp_file=$(mktemp)
            awk -v section="$section" -v key="$key" -v value="$value" '
                BEGIN { in_section=0 }
                /^\[.*\]$/ {
                    in_section = ($0 == "["section"]")
                }
                in_section && $0 ~ "^"key"=" {
                    print key"="value
                    next
                }
                { print }
            ' "$CONFIG_FILE" > "$tmp_file"
            mv "$tmp_file" "$CONFIG_FILE"
        else
            sed -i "/^\[$section\]/a $key=$escaped_value" "$CONFIG_FILE"
        fi
    fi
}

create_backup() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_file="$BACKUP_DIR/backup_$timestamp.tar.gz"
    
    mkdir -p "$BACKUP_DIR"
    
    tar -czf "$backup_file" -C "$CONFIG_DIR" . 2>/dev/null
    
    if [ $? -eq 0 ]; then
        echo -e "\n\033[1;32m✓ Backup created: $backup_file\033[0m"
    else
        echo -e "\n\033[1;31m✗ Backup failed!\033[0m"
    fi
    sleep 1
}

restore_backup() {
    local backups=($(ls -1 "$BACKUP_DIR"/*.tar.gz 2>/dev/null))
    
    if [ ${#backups[@]} -eq 0 ]; then
        echo -e "\n\033[1;33m⚠️  No backups found!\033[0m"
        sleep 1
        return
    fi
    
    echo -e "\n\033[1;33m📂 Available backups:\033[0m"
    for i in "${!backups[@]}"; do
        echo "  $((i+1))) $(basename "${backups[$i]}")"
    done
    
    echo -ne "\n\033[1;36mEnter backup number to restore: \033[0m"
    read -r choice
    
    if [ "$choice" -ge 1 ] && [ "$choice" -le "${#backups[@]}" ]; then
        local backup_file="${backups[$((choice-1))]}"
        
        echo -e "\n\033[1;33m🔄 Restoring backup...\033[0m"
        tar -xzf "$backup_file" -C "$CONFIG_DIR"
        
        if [ $? -eq 0 ]; then
            echo -e "\033[1;32m✓ Backup restored!\033[0m"
        else
            echo -e "\033[1;31m✗ Restore failed!\033[0m"
        fi
    else
        echo -e "\033[1;31m✗ Invalid choice!\033[0m"
    fi
    sleep 1
}

list_backups() {
    local backups=($(ls -1 "$BACKUP_DIR"/*.tar.gz 2>/dev/null))
    
    if [ ${#backups[@]} -eq 0 ]; then
        echo -e "\n\033[1;33m⚠️  No backups found!\033[0m"
    else
        echo -e "\n\033[1;33m📂 Available backups:\033[0m"
        for backup in "${backups[@]}"; do
            echo "  📄 $(basename "$backup")"
        done
    fi
    
    echo ""
    echo -e "\033[1;33mPress any key to continue...\033[0m"
    read -n 1 -s
}
CORELIB
    
    chmod +x "$INSTALL_DIR/lib/core.sh"
    print_success "Core library installed"
}

install_fastfetch_lib() {
    print_info "Installing fastfetch library..."
    
    tee "$INSTALL_DIR/lib/fastfetch.sh" > /dev/null << 'FASTFETCHLIB'
#!/bin/bash

# Provide no-op logger if core lib isn't sourced
type log_debug >/dev/null 2>&1 || log_debug() { :; }

# Return distro-appropriate ASCII logo path for auto fallback on unsupported terminals
get_distro_ascii_logo_path() {
    local id=""
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        id="${ID:-}"
    fi
    case "$id" in
        arch|endeavouros|manjaro|cachyos) echo "$HOME/.local/share/termfetch-studio/logos/arch.txt" ;;
        *) echo "$HOME/.local/share/termfetch-studio/logos/tux.txt" ;;
    esac
}

get_logo_source() {
    local logo_type=$1
    local custom_image=$2

    case $logo_type in
        "pikachu")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/pikachu.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/pikachu.txt"
            else
                echo "auto"
            fi
            ;;
        "tux")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/tux.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/tux.txt"
            else
                echo "auto"
            fi
            ;;
        "arch")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/arch.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/arch.txt"
            else
                echo "auto"
            fi
            ;;
        "cat")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/cat.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/cat.txt"
            else
                echo "auto"
            fi
            ;;
        "neko")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/neko.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/neko.txt"
            else
                echo "auto"
            fi
            ;;
        "bunny")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/bunny.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/bunny.txt"
            else
                echo "auto"
            fi
            ;;
        "heart")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/heart.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/heart.txt"
            else
                echo "auto"
            fi
            ;;
        "sakura")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/sakura.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/sakura.txt"
            else
                echo "auto"
            fi
            ;;
        "naruto")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/naruto.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/naruto.txt"
            else
                echo "auto"
            fi
            ;;
        "pokeball")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/pokeball.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/pokeball.txt"
            else
                echo "auto"
            fi
            ;;
        "spider")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/spider.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/spider.txt"
            else
                echo "auto"
            fi
            ;;
        "dragon")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/dragon.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/dragon.txt"
            else
                echo "auto"
            fi
            ;;
        "creeper")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/creeper.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/creeper.txt"
            else
                echo "auto"
            fi
            ;;
        "star")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/star.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/star.txt"
            else
                echo "auto"
            fi
            ;;
        "rocket")
            if [ -f "$HOME/.local/share/termfetch-studio/logos/rocket.txt" ]; then
                echo "$HOME/.local/share/termfetch-studio/logos/rocket.txt"
            else
                echo "auto"
            fi
            ;;
        "custom")
            if [ -f "$LOGOS_DIR/custom.txt" ]; then
                echo "$LOGOS_DIR/custom.txt"
            else
                echo "auto"
            fi
            ;;
        "image")
            # ALWAYS check terminal support at runtime
            local terminal_support=$(check_terminal_image_support)

            if [ "$terminal_support" = "none" ]; then
                # Terminal doesn't support images - use fallback
                local fallback=$(get_config_value "fastfetch" "logo_fallback")
                [ -z "$fallback" ] && fallback="auto"

                case $fallback in
                    "ascii")
                        # Use pikachu ASCII logo as fallback
                        echo "$HOME/.local/share/termfetch-studio/logos/pikachu.txt"
                        ;;
                    "none") echo "none" ;;
                    *) echo "$(get_distro_ascii_logo_path)" ;;  # Auto → distro ASCII fallback
                esac
            else
                # Terminal supports images
                if [ -n "$custom_image" ] && [ -f "$custom_image" ]; then
                    echo "$custom_image"
                else
                    # Use builtin distro logo; leave source empty for auto
                    echo ""
                fi
            fi
            ;;
        "none") echo "none" ;;
        *) echo "" ;;  # For auto/builtin, no explicit source
    esac
}

get_logo_type() {
    local logo_type=$1
    local custom_image=$2

    case $logo_type in
        "pikachu"|"tux"|"arch"|"cat"|"neko"|"bunny"|"heart"|"sakura"|"naruto"|"pokeball"|"spider"|"dragon"|"creeper"|"star"|"rocket"|"custom") 
            # ASCII logos - don't specify type, let fastfetch auto-detect from file
            echo "" ;;
        "image")
            # ALWAYS check terminal support at runtime, even if custom image exists
            local terminal_support=$(check_terminal_image_support)

            # Debug logging
            log_debug "get_logo_type: terminal_support=$terminal_support, custom_image=$custom_image"

            if [ "$terminal_support" = "none" ]; then
                # Terminal doesn't support images - use fallback
                local fallback=$(get_config_value "fastfetch" "logo_fallback")
                [ -z "$fallback" ] && fallback="auto"

                case $fallback in
                    "ascii") echo "" ;;   # ASCII file - no type needed
                    "none") echo "none" ;;
                    *) echo "" ;;          # Auto → distro ASCII - no type needed
                esac
            else
                # Terminal supports images - check if custom image exists
                if [ -n "$custom_image" ] && [ -f "$custom_image" ]; then
                    # Use appropriate protocol for this terminal
                    case $terminal_support in
                        "kitty") echo "kitty" ;;
                        "wezterm") echo "wezterm" ;;
                        "iterm2") echo "iterm2" ;;
                        "konsole") echo "kitty" ;;  # Konsole supports kitty protocol
                        *) echo "kitty" ;;  # Default to kitty protocol
                    esac
                else
                    # No custom image - use distro auto logo
                    echo "auto"
                fi
            fi
            ;;
        "none") echo "none" ;;
        *) echo "auto" ;;
    esac
}

get_theme_type() {
    local theme=$1
    
    case $theme in
        "gruvbox-light"|"solarized-light"|"one-light"|"ayu-light"|"strawberry")
            echo "light"
            ;;
        *)
            echo "dark"
            ;;
    esac
}

get_theme_ansi_color() {
    local theme=$1
    local color_type=$2
    
    case $theme in
        dracula)
            case $color_type in
                primary) echo "35" ;;   # magenta/purple
                secondary) echo "33" ;; # yellow
                accent) echo "32" ;;    # green
                text) echo "37" ;;      # white
                *) echo "36" ;;         # cyan
            esac
            ;;
        nord)
            case $color_type in
                primary) echo "36" ;;   # blue
                secondary) echo "34" ;; # light blue
                accent) echo "32" ;;    # green
                text) echo "37" ;;      # white
                *) echo "36" ;;         # cyan
            esac
            ;;
        gruvbox-dark)
            case $color_type in
                primary) echo "33" ;;   # yellow
                secondary) echo "34" ;; # blue
                accent) echo "32" ;;    # green
                text) echo "37" ;;      # white
                *) echo "33" ;;         # yellow
            esac
            ;;
        tokyo-night)
            case $color_type in
                primary) echo "34" ;;   # blue
                secondary) echo "35" ;; # magenta
                accent) echo "32" ;;    # green
                text) echo "37" ;;      # white
                *) echo "34" ;;         # blue
            esac
            ;;
        one-dark)
            case $color_type in
                primary) echo "36" ;;   # cyan
                secondary) echo "35" ;; # magenta
                accent) echo "32" ;;    # green
                text) echo "37" ;;      # white
                *) echo "36" ;;         # cyan
            esac
            ;;
        synthwave)
            case $color_type in
                primary) echo "36" ;;   # cyan
                secondary) echo "35" ;; # magenta
                accent) echo "32" ;;    # green
                text) echo "37" ;;      # white
                *) echo "36" ;;         # cyan
            esac
            ;;
        *)
            echo "36" ;; # cyan
    esac
}

# Helper function to get icon color (either custom or theme-based)
get_fastfetch_color() {
    local position=$1  # primary, secondary, etc.
    local theme=$2

    local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme_colors" ] && use_theme_colors="true"

    if [ "$use_theme_colors" = "false" ]; then
        # Use custom colors if set
        local custom_color=$(get_config_value "fastfetch" "key_color")
        [ -n "$custom_color" ] && [ "$custom_color" != "theme" ] && echo "$custom_color" && return
    fi

    # Fall back to theme colors
    get_theme_ansi_color "$theme" "$position"
}

# Helper function to get text color (either custom or theme-based)
get_fastfetch_text_color() {
    local position=$1  # primary, secondary, etc.
    local theme=$2

    local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme_colors" ] && use_theme_colors="true"

    if [ "$use_theme_colors" = "false" ]; then
        # Use custom text color if set
        local custom_text_color=$(get_config_value "fastfetch" "value_color")
        if [ -n "$custom_text_color" ] && [ "$custom_text_color" != "default" ]; then
            echo "$custom_text_color"
            return
        fi
    fi

    # Fall back to theme colors
    get_theme_ansi_color "$theme" "$position"
}

# Helper function to get color dots string based on theme
get_color_dots() {
    local theme=$1
    local color1=$(get_theme_ansi_color "$theme" "primary")
    local color2=$(get_theme_ansi_color "$theme" "secondary")
    local color3=$(get_theme_ansi_color "$theme" "accent")
    local color4=$(get_theme_ansi_color "$theme" "highlight")
    local color5=$(get_theme_ansi_color "$theme" "info")
    local color6="$color1"
    local color7="$color2"
    local color8="$color3"

    echo "\\u001b[1;${color1}m●\\u001b[0m \\u001b[1;${color2}m●\\u001b[0m \\u001b[1;${color3}m●\\u001b[0m \\u001b[1;${color4}m●\\u001b[0m \\u001b[1;${color5}m●\\u001b[0m \\u001b[1;${color6}m●\\u001b[0m \\u001b[1;${color7}m●\\u001b[0m \\u001b[1;${color8}m●\\u001b[0m"
}

create_full_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3

    local logo_source=$(get_logo_source "$logo_type" "$custom_image")
    local logo_actual_type=$(get_logo_type "$logo_type" "$custom_image")

    # Get theme-based or custom ANSI colors
    local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme_colors" ] && use_theme_colors="true"

    local color1 color2 color3 color4 color5
    local output_color
    local show_labels=$(get_config_value "fastfetch" "labels")
    [ -z "$show_labels" ] && show_labels="false"
    
    # Color dots should be independent of icon colors
    local dot_color1=$(get_theme_ansi_color "$theme" "primary")
    local dot_color2=$(get_theme_ansi_color "$theme" "secondary")
    local dot_color3=$(get_theme_ansi_color "$theme" "accent")
    local dot_color4=$(get_theme_ansi_color "$theme" "highlight")
    local dot_color5=$(get_theme_ansi_color "$theme" "info")
    local dot_color6="$dot_color1"
    local dot_color7="$dot_color2"
    local dot_color8="$dot_color3"
    
    if [ "$use_theme_colors" = "false" ]; then
        local custom_key=$(get_config_value "fastfetch" "key_color")
        local custom_value=$(get_config_value "fastfetch" "value_color")
        [ -z "$custom_key" ] && custom_key="34"
        [ -z "$custom_value" ] && custom_value="37"
        color1="$custom_key"
        color2="$custom_key"
        color3="$custom_key"
        color4="$custom_key"
        color5="$custom_key"
        # Text colors - different from icon colors
        tcolor1="$custom_value"
        tcolor2="$custom_value"
        tcolor3="$custom_value"
        tcolor4="$custom_value"
        tcolor5="$custom_value"
        output_color="$custom_value"
    else
        color1=$(get_theme_ansi_color "$theme" "primary")
        color2=$(get_theme_ansi_color "$theme" "secondary")
        color3=$(get_theme_ansi_color "$theme" "accent")
        color4=$(get_theme_ansi_color "$theme" "highlight")
        color5=$(get_theme_ansi_color "$theme" "info")
        # Text colors - each group's text color matches its icon color
        # For light themes, use darker variants for better readability
        local theme_type=$(get_theme_type "$theme")
        if [ "$theme_type" = "light" ]; then
            # Light themes: use much darker colors for text for better contrast
            tcolor1="0"   # OS Group - pure black
            tcolor2="0"   # WM Group - pure black
            tcolor3="0"   # PC Group - pure black
            tcolor4="0"   # NET Group - pure black
            tcolor5="0"   # TIME Group - pure black
        else
            # Dark themes: use same colors as icons
            tcolor1="$color1"  # OS Group - same as icon color
            tcolor2="$color2"  # WM Group - same as icon color  
            tcolor3="$color3"  # PC Group - same as icon color
            tcolor4="$color4"  # NET Group - same as icon color
            tcolor5="$color5"  # TIME Group - same as icon color
        fi
        output_color=$(get_fastfetch_text_color "primary" "$theme")
    fi

    cat > "$CONFIG_DIR/fastfetch.jsonc" << EOF
{
    // TermFetch Studio - Full Info Preset
    // Theme: $theme
    "\$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",

    "logo": {
        "type": "$logo_actual_type",
        "source": "$logo_source",
        "height": 18,
        "padding": {
            "top": 2,
            "right": 4
        }
    },

    "display": {
        "separator": " ➜ "
    },

    "modules": [
        "break",
        "break",
        {
            "type": "title",
            "format": "{user-name-colored}@{host-name-colored}",
            "key": ""
        },
        "break",
        {
            "type": "os",
            "key": " ├ 󰣇 ",
            "keyColor": "$color1",
            "outputColor": "$tcolor1",
            "keyWidth": 10,
            "keyShow": $show_labels
        },
        {
            "type": "kernel",
            "key": " ├ 󰌢 ",
            "keyColor": "$color1",
            "outputColor": "$tcolor1",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "packages",
            "key": " ├ 󰏖 ",
            "keyColor": "$color1",
            "outputColor": "$tcolor1",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "shell",
            "key": " ├ 󰆍 ",
            "keyColor": "$color1",
            "outputColor": "$tcolor1",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        "break",
        {
            "type": "wm",
            "key": " ├ 󰖲 ",
            "keyColor": "$color2",
            "outputColor": "$tcolor2",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "de",
            "key": " ├ 󱂬 ",
            "keyColor": "$color2",
            "outputColor": "$tcolor2",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "wmtheme",
            "key": " ├ 󰉼 ",
            "keyColor": "$color2",
            "outputColor": "$tcolor2",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "icons",
            "key": " ├ 󰀻 ",
            "keyColor": "$color2",
            "outputColor": "$tcolor2",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "cursor",
            "key": " ├ 󰘔 ",
            "keyColor": "$color2",
            "outputColor": "$tcolor2",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "terminal",
            "key": " ├ 󰆍 ",
            "keyColor": "$color2",
            "outputColor": "$tcolor2",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "terminalfont",
            "key": " ├ 󰛖 ",
            "keyColor": "$color2",
            "outputColor": "$tcolor2",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        "break",
        {
            "type": "host",
            "key": " ├ 󰌢 ",
            "keyColor": "$color3",
            "outputColor": "$tcolor3",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "cpu",
            "key": " ├ 󰻠 ",
            "keyColor": "$color3",
            "outputColor": "$tcolor3",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "gpu",
            "key": " ├ 󰢮 ",
            "keyColor": "$color3",
            "outputColor": "$tcolor3",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "memory",
            "key": " ├ 󰍛 ",
            "keyColor": "$color3",
            "outputColor": "$tcolor3",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "swap",
            "key": " ├ 󰓡 ",
            "keyColor": "$color3",
            "outputColor": "$tcolor3",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "disk",
            "key": " ├ 󰋊 ",
            "keyColor": "$color3",
            "outputColor": "$tcolor3",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "battery",
            "key": " ├ 󰁹 ",
            "keyColor": "$color3",
            "outputColor": "$tcolor3",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "display",
            "key": " ├ 󰍹 ",
            "keyColor": "$color3",
            "outputColor": "$tcolor3",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        "break",
        {
            "type": "localip",
            "key": " ├ 󰣺 ",
            "keyColor": "$color4",
            "outputColor": "$tcolor4",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "publicip",
            "key": " ├ 󰞉 ",
            "keyColor": "$color4",
            "outputColor": "$tcolor4",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "wifi",
            "key": " ├ 󰖩 ",
            "keyColor": "$color4",
            "outputColor": "$tcolor4",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        "break",
        {
            "type": "datetime",
            "key": " ├ 󰅐 ",
            "keyColor": "$color5",
            "outputColor": "$tcolor5",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "uptime",
            "key": " ├ 󰔚 ",
            "keyColor": "$color5",
            "outputColor": "$tcolor5",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        {
            "type": "media",
            "key": " ├ 󰎈 ",
            "keyColor": "$color5",
            "outputColor": "$tcolor5",
            "keyWidth": 10,
            "keyShow": $show_labels,
        },
        "break",
        "break"
    ]
}
EOF
}

create_minimal_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3

    local logo_source=$(get_logo_source "$logo_type" "$custom_image")
    local logo_actual_type=$(get_logo_type "$logo_type" "$custom_image")

    # Get theme-based or custom ANSI colors
    local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme_colors" ] && use_theme_colors="true"

    local color1 color2 color3 color4 color5 color6 color7 color8
    local output_color
    local show_labels=$(get_config_value "fastfetch" "labels")
    [ -z "$show_labels" ] && show_labels="false"
    
    # Color dots should be independent of icon colors
    local dot_color1=$(get_theme_ansi_color "$theme" "primary")
    local dot_color2=$(get_theme_ansi_color "$theme" "secondary")
    local dot_color3=$(get_theme_ansi_color "$theme" "accent")
    local dot_color4=$(get_theme_ansi_color "$theme" "highlight")
    local dot_color5=$(get_theme_ansi_color "$theme" "info")
    local dot_color6="$dot_color1"
    local dot_color7="$dot_color2"
    local dot_color8="$dot_color3"
    
    if [ "$use_theme_colors" = "false" ]; then
        local custom_key=$(get_config_value "fastfetch" "key_color")
        local custom_value=$(get_config_value "fastfetch" "value_color")
        [ -z "$custom_key" ] && custom_key="34"
        [ -z "$custom_value" ] && custom_value="37"
        color1="$custom_key"
        color2="$custom_key"
        color3="$custom_key"
        color4="$custom_key"
        color5="$custom_key"
        color6="$custom_key"
        color7="$custom_key"
        color8="$custom_key"
        output_color="$custom_value"
    else
        color1=$(get_theme_ansi_color "$theme" "primary")
        color2=$(get_theme_ansi_color "$theme" "secondary")
        color3=$(get_theme_ansi_color "$theme" "accent")
        color4=$(get_theme_ansi_color "$theme" "highlight")
        color5=$(get_theme_ansi_color "$theme" "info")
        color6="$color1"
        color7="$color2"
        color8="$color3"
        output_color=$(get_fastfetch_text_color "primary" "$theme")
    fi

    cat > "$CONFIG_DIR/fastfetch.jsonc" << EOF
{
    // TermFetch Studio - Minimal Preset
    // Theme: $theme
    "\$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",

    "logo": {
        "type": "$logo_actual_type",
        "source": "$logo_source",
        "height": 12,
        "padding": {
            "top": 2,
            "right": 3
        }
    },

    "display": {
        "separator": " ➜ "
    },

    "modules": [
        "break",
        "break",
        {
            "type": "title",
            "format": "{user-name-colored}@{host-name-colored}",
            "key": ""
        },
        "break",
        {
            "type": "custom",
            "format": "\\u001b[1;${dot_color1}m●\\u001b[0m \\u001b[1;${dot_color2}m●\\u001b[0m \\u001b[1;${dot_color3}m●\\u001b[0m \\u001b[1;${dot_color4}m●\\u001b[0m \\u001b[1;${dot_color5}m●\\u001b[0m \\u001b[1;${dot_color6}m●\\u001b[0m \\u001b[1;${dot_color7}m●\\u001b[0m \\u001b[1;${dot_color8}m●\\u001b[0m"
        },
        "break",
        {
            "type": "os",
            "key": " ├ 󰣇 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        {
            "type": "kernel",
            "key": " ├ 󰌢 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        {
            "type": "shell",
            "key": " ├ 󰆍 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        {
            "type": "uptime",
            "key": " ├ 󰔚 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        "break",
        {
            "type": "custom",
            "format": "\\u001b[1;${dot_color1}m●\\u001b[0m \\u001b[1;${dot_color2}m●\\u001b[0m \\u001b[1;${dot_color3}m●\\u001b[0m \\u001b[1;${dot_color4}m●\\u001b[0m \\u001b[1;${dot_color5}m●\\u001b[0m \\u001b[1;${dot_color6}m●\\u001b[0m \\u001b[1;${dot_color7}m●\\u001b[0m \\u001b[1;${dot_color8}m●\\u001b[0m"
        },
        "break"
    ]
}
EOF
}

create_focused_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3

    local logo_source=$(get_logo_source "$logo_type" "$custom_image")
    local logo_actual_type=$(get_logo_type "$logo_type" "$custom_image")

    # Get theme-based ANSI colors
    local color1=$(get_theme_ansi_color "$theme" "primary")
    local color2=$(get_theme_ansi_color "$theme" "secondary")
    local color3=$(get_theme_ansi_color "$theme" "accent")

    local color4 color5 color6 color7 color8
    local output_color
    local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme_colors" ] && use_theme_colors="true"

    if [ "$use_theme_colors" = "false" ]; then
        local custom_key=$(get_config_value "fastfetch" "key_color")
        local custom_value=$(get_config_value "fastfetch" "value_color")
        [ -z "$custom_key" ] && custom_key="34"
        [ -z "$custom_value" ] && custom_value="default"
        # Override existing color1, color2, color3 with custom_key
        color1="$custom_key"
        color2="$custom_key"
        color3="$custom_key"
        color4="$custom_key"
        color5="$custom_key"
        color6="$custom_key"
        color7="$custom_key"
        color8="$custom_key"
        output_color=$(get_fastfetch_text_color "primary" "$theme")
    else
        # Keep existing theme color assignments for color1, color2, color3
        # Add these:
        color4=$(get_theme_ansi_color "$theme" "highlight")
        color5=$(get_theme_ansi_color "$theme" "info")
        color6="$color1"
        color7="$color2"
        color8="$color3"
        output_color=$(get_fastfetch_text_color "primary" "$theme")
    fi

    cat > "$CONFIG_DIR/fastfetch.jsonc" << EOF
{
    // TermFetch Studio - Focused Preset
    // Theme: $theme
    "\$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",

    "logo": {
        "type": "$logo_actual_type",
        "source": "$logo_source",
        "height": 15,
        "padding": {
            "top": 2,
            "right": 3
        }
    },

    "display": {
        "separator": " ➜ "
    },

    "modules": [
        "break",
        "break",
        {
            "type": "title",
            "format": "{user-name-colored}@{host-name-colored}",
            "key": ""
        },
        "break",
        {
            "type": "custom",
            "format": "\\u001b[1;${dot_color1}m●\\u001b[0m \\u001b[1;${dot_color2}m●\\u001b[0m \\u001b[1;${dot_color3}m●\\u001b[0m \\u001b[1;${dot_color4}m●\\u001b[0m \\u001b[1;${dot_color5}m●\\u001b[0m \\u001b[1;${dot_color6}m●\\u001b[0m \\u001b[1;${dot_color7}m●\\u001b[0m \\u001b[1;${dot_color8}m●\\u001b[0m"
        },
        "break",
        {
            "type": "os",
            "key": " ├ 󰣇 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        {
            "type": "kernel",
            "key": " ├ 󰌢 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        {
            "type": "packages",
            "key": " ├ 󰏖 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        {
            "type": "uptime",
            "key": " ├ 󰔟 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        "break",
        {
            "type": "terminal",
            "key": " ├ 󰆍 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        {
            "type": "shell",
            "key": " ├ 󰆍 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        "break",
        {
            "type": "cpu",
            "key": " ├ 󰻠 ",
            "keyColor": "$color3",
            "outputColor": "$output_color"
        },
        {
            "type": "memory",
            "key": " ├ 󰍛 ",
            "keyColor": "$color3",
            "outputColor": "$output_color"
        },
        {
            "type": "disk",
            "key": " ├ 󰋊 ",
            "keyColor": "$color3",
            "outputColor": "$output_color"
        },
        "break",
        {
            "type": "custom",
            "format": "\\u001b[1;${dot_color1}m●\\u001b[0m \\u001b[1;${dot_color2}m●\\u001b[0m \\u001b[1;${dot_color3}m●\\u001b[0m \\u001b[1;${dot_color4}m●\\u001b[0m \\u001b[1;${dot_color5}m●\\u001b[0m \\u001b[1;${dot_color6}m●\\u001b[0m \\u001b[1;${dot_color7}m●\\u001b[0m \\u001b[1;${dot_color8}m●\\u001b[0m"
        },
        "break"
    ]
}
EOF
}

create_developer_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3

    local logo_source=$(get_logo_source "$logo_type" "$custom_image")
    local logo_actual_type=$(get_logo_type "$logo_type" "$custom_image")

    # Get theme-based ANSI colors
    local color1=$(get_theme_ansi_color "$theme" "primary")
    local color2=$(get_theme_ansi_color "$theme" "secondary")
    local color3=$(get_theme_ansi_color "$theme" "accent")
    local color4=$(get_theme_ansi_color "$theme" "highlight")

    local color5 color6 color7 color8
    local output_color
    local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme_colors" ] && use_theme_colors="true"

    if [ "$use_theme_colors" = "false" ]; then
        local custom_key=$(get_config_value "fastfetch" "key_color")
        local custom_value=$(get_config_value "fastfetch" "value_color")
        [ -z "$custom_key" ] && custom_key="34"
        [ -z "$custom_value" ] && custom_value="default"
        # Override existing color1, color2, color3 with custom_key
        color1="$custom_key"
        color2="$custom_key"
        color3="$custom_key"
        color4="$custom_key"
        color5="$custom_key"
        color6="$custom_key"
        color7="$custom_key"
        color8="$custom_key"
        output_color=$(get_fastfetch_text_color "primary" "$theme")
    else
        # Keep existing theme color assignments for color1, color2, color3
        # Add these:
        color5=$(get_theme_ansi_color "$theme" "info")
        color6="$color1"
        color7="$color2"
        color8="$color3"
        output_color=$(get_fastfetch_text_color "primary" "$theme")
    fi

    cat > "$CONFIG_DIR/fastfetch.jsonc" << EOF
{
    // TermFetch Studio - Developer Preset
    // Theme: $theme
    "\$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",

    "logo": {
        "type": "$logo_actual_type",
        "source": "$logo_source",
        "height": 16,
        "padding": {
            "top": 2,
            "right": 4
        }
    },

    "display": {
        "separator": " ➜ "
    },

    "modules": [
        "break",
        "break",
        {
            "type": "title",
            "format": "{user-name-colored}@{host-name-colored}",
            "key": ""
        },
        "break",
        {
            "type": "custom",
            "format": "\\u001b[1;${dot_color1}m●\\u001b[0m \\u001b[1;${dot_color2}m●\\u001b[0m \\u001b[1;${dot_color3}m●\\u001b[0m \\u001b[1;${dot_color4}m●\\u001b[0m \\u001b[1;${dot_color5}m●\\u001b[0m \\u001b[1;${dot_color6}m●\\u001b[0m \\u001b[1;${dot_color7}m●\\u001b[0m \\u001b[1;${dot_color8}m●\\u001b[0m"
        },
        "break",
        {
            "type": "os",
            "key": " ├ 󰣇 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        {
            "type": "kernel",
            "key": " ├ 󰌢 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        {
            "type": "packages",
            "key": " ├ 󰏖 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        {
            "type": "shell",
            "key": " ├ 󰆍 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        "break",
        {
            "type": "terminal",
            "key": " ├ 󰆍 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        {
            "type": "terminalfont",
            "key": " ├ 󰛖 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        {
            "type": "de",
            "key": " ├ 󱂬 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        {
            "type": "wm",
            "key": " ├ 󰖲 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        "break",
        {
            "type": "cpu",
            "key": " ├ 󰻠 ",
            "keyColor": "$color3",
            "outputColor": "$output_color"
        },
        {
            "type": "gpu",
            "key": " ├ 󰢮 ",
            "keyColor": "$color3",
            "outputColor": "$output_color"
        },
        {
            "type": "memory",
            "key": " ├ 󰍛 ",
            "keyColor": "$color3",
            "outputColor": "$output_color"
        },
        {
            "type": "swap",
            "key": " ├ 󰓡 ",
            "keyColor": "$color3",
            "outputColor": "$output_color"
        },
        {
            "type": "disk",
            "key": " ├ 󰋊 ",
            "keyColor": "$color3",
            "outputColor": "$output_color"
        },
        "break",
        {
            "type": "localip",
            "key": " ├ 󰩟 ",
            "keyColor": "$color4",
            "outputColor": "$output_color"
        },
        {
            "type": "publicip",
            "key": " ├ 󰞉 ",
            "keyColor": "$color4",
            "outputColor": "$output_color"
        },
        "break",
        {
            "type": "custom",
            "format": "\\u001b[1;${dot_color1}m●\\u001b[0m \\u001b[1;${dot_color2}m●\\u001b[0m \\u001b[1;${dot_color3}m●\\u001b[0m \\u001b[1;${dot_color4}m●\\u001b[0m \\u001b[1;${dot_color5}m●\\u001b[0m \\u001b[1;${dot_color6}m●\\u001b[0m \\u001b[1;${dot_color7}m●\\u001b[0m \\u001b[1;${dot_color8}m●\\u001b[0m"
        },
        "break"
    ]
}
EOF
}

create_gaming_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3

    local logo_source=$(get_logo_source "$logo_type" "$custom_image")
    local logo_actual_type=$(get_logo_type "$logo_type" "$custom_image")

    # Get theme-based ANSI colors
    local color1=$(get_theme_ansi_color "$theme" "primary")
    local color2=$(get_theme_ansi_color "$theme" "secondary")
    local color3=$(get_theme_ansi_color "$theme" "accent")

    local color4 color5 color6 color7 color8
    local output_color
    local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme_colors" ] && use_theme_colors="true"

    if [ "$use_theme_colors" = "false" ]; then
        local custom_key=$(get_config_value "fastfetch" "key_color")
        local custom_value=$(get_config_value "fastfetch" "value_color")
        [ -z "$custom_key" ] && custom_key="34"
        [ -z "$custom_value" ] && custom_value="default"
        # Override existing color1, color2, color3 with custom_key
        color1="$custom_key"
        color2="$custom_key"
        color3="$custom_key"
        color4="$custom_key"
        color5="$custom_key"
        color6="$custom_key"
        color7="$custom_key"
        color8="$custom_key"
        output_color=$(get_fastfetch_text_color "primary" "$theme")
    else
        # Keep existing theme color assignments for color1, color2, color3
        # Add these:
        color4=$(get_theme_ansi_color "$theme" "highlight")
        color5=$(get_theme_ansi_color "$theme" "info")
        color6="$color1"
        color7="$color2"
        color8="$color3"
        output_color=$(get_fastfetch_text_color "primary" "$theme")
    fi

    cat > "$CONFIG_DIR/fastfetch.jsonc" << EOF
{
    // TermFetch Studio - Gaming Preset
    // Theme: $theme
    "\$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",

    "logo": {
        "type": "$logo_actual_type",
        "source": "$logo_source",
        "height": 16,
        "padding": {
            "top": 2,
            "right": 4
        }
    },

    "display": {
        "separator": " ➜ "
    },

    "modules": [
        "break",
        "break",
        {
            "type": "title",
            "format": "{user-name-colored}@{host-name-colored}",
            "key": ""
        },
        "break",
        {
            "type": "custom",
            "format": "\\u001b[1;${dot_color1}m●\\u001b[0m \\u001b[1;${dot_color2}m●\\u001b[0m \\u001b[1;${dot_color3}m●\\u001b[0m \\u001b[1;${dot_color4}m●\\u001b[0m \\u001b[1;${dot_color5}m●\\u001b[0m \\u001b[1;${dot_color6}m●\\u001b[0m \\u001b[1;${dot_color7}m●\\u001b[0m \\u001b[1;${dot_color8}m●\\u001b[0m"
        },
        "break",
        {
            "type": "os",
            "key": " ├ 󰣇 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        {
            "type": "kernel",
            "key": " ├ 󰌢 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        {
            "type": "uptime",
            "key": " ├ 󰔟 ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        },
        "break",
        {
            "type": "cpu",
            "key": " ├ 󰻠 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        {
            "type": "gpu",
            "key": " ├ 󰢮 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        {
            "type": "memory",
            "key": " ├ 󰍛 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        {
            "type": "swap",
            "key": " ├ 󰓡 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        {
            "type": "disk",
            "key": " ├ 󰋊 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        {
            "type": "display",
            "key": " ├ 󰍹 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        {
            "type": "battery",
            "key": " ├ 󰂎 ",
            "keyColor": "$color2",
            "outputColor": "$output_color"
        },
        "break",
        {
            "type": "localip",
            "key": " ├ 󰩟 ",
            "keyColor": "$color3",
            "outputColor": "$output_color"
        },
        {
            "type": "wifi",
            "key": " ├ 󰖩 ",
            "keyColor": "$color3",
            "outputColor": "$output_color"
        },
        "break",
        {
            "type": "custom",
            "format": "\\u001b[1;${dot_color1}m●\\u001b[0m \\u001b[1;${dot_color2}m●\\u001b[0m \\u001b[1;${dot_color3}m●\\u001b[0m \\u001b[1;${dot_color4}m●\\u001b[0m \\u001b[1;${dot_color5}m●\\u001b[0m \\u001b[1;${dot_color6}m●\\u001b[0m \\u001b[1;${dot_color7}m●\\u001b[0m \\u001b[1;${dot_color8}m●\\u001b[0m"
        },
        "break"
    ]
}
EOF
}

# Create custom config using a saved list of modules in config.ini
create_custom_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3

    local logo_source=$(get_logo_source "$logo_type" "$custom_image")
    local logo_actual_type=$(get_logo_type "$logo_type" "$custom_image")

    local color1=$(get_theme_ansi_color "$theme" "primary")

    local color2 color3 color4 color5 color6 color7 color8
    local output_color
    local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme_colors" ] && use_theme_colors="true"

    if [ "$use_theme_colors" = "false" ]; then
        local custom_key=$(get_config_value "fastfetch" "key_color")
        local custom_value=$(get_config_value "fastfetch" "value_color")
        [ -z "$custom_key" ] && custom_key="34"
        [ -z "$custom_value" ] && custom_value="default"
        # Override existing color1, color2, color3 with custom_key
        color1="$custom_key"
        color2="$custom_key"
        color3="$custom_key"
        color4="$custom_key"
        color5="$custom_key"
        color6="$custom_key"
        color7="$custom_key"
        color8="$custom_key"
        output_color=$(get_fastfetch_text_color "primary" "$theme")
    else
        # Keep existing theme color assignments for color1, color2, color3
        # Add these:
        color2=$(get_theme_ansi_color "$theme" "secondary")
        color3=$(get_theme_ansi_color "$theme" "accent")
        color4=$(get_theme_ansi_color "$theme" "highlight")
        color5=$(get_theme_ansi_color "$theme" "info")
        color6="$color1"
        color7="$color2"
        color8="$color3"
        output_color=$(get_fastfetch_text_color "primary" "$theme")
    fi

    local show_labels=$(get_config_value "fastfetch" "labels")
    [ -z "$show_labels" ] && show_labels="false"

    # Read modules from config (comma separated), provide sensible default
    local mods=$(get_config_value "custom" "modules")
    if [ -z "$mods" ]; then
        mods="os,kernel,packages,shell,cpu,memory,disk,uptime"
    fi

    # Start building JSON
    cat > "$CONFIG_DIR/fastfetch.jsonc" << 'JSONSTART'
{
    // TermFetch Studio - Custom Preset
JSONSTART

    echo "    // Theme: $theme" >> "$CONFIG_DIR/fastfetch.jsonc"

    cat >> "$CONFIG_DIR/fastfetch.jsonc" << 'JSONMID'
    "$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",

    "logo": {
JSONMID

    cat >> "$CONFIG_DIR/fastfetch.jsonc" << EOF
        "type": "$logo_actual_type",
        "source": "$logo_source",
        "height": 16,
        "padding": {
            "top": 2,
            "right": 3
        }
    },

    "display": {
        "separator": " ➜ "
    },

    "modules": [
        "break",
        "break",
        {
            "type": "title",
            "format": "{user-name-colored}@{host-name-colored}",
            "key": ""
        },
        "break",
        {
            "type": "custom",
            "format": "\\u001b[1;${dot_color1}m●\\u001b[0m \\u001b[1;${dot_color2}m●\\u001b[0m \\u001b[1;${dot_color3}m●\\u001b[0m \\u001b[1;${dot_color4}m●\\u001b[0m \\u001b[1;${dot_color5}m●\\u001b[0m \\u001b[1;${dot_color6}m●\\u001b[0m \\u001b[1;${dot_color7}m●\\u001b[0m \\u001b[1;${dot_color8}m●\\u001b[0m"
        },
        "break"
EOF

    # Add custom modules with proper icons
    for m in $(echo "$mods" | tr ',' ' '); do
        local icon="󰀀"
        case $m in
            os) icon="󰣇" ;;
            kernel) icon="󰌢" ;;
            packages) icon="󰏖" ;;
            shell) icon="󰆍" ;;
            wm) icon="󰖲" ;;
            de) icon="󱂬" ;;
            wmtheme) icon="󰉼" ;;
            icons) icon="󰀻" ;;
            cursor) icon="󰘔" ;;
            terminal) icon="󰆍" ;;
            terminalfont) icon="󰛖" ;;
            host) icon="󰌢" ;;
            cpu) icon="󰻠" ;;
            gpu) icon="󰢮" ;;
            memory) icon="󰍛" ;;
            swap) icon="󰓡" ;;
            disk) icon="󰋊" ;;
            battery) icon="󰁹" ;;
            display) icon="󰍹" ;;
            localip) icon="󰩟" ;;
            publicip) icon="󰞉" ;;
            wifi) icon="󰖩" ;;
            datetime) icon="󰅐" ;;
            uptime) icon="󰔚" ;;
            media) icon="󰎈" ;;
        esac

        cat >> "$CONFIG_DIR/fastfetch.jsonc" << EOF
        ,{
            "type": "$m",
            "key": " ├ $icon  ",
            "keyColor": "$color1",
            "outputColor": "$output_color"
        }
EOF
    done

    # Add closing color dots
    cat >> "$CONFIG_DIR/fastfetch.jsonc" << EOF
        ,"break"
        ,{
            "type": "custom",
            "format": "\\u001b[1;${dot_color1}m●\\u001b[0m \\u001b[1;${dot_color2}m●\\u001b[0m \\u001b[1;${dot_color3}m●\\u001b[0m \\u001b[1;${dot_color4}m●\\u001b[0m \\u001b[1;${dot_color5}m●\\u001b[0m \\u001b[1;${dot_color6}m●\\u001b[0m \\u001b[1;${dot_color7}m●\\u001b[0m \\u001b[1;${dot_color8}m●\\u001b[0m"
        }
EOF

    # Close JSON
    cat >> "$CONFIG_DIR/fastfetch.jsonc" << 'JSONEND'
        ,"break"
    ]
}
JSONEND
}

# Interactive editor for custom preset modules
configure_custom_preset() {
    local all=(os kernel packages shell wm de wmtheme icons cursor terminal terminalfont host cpu gpu memory swap disk battery display localip publicip wifi datetime uptime media)
    local current=$(get_config_value "custom" "modules")
    local selected=()
    if [ -n "$current" ]; then
        IFS=',' read -r -a selected <<< "$current"
    else
        selected=(os kernel shell uptime)
    fi

    # Module display names and icons
    declare -A module_names=(
        [os]="󰣇 Operating System"
        [kernel]="󰌢 Kernel"
        [packages]="󰏖 Packages"
        [shell]="󰆍 Shell"
        [wm]="󰖲 Window Manager"
        [de]="󱂬 Desktop Environment"
        [wmtheme]="󰉼 WM Theme"
        [icons]="󰀻 Icon Theme"
        [cursor]="󰘔 Cursor Theme"
        [terminal]="󰆍 Terminal"
        [terminalfont]="󰛖 Terminal Font"
        [host]="󰌢 Host"
        [cpu]="󰻠 CPU"
        [gpu]="󰢮 GPU"
        [memory]="󰍛 Memory"
        [swap]="󰓡 Swap"
        [disk]="󰋊 Disk"
        [battery]="󰁹 Battery"
        [display]="󰍹 Display"
        [localip]="󰩟 Local IP"
        [publicip]="󰞉 Public IP"
        [wifi]="󰖩 WiFi"
        [datetime]="󰅐 Date & Time"
        [uptime]="󰔚 Uptime"
        [media]="󰎈 Media"
    )

    local selected_index=0
    local total=${#all[@]}

    while true; do
        show_header
        echo -e "\033[1;36m╔═══════════════════════════════════════════════════════╗\033[0m"
        echo -e "\033[1;36m║          🧩 Custom Preset Configuration 🧩           ║\033[0m"
        echo -e "\033[1;36m╚═══════════════════════════════════════════════════════╝\033[0m"
        echo ""
        echo -e "\033[1;33m📋 GUIDE: [↑↓] Navigate  [ENTER] Toggle  [S] Save  [R] Reorder  [Q] Quit\033[0m"
        echo ""

        # Display modules in single column
        for i in "${!all[@]}"; do
            local name=${all[$i]}
            local mark="✗"
            local mark_color="\033[1;31m"
            for s in "${selected[@]}"; do
                if [ "$s" = "$name" ]; then
                    mark="✓"
                    mark_color="\033[1;32m"
                    break
                fi
            done
            local display_name="${module_names[$name]}"
            
            if [ $i -eq $selected_index ]; then
                echo -e "  \033[1;32m▶\033[0m $mark_color$mark\033[0m $display_name"
            else
                echo -e "    $mark_color$mark\033[0m $display_name"
            fi
        done

        echo ""
        echo -e "\033[1;34m═══════════════════════════════════════════════════════════\033[0m"
        echo -e "\033[1;32mSelected: ${#selected[@]} modules\033[0m"
        echo -e "\033[1;36mCurrent: ${module_names[${all[$selected_index]}]}\033[0m"
        echo ""
        echo -e "\033[1;33m[↑↓] Navigate  [ENTER] Toggle  [S] Save  [R] Reorder  [Q] Quit\033[0m"

        # Read single key
        read -rsn1 key
        
        case $key in
            $'\x1b')
                read -rsn2 key
                case $key in
                    '[A') # Up arrow
                        if [ $selected_index -gt 0 ]; then
                            selected_index=$((selected_index - 1))
                        fi
                        ;;
                    '[B') # Down arrow
                        if [ $selected_index -lt $((total - 1)) ]; then
                            selected_index=$((selected_index + 1))
                        fi
                        ;;
                esac
                ;;
            '') # Enter key
                local item=${all[$selected_index]}
                local exists=false
                local new=()
                for s in "${selected[@]}"; do
                    if [ "$s" = "$item" ]; then exists=true; else new+=("$s"); fi
                done
                if [ "$exists" = true ]; then
                    selected=("${new[@]}")
                else
                    selected+=("$item")
                fi
                ;;
            's'|'S') # Save
                local modules_str=$(IFS=','; echo "${selected[*]}")
                set_config_value "custom" "modules" "$modules_str"
                set_config_value "fastfetch" "preset" "custom"
                create_custom_config "$(get_config_value fastfetch logo_type)" "$(get_config_value fastfetch custom_image)" "$(get_config_value colors theme)"
                echo -e "\033[1;32m✓ Custom preset configured!\033[0m"
                break
                ;;
            'r'|'R') # Reorder
                reorder_custom_modules
                ;;
            'q'|'Q') # Quit
                break
                ;;
        esac
    done
}

# Reorder custom modules
reorder_custom_modules() {
    local current=$(get_config_value "custom" "modules")
    local selected=()
    if [ -n "$current" ]; then
        IFS=',' read -r -a selected <<< "$current"
    else
        echo -e "\033[1;31mNo custom modules to reorder!\033[0m"
        sleep 1
        return
    fi
    
    if [ ${#selected[@]} -le 1 ]; then
        echo -e "\033[1;31mNeed at least 2 modules to reorder!\033[0m"
        sleep 1
        return
    fi
    
    show_header
    echo -e "\033[1;36m╔═══════════════════════════════════════════════════════╗\033[0m"
    echo -e "\033[1;36m║            🔄 Reorder Custom Modules 🔄              ║\033[0m"
    echo -e "\033[1;36m╚═══════════════════════════════════════════════════════╝\033[0m"
    echo ""
    echo -e "\033[1;33m📋 REORDER GUIDE:\033[0m"
    echo -e "\033[1;36m  [←→] Select item to move\033[0m"
    echo -e "\033[1;36m  [↑↓] Move selected item up/down\033[0m"
    echo -e "\033[1;36m  [ENTER] Save new order\033[0m"
    echo -e "\033[1;36m  [Q] Cancel and exit\033[0m"
    echo ""
    
    local selected_index=0
    local total=${#selected[@]}
    
    while true; do
        show_header
        echo -e "\033[1;36m╔═══════════════════════════════════════════════════════╗\033[0m"
        echo -e "\033[1;36m║            🔄 Reorder Custom Modules 🔄              ║\033[0m"
        echo -e "\033[1;36m╚═══════════════════════════════════════════════════════╝\033[0m"
        echo ""
        
        # Display modules with current order
        for i in "${!selected[@]}"; do
            if [ $i -eq $selected_index ]; then
                echo -e "  \033[1;32m▶\033[0m $((i+1)). ${selected[$i]}"
            else
                echo -e "    $((i+1)). ${selected[$i]}"
            fi
        done
        
        echo ""
        echo -e "\033[1;33m[↑↓] Swap with Up/Down  [←→] Select Item  [ENTER] Save  [Q] Cancel\033[0m"
        
        # Read single key
        read -rsn1 key
        
        case $key in
            $'\x1b')
                read -rsn2 key
                case $key in
                    '[A') # Up arrow
                        if [ $selected_index -gt 0 ]; then
                            # Swap with previous item
                            local temp="${selected[$selected_index]}"
                            selected[$selected_index]="${selected[$((selected_index-1))]}"
                            selected[$((selected_index-1))]="$temp"
                            selected_index=$((selected_index-1))
                        fi
                        ;;
                    '[B') # Down arrow
                        if [ $selected_index -lt $((total-1)) ]; then
                            # Swap with next item
                            local temp="${selected[$selected_index]}"
                            selected[$selected_index]="${selected[$((selected_index+1))]}"
                            selected[$((selected_index+1))]="$temp"
                            selected_index=$((selected_index+1))
                        fi
                        ;;
                    '[C') # Right arrow - Select item below
                        if [ $selected_index -lt $((total-1)) ]; then
                            selected_index=$((selected_index+1))
                        fi
                        ;;
                    '[D') # Left arrow - Select item above
                        if [ $selected_index -gt 0 ]; then
                            selected_index=$((selected_index-1))
                        fi
                        ;;
                esac
                ;;
            '') # Enter key
                # Save new order
                local modules_str=$(IFS=','; echo "${selected[*]}")
                set_config_value "custom" "modules" "$modules_str"
                create_custom_config "$(get_config_value fastfetch logo_type)" "$(get_config_value fastfetch custom_image)" "$(get_config_value colors theme)"
                echo -e "\033[1;32m✓ Module order updated!\033[0m"
                break
                ;;
            'q'|'Q') # Quit
                break
                ;;
        esac
    done
}

FASTFETCHLIB
    
    chmod +x "$INSTALL_DIR/lib/fastfetch.sh"
    print_success "Fastfetch library installed"
}

setup_permissions() {
    print_info "Setting up permissions..."

    chmod -R 755 "$INSTALL_DIR"
    chown -R "$USER:$USER" "$INSTALL_DIR"

    chmod -R 755 "$CONFIG_DIR"
    chown -R "$USER:$USER" "$CONFIG_DIR"

    print_success "Permissions configured"
}

setup_shell_integration() {
    print_info "Setting up shell integration..."

    # Detect shell and add integration
    if [ -n "$BASH_VERSION" ] || [ -f "$HOME/.bashrc" ]; then
        # Remove any existing TermFetch Studio block
        if [ -f "$HOME/.bashrc" ]; then
            sed -i '/# TermFetch Studio - START/,/# TermFetch Studio - END/d' "$HOME/.bashrc" 2>/dev/null || true
        fi

        # Add new integration block with markers
        cat >> "$HOME/.bashrc" << 'BASHRC'

# TermFetch Studio - START
export PATH="$HOME/.local/bin:$PATH"
[ -f ~/.config/termfetch-studio/theme.sh ] && source ~/.config/termfetch-studio/theme.sh
if [ -x "$HOME/.local/bin/tfs-fastfetch" ]; then
    "$HOME/.local/bin/tfs-fastfetch"
elif command -v fastfetch &> /dev/null && [ -f ~/.config/termfetch-studio/fastfetch.jsonc ]; then
    fastfetch --config ~/.config/termfetch-studio/fastfetch.jsonc
fi
# TermFetch Studio - END
BASHRC
        print_success "Added to ~/.bashrc"
    fi

    if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
        # Remove any existing TermFetch Studio block
        if [ -f "$HOME/.zshrc" ]; then
            sed -i '/# TermFetch Studio - START/,/# TermFetch Studio - END/d' "$HOME/.zshrc" 2>/dev/null || true
        fi

        # Add new integration block with markers
        cat >> "$HOME/.zshrc" << 'ZSHRC'

# TermFetch Studio - START
export PATH="$HOME/.local/bin:$PATH"
[ -f ~/.config/termfetch-studio/theme.sh ] && source ~/.config/termfetch-studio/theme.sh
if command -v fastfetch &> /dev/null && [ -f ~/.config/termfetch-studio/fastfetch.jsonc ]; then
    fastfetch --config ~/.config/termfetch-studio/fastfetch.jsonc
fi
# TermFetch Studio - END
ZSHRC
        print_success "Added to ~/.zshrc"
    fi

    if [ -f "$HOME/.config/fish/config.fish" ]; then
        # Remove any existing TermFetch Studio block
        sed -i '/# TermFetch Studio - START/,/# TermFetch Studio - END/d' "$HOME/.config/fish/config.fish" 2>/dev/null || true

        # Add new integration block with markers
        cat >> "$HOME/.config/fish/config.fish" << 'FISHRC'

# TermFetch Studio - START
fish_add_path ~/.local/bin
test -f ~/.config/termfetch-studio/theme.sh && bash ~/.config/termfetch-studio/theme.sh
if command -v fastfetch &> /dev/null && test -f ~/.config/termfetch-studio/fastfetch.jsonc
    fastfetch --config ~/.config/termfetch-studio/fastfetch.jsonc
end
# TermFetch Studio - END
FISHRC
        print_success "Added to ~/.config/fish/config.fish"
    fi

    print_success "Shell integration configured"
}

create_uninstall_script() {
    print_info "Creating uninstall script..."
    
    tee "$INSTALL_DIR/uninstall.sh" > /dev/null << 'UNINSTALL'
#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_banner() {
    echo -e "${RED}"
    cat << "EOF"
╔═══════════════════════════════════════════════════╗
║                                                   ║
║         ████████╗███████╗██████╗ ███╗   ███╗      ║
║         ╚══██╔══╝██╔════╝██╔══██╗████╗ ████║      ║
║            ██║   █████╗  ██████╔╝██╔████╔██║      ║
║            ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║      ║
║            ██║   ███████╗██║  ██║██║ ╚═╝ ██║      ║
║            ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝      ║
║                                                   ║
║              TermFetch Studio v1.0                ║
║                 Uninstall Script                  ║
║                                                   ║
╚═══════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

confirm_uninstall() {
    echo -e "${YELLOW}⚠️  This will completely remove TermFetch Studio from your system.${NC}"
    echo -e "${YELLOW}The following will be removed:${NC}"
    echo "  - /usr/local/bin/termfetch-studio"
    echo "  - $HOME/.local/share/termfetch-studio/"
    echo "  - $HOME/.config/termfetch-studio/"
    echo ""
    read -p "Are you sure you want to continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${GREEN}Uninstall cancelled.${NC}"
        exit 0
    fi
}

remove_shell_integration() {
    print_info "Removing shell integration..."

    # Resolve target user and home even when run via sudo
    TARGET_USER="${SUDO_USER:-$USER}"
    TARGET_HOME=$(getent passwd "$TARGET_USER" | cut -d: -f6)
    [ -z "$TARGET_HOME" ] && TARGET_HOME="$HOME"

    # Remove from bash using perl for reliable deletion
    if [ -f "$TARGET_HOME/.bashrc" ]; then
        if grep -q "TermFetch Studio" "$TARGET_HOME/.bashrc" 2>/dev/null; then
            perl -i -ne 'print unless /# TermFetch Studio - START/ .. /# TermFetch Studio - END/' "$TARGET_HOME/.bashrc"
            print_info "Removed from .bashrc"
        fi
    fi

    # Remove from zsh using perl
    if [ -f "$TARGET_HOME/.zshrc" ]; then
        if grep -q "TermFetch Studio" "$TARGET_HOME/.zshrc" 2>/dev/null; then
            perl -i -ne 'print unless /# TermFetch Studio - START/ .. /# TermFetch Studio - END/' "$TARGET_HOME/.zshrc"
            print_info "Removed from .zshrc"
        fi
    fi

    # Remove from fish using perl
    if [ -f "$TARGET_HOME/.config/fish/config.fish" ]; then
        if grep -q "TermFetch Studio" "$TARGET_HOME/.config/fish/config.fish" 2>/dev/null; then
            perl -i -ne 'print unless /# TermFetch Studio - START/ .. /# TermFetch Studio - END/' "$TARGET_HOME/.config/fish/config.fish"
            print_info "Removed from config.fish"
        fi
    fi
}

remove_installation() {
    print_info "Removing TermFetch Studio..."

    # Remove main executable
    if [ -f "/usr/local/bin/termfetch-studio" ]; then
        rm -f "/usr/local/bin/termfetch-studio"
        print_info "Removed main executable"
    fi

    # Remove installation directory
    if [ -d "$HOME/.local/share/termfetch-studio" ]; then
        rm -rf "$HOME/.local/share/termfetch-studio"
        print_info "Removed installation directory"
    fi

    # Remove shell integration
    remove_shell_integration

    # Ask about config directory
    echo ""
    echo -e "${YELLOW}Do you want to remove the configuration directory?${NC}"
    echo -e "${YELLOW}This will delete all your settings and backups.${NC}"
    read -p "Remove config directory? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        TARGET_USER="${SUDO_USER:-$USER}"
        TARGET_HOME=$(getent passwd "$TARGET_USER" | cut -d: -f6)
        [ -z "$TARGET_HOME" ] && TARGET_HOME="$HOME"
        if [ -d "$TARGET_HOME/.config/termfetch-studio" ]; then
            rm -rf "$TARGET_HOME/.config/termfetch-studio"
            print_info "Removed configuration directory"
        fi
    else
        TARGET_USER="${SUDO_USER:-$USER}"
        TARGET_HOME=$(getent passwd "$TARGET_USER" | cut -d: -f6)
        [ -z "$TARGET_HOME" ] && TARGET_HOME="$HOME"
        print_info "Configuration directory preserved at $TARGET_HOME/.config/termfetch-studio"
    fi
}

main() {
    print_banner
    confirm_uninstall
    remove_installation
    
    echo ""
    echo -e "${GREEN}✨ TermFetch Studio has been successfully uninstalled! ✨${NC}"
    echo -e "${GREEN}Thank you for using TermFetch Studio!${NC}"
}

main "$@"
UNINSTALL
    
    chmod +x "$INSTALL_DIR/uninstall.sh"
    print_success "Uninstall script created"
}

show_post_install_info() {
    print_banner
    echo -e "${GREEN}✨ TermFetch Studio has been successfully installed! ✨${NC}"
    echo ""

    # Check if $HOME/.local/bin is in PATH
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo -e "${YELLOW}⚠️  Important: $HOME/.local/bin is not in your PATH${NC}"
        echo -e "${CYAN}Add it to your shell configuration:${NC}"
        echo ""
        echo -e "${YELLOW}For Bash:${NC}"
        echo -e "  ${GREEN}echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc${NC}"
        echo -e "  ${GREEN}source ~/.bashrc${NC}"
        echo ""
        echo -e "${YELLOW}For Zsh:${NC}"
        echo -e "  ${GREEN}echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc${NC}"
        echo -e "  ${GREEN}source ~/.zshrc${NC}"
        echo ""
        echo -e "${YELLOW}For Fish:${NC}"
        echo -e "  ${GREEN}fish_add_path ~/.local/bin${NC}"
        echo ""
        echo -e "${CYAN}Or restart your terminal for changes to take effect.${NC}"
        echo ""
    fi

    echo -e "${CYAN}${BOLD}What's Next:${NC}"
    echo -e "  ${YELLOW}1.${NC} Run ${GREEN}termfetch-studio${NC} to start customizing"
    echo -e "  ${YELLOW}2.${NC} Choose your theme and fastfetch preset"
    echo -e "  ${YELLOW}3.${NC} Configure logos and images"
    echo ""
    echo -e "${CYAN}${BOLD}Quick Commands:${NC}"
    echo -e "  ${GREEN}termfetch-studio${NC}           # Interactive menu"
    echo -e "  ${GREEN}termfetch-studio --preview${NC} # Quick preview"
    echo -e "  ${GREEN}termfetch-studio --help${NC}    # Show help"
    echo -e "  ${GREEN}termfetch-studio uninstall${NC} # Uninstall"
    echo ""
    echo -e "${CYAN}${BOLD}Font Setup:${NC}"
    echo -e "  ${YELLOW}•${NC} Make sure to set your terminal font to a Nerd Font"
    echo -e "  ${YELLOW}•${NC} Recommended: JetBrainsMono Nerd Font, Hack Nerd Font"
    echo -e "  ${YELLOW}•${NC} ${RED}IMPORTANT:${NC} In your terminal settings, set font to:"
    echo -e "      ${GREEN}'JetBrainsMono Nerd Font'${NC} or ${GREEN}'Hack Nerd Font'${NC}"
    echo -e "  ${YELLOW}•${NC} Without a Nerd Font, icons will not display correctly!"
    echo ""
    echo -e "${MAGENTA}${BOLD}Enjoy your beautiful terminal! 🎨${NC}"
    echo ""
}

main() {
    print_banner
    
    # Initialize logging
    mkdir -p "$CONFIG_DIR"
    log_debug "TermFetch Studio started"
    log_terminal_info
    
    # Check if running as root
    if [ "$EUID" -eq 0 ]; then
        print_error "Please don't run this script as root"
        log_error "Script run as root - blocked"
        exit 1
    fi
    
    # Check dependencies
    install_dependencies
    
    # Install Nerd Fonts if needed
    install_nerd_fonts
    
    # Create directories
    create_directories
    
    # Install components
    install_ascii_logos
    install_preset_images
    install_core_lib
    install_fastfetch_lib
    install_main_script
    install_fastfetch_wrapper

    # Setup permissions
    setup_permissions

    # Setup shell integration
    setup_shell_integration

    # Create uninstall script
    create_uninstall_script
    
    # Show post-install info
    show_post_install_info
    
    echo ""
    echo -e "${CYAN}Would you like to start TermFetch Studio now? (Y/n): ${NC}"
    read -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        "$BIN_DIR/termfetch-studio"
    fi
}

main "$@"

