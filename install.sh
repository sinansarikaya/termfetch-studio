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
    # Terminal graphics support is determined by the installed runtime library.
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
║             TermFetch Studio v1.0.0               ║
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
        local candidate
        for candidate in "${ID:-}" ${ID_LIKE:-}; do
            case "$candidate" in
                arch|manjaro|endeavouros|cachyos) echo arch; return ;;
                debian|ubuntu|linuxmint|pop) echo debian; return ;;
                fedora|rhel|centos) echo fedora; return ;;
                opensuse|opensuse-*|suse|sles) echo suse; return ;;
            esac
        done
        echo unknown
    else
        echo "unknown"
    fi
}

# The previous "from source" helper downloaded a binary and was never called.

require_command() {
    command -v "$1" >/dev/null 2>&1 || { print_error "Required command not found: $1"; return 1; }
}

install_debian_fastfetch() {
    local deb_arch expected_hash url tmp_dir
    case "$(uname -m)" in
        x86_64) deb_arch=amd64; expected_hash=fdee4233997e58cac927bb9d03278c5ff0d446f8993b9d80125b7035359ce908 ;;
        aarch64|arm64) deb_arch=aarch64; expected_hash=3f9309dba5a5bcf194aa64d836d27222dfbdeb50004af0182de08519f512f419 ;;
        *) print_error "Unsupported architecture: $(uname -m)"; return 1 ;;
    esac
    require_command dpkg || return 1
    require_command mktemp || return 1
    require_command sha256sum || return 1
    if command -v curl >/dev/null 2>&1; then
        :
    elif command -v wget >/dev/null 2>&1; then
        :
    else
        print_error "A download tool (curl or wget) is required for fastfetch"
        return 1
    fi
    tmp_dir=$(mktemp -d) || return 1
    url="https://github.com/fastfetch-cli/fastfetch/releases/download/2.53.0/fastfetch-linux-${deb_arch}.deb"
    if { command -v curl >/dev/null 2>&1 && curl -fL --retry 3 -o "$tmp_dir/fastfetch.deb" "$url"; } ||
       { command -v wget >/dev/null 2>&1 && wget -q -O "$tmp_dir/fastfetch.deb" "$url"; }; then
        if printf '%s  %s\n' "$expected_hash" "$tmp_dir/fastfetch.deb" | sha256sum -c - >/dev/null; then
            if ! sudo dpkg -i "$tmp_dir/fastfetch.deb"; then
                sudo apt-get install -f -y || { rm -rf -- "$tmp_dir"; return 1; }
            fi
        else
            print_error "Fastfetch download checksum mismatch"
            rm -rf -- "$tmp_dir"
            return 1
        fi
    else
        print_error "Failed to download fastfetch"
        rm -rf -- "$tmp_dir"
        return 1
    fi
    rm -rf -- "$tmp_dir"
    command -v fastfetch >/dev/null 2>&1
}

validate_fastfetch_version() {
    local version major minor
    version=$(fastfetch --version) || { print_error "Cannot read fastfetch version"; return 1; }
    if [[ "$version" =~ fastfetch[[:space:]]+([0-9]+)\.([0-9]+)\.([0-9]+) ]]; then
        major=$((10#${BASH_REMATCH[1]}))
        minor=$((10#${BASH_REMATCH[2]}))
        if (( major > 2 || (major == 2 && minor >= 42) )); then
            return 0
        fi
    fi
    print_error "fastfetch >= 2.42.0 is required for RGB configuration; found: $version"
    return 1
}

install_dependencies() {
    print_info "Checking and installing dependencies..."

    local distro_base=$(detect_distro_base)
    local missing_deps=()
    local optional_missing=()

    if ! command -v awk >/dev/null 2>&1; then
        require_command sudo || return 1
        case "$distro_base" in
            arch) require_command pacman && sudo pacman -S --noconfirm gawk || return 1 ;;
            debian) require_command apt-get && sudo apt-get install -y gawk || return 1 ;;
            fedora) require_command dnf && sudo dnf install -y gawk || return 1 ;;
            suse) require_command zypper && sudo zypper --non-interactive install gawk || return 1 ;;
            *) print_error "Required command awk is missing; install it before continuing"; return 1 ;;
        esac
    fi
    local tool
    for tool in awk sed grep tar gzip mktemp cp mv mkdir chmod chown uname date cat tr cut head tail sort tee touch sleep; do
        require_command "$tool" || return 1
    done

    # Check required dependencies
    if ! command -v fastfetch &> /dev/null; then
        missing_deps+=("fastfetch")
    fi

    # Check optional dependencies
    if ! command -v chafa &> /dev/null; then
        optional_missing+=("chafa")
    fi

    if ! command -v magick &> /dev/null && ! command -v convert &> /dev/null; then
        optional_missing+=("imagemagick")
    fi

    # Auto-install missing required dependencies
    if [ ${#missing_deps[@]} -ne 0 ]; then
        require_command sudo || return 1
        print_info "Installing required dependencies: ${missing_deps[*]}"
        case $distro_base in
            arch)
                require_command pacman || return 1
                sudo pacman -S --noconfirm fastfetch || {
                    print_error "Failed to install fastfetch"
                    print_info "Please install manually: sudo pacman -S fastfetch"
                    return 1
                }
                ;;
            debian)
                install_debian_fastfetch || return 1
                ;;
            fedora)
                require_command dnf || return 1
                sudo dnf install -y fastfetch || {
                    print_error "Failed to install fastfetch"
                    print_info "Please install manually: sudo dnf install fastfetch"
                    return 1
                }
                ;;
            suse)
                require_command zypper || return 1
                sudo zypper --non-interactive install fastfetch || {
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
    
    validate_fastfetch_version || return 1

    # Auto-install optional dependencies
    if [ ${#optional_missing[@]} -ne 0 ] && [ "$distro_base" != unknown ]; then
        require_command sudo || return 1
        case "$distro_base" in
            arch) require_command pacman || return 1 ;;
            debian) require_command apt-get || return 1 ;;
            fedora) require_command dnf || return 1 ;;
            suse) require_command zypper || return 1 ;;
        esac
        print_info "Installing optional dependencies: ${optional_missing[*]}"
        local optional_ok=true
        case $distro_base in
            arch)
                sudo pacman -S --noconfirm "${optional_missing[@]}" || optional_ok=false
                ;;
            debian)
                sudo apt-get install -y "${optional_missing[@]}" || optional_ok=false
                ;;
            fedora)
                local packages=("${optional_missing[@]/imagemagick/ImageMagick}")
                sudo dnf install -y "${packages[@]}" || optional_ok=false
                ;;
            suse)
                local packages=("${optional_missing[@]/imagemagick/ImageMagick}")
                sudo zypper --non-interactive install "${packages[@]}" || optional_ok=false
                ;;
        esac
        if [ "$optional_ok" = true ] && { [[ "${optional_missing[*]}" != *chafa* ]] || command -v chafa >/dev/null 2>&1; } &&
           { [[ "${optional_missing[*]}" != *imagemagick* ]] || command -v magick >/dev/null 2>&1 || command -v convert >/dev/null 2>&1; }; then
            print_success "Optional dependencies installed"
        else
            print_warning "Some optional dependencies failed to install"
        fi
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
    if command -v fc-list >/dev/null 2>&1 && fc-list 2>/dev/null | grep -i "nerd" &> /dev/null; then
        font_installed=true
    fi
    
    if [ "$font_installed" = false ]; then
        echo ""
        print_warning "Nerd Fonts not detected. These are required for icons to display properly."
        echo ""
        read -p "Install Nerd Fonts? (Y/n) " -n 1 -r
        echo

        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            local distro_base
            distro_base=$(detect_distro_base)
            require_command sudo || return 1
            case $distro_base in
                arch)
                    require_command pacman || return 1
                    print_info "Installing Nerd Fonts via pacman..."
                    if ! sudo pacman -S --noconfirm ttf-jetbrains-mono-nerd; then
                        print_warning "Nerd Font installation failed"
                    fi
                    ;;
                debian|fedora|suse)
                    print_info "Installing Fontconfig utilities..."
                    case $distro_base in
                        debian) require_command apt-get || return 1; sudo apt-get install -y fontconfig curl unzip || true ;;
                        fedora) require_command dnf || return 1; sudo dnf install -y fontconfig curl unzip || true ;;
                        suse) require_command zypper || return 1; sudo zypper --non-interactive install fontconfig curl unzip || true ;;
                    esac
                    local font_tmp
                    for tool in curl unzip fc-cache mktemp; do
                        if ! command -v "$tool" >/dev/null 2>&1; then
                            print_warning "Nerd Font installation skipped: $tool is unavailable"
                            return 0
                        fi
                    done
                    print_info "Downloading and installing JetBrainsMono Nerd Font..."
                    font_tmp=$(mktemp -d) || return 1
                    if curl -fL -o "$font_tmp/JetBrainsMono.zip" "https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.zip" &&
                       mkdir -p "$HOME/.local/share/fonts/JetBrainsMono" &&
                       unzip -o "$font_tmp/JetBrainsMono.zip" -d "$HOME/.local/share/fonts/JetBrainsMono" &&
                       fc-cache -fv "$HOME/.local/share/fonts/JetBrainsMono"; then
                        print_success "JetBrainsMono Nerd Font installed"
                    else
                        print_warning "Nerd Font installation failed"
                    fi
                    rm -rf -- "$font_tmp"
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
        exit $?
        ;;
    --restore)
        init_config
        restore_backup
        exit $?
        ;;
    uninstall)
        shift  # Remove 'uninstall' from arguments
        bash "$INSTALL_DIR/uninstall.sh" "$@"
        exit 0
        ;;
    "")
        # One alternate-screen session for the whole interactive application.
        trap 'ui_session_end' EXIT
        ui_session_begin
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
logo_width=""
logo_height=""
case "$logo_actual_type" in
  kitty-direct|kitty|iterm|sixel)
    if [ -n "$logo_source" ] && [ -f "$logo_source" ] && declare -f compute_logo_dimensions >/dev/null 2>&1; then
      dims=$(configured_logo_dimensions "$logo_source" 14)
      read -r logo_width logo_height <<< "$dims"
    else
      logo_width=20
      logo_height=10
    fi
    ;;
esac

# Prepare temp config with adjusted logo fields
TMP=$(mktemp "$CONFIG_DIR/.fastfetch.XXXXXXXX.jsonc")
trap 'rm -f -- "$TMP"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

# Update logo.type and logo.source inside the JSON safely with awk
awk -v t="$logo_actual_type" -v s="$logo_source" -v w="$logo_width" -v h="$logo_height" '
  BEGIN{ inlogo=0 }
  {
    # Detect logo block start
    if ($0 ~ /"logo"[[:space:]]*:[[:space:]]*\{/){
      inlogo=1
      print
      if (t != "") print "        \"type\": \"" t "\",";
      if (s != "") print "        \"source\": \"" s "\",";
      if (w != "") print "        \"width\": " w ",";
      if (h != "") print "        \"height\": " h ",";
      next
    }
    
    # Detect logo block end
    if (inlogo==1 && $0 ~ /^[[:space:]]*}[[:space:]]*,?[[:space:]]*$/){
      inlogo=0
      print
      next
    }
    
    # Inside logo block - skip overridden fields
    if (inlogo==1 && ($0 ~ /"type"[[:space:]]*:/ || $0 ~ /"source"[[:space:]]*:/ || $0 ~ /"width"[[:space:]]*:/ || $0 ~ /"height"[[:space:]]*:/)){
      next
    }
    
    # Default: print the line as-is
    print
  }
' "$JSON" > "$TMP"

# Run fastfetch with the temp config
fastfetch --config "$TMP" "$@"
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

# ImageMagick 6 exposes convert/identify; version 7 prefers magick.
has_image_magick() { command -v magick >/dev/null 2>&1 || command -v convert >/dev/null 2>&1; }
image_convert() {
    if command -v magick >/dev/null 2>&1; then magick "$@"; else convert "$@"; fi
}
image_identify() {
    if command -v magick >/dev/null 2>&1; then magick identify "$@"; else identify "$@"; fi
}

get_image_pixel_dimensions() {
    local img="$1"
    [ -f "$img" ] || return 1

    # 1. ImageMagick identify
    if command -v magick >/dev/null 2>&1; then
        local out
        out=$(magick identify -format "%w %h" "$img" 2>/dev/null)
        if [[ "$out" =~ ^([0-9]+)[[:space:]]+([0-9]+)$ ]]; then
            echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]}"
            return 0
        fi
    elif command -v identify >/dev/null 2>&1; then
        local out
        out=$(identify -format "%w %h" "$img" 2>/dev/null)
        if [[ "$out" =~ ^([0-9]+)[[:space:]]+([0-9]+)$ ]]; then
            echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]}"
            return 0
        fi
    fi

    # 2. file command
    if command -v file >/dev/null 2>&1; then
        local f_out
        f_out=$(file -b "$img" 2>/dev/null)
        if [[ "$f_out" =~ ([0-9]+)[[:space:]]*x[[:space:]]*([0-9]+) ]]; then
            echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]}"
            return 0
        fi
    fi

    # 3. Python standard library (no third-party dependencies)
    if command -v python3 >/dev/null 2>&1; then
        local py_out
        py_out=$(python3 -c '
import sys, struct
p = sys.argv[1]
w, h = 0, 0
try:
    with open(p, "rb") as f:
        head = f.read(32)
        if head.startswith(b"\x89PNG\r\n\x1a\n") and len(head) >= 24:
            w, h = struct.unpack(">II", head[16:24])
        elif head[:2] == b"\xff\xd8":
            f.seek(2)
            while True:
                b = f.read(2)
                if len(b) < 2 or b[0] != 0xff: break
                while b[1] == 0xff: b = b[0:1] + f.read(1)
                m = b[1]
                if m in (0xc0, 0xc1, 0xc2, 0xc3, 0xc5, 0xc6, 0xc7, 0xc9, 0xca, 0xcb, 0xcd, 0xce, 0xcf):
                    L = struct.unpack(">H", f.read(2))[0]
                    d = f.read(L - 2)
                    h, w = struct.unpack(">HH", d[1:5])
                    break
                elif m in (0xd9, 0xda): break
                else:
                    L = struct.unpack(">H", f.read(2))[0]
                    f.seek(L - 2, 1)
        elif head[:6] in (b"GIF87a", b"GIF89a"):
            w, h = struct.unpack("<HH", head[6:10])
        elif head[:2] == b"BM":
            w, h = struct.unpack("<ii", head[18:26])
            h = abs(h)
except Exception:
    pass
if w > 0 and h > 0:
    print(f"{w} {h}")
' "$img" 2>/dev/null)
        if [[ "$py_out" =~ ^([0-9]+)[[:space:]]+([0-9]+)$ ]]; then
            echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]}"
            return 0
        fi
    fi

    return 1
}

ui_terminal_size() {
    local rows cols
    read -r rows cols < <(stty size 2>/dev/null) || true
    [[ "$cols" =~ ^[1-9][0-9]*$ ]] || cols=${COLUMNS:-80}
    [[ "$rows" =~ ^[1-9][0-9]*$ ]] || rows=${LINES:-24}
    [[ "$cols" =~ ^[1-9][0-9]*$ ]] || cols=80
    [[ "$rows" =~ ^[1-9][0-9]*$ ]] || rows=24
    printf '%s %s\n' "$cols" "$rows"
}

get_terminal_cell_dimensions() {
    # Kernel winsize is a read-only query; no cursor/device response is consumed.
    local pixels cols rows cw ch
    if [ -t 0 ] && command -v python3 >/dev/null 2>&1; then
        pixels=$(python3 -c 'import os,fcntl,termios,struct
try:
 fd=os.open("/dev/tty",os.O_RDONLY|os.O_NOCTTY)
 r,c,w,h=struct.unpack("HHHH",fcntl.ioctl(fd,termios.TIOCGWINSZ,b"\0"*8));os.close(fd)
 if r and c and w and h: print(max(1,w//c),max(1,h//r))
except OSError: pass' 2>/dev/null)
        if [[ "$pixels" =~ ^([1-9][0-9]*)[[:space:]]+([1-9][0-9]*)$ ]]; then
            printf '%s\n' "$pixels"
            return
        fi
    fi
    if [ -t 0 ] && [ -n "${KITTY_WINDOW_ID:-}" ] && command -v kitten >/dev/null 2>&1 && command -v timeout >/dev/null 2>&1; then
        pixels=$(timeout 1s kitten icat --print-window-size 2>/dev/null) || pixels=""
        if [[ "$pixels" =~ ^([1-9][0-9]*)x([1-9][0-9]*)$ ]]; then
            local pw=${BASH_REMATCH[1]} ph=${BASH_REMATCH[2]}
            read -r cols rows < <(ui_terminal_size)
            cw=$((pw / cols)); ch=$((ph / rows))
            if (( cw > 0 && ch > 0 )); then printf '%s %s\n' "$cw" "$ch"; return; fi
        fi
    fi
    # Without pixel metadata use conventional cells, with explicit bounded sizes.
    printf '1 2\n'
}

compute_logo_dimensions() {
    local img=$1 max_w=${2:-20} max_h=${3:-14}
    local cols rows pw=1 ph=1 cw ch dims w h
    read -r cols rows < <(ui_terminal_size)
    read -r cw ch < <(get_terminal_cell_dimensions)
    [[ "$max_w" =~ ^[1-9][0-9]*$ ]] || max_w=20
    [[ "$max_h" =~ ^[1-9][0-9]*$ ]] || max_h=14
    local bound_w=$((cols / 3)) bound_h=$((rows - 4))
    ((bound_w < 1)) && bound_w=1
    ((bound_h < 1)) && bound_h=1
    ((bound_w > max_w)) && bound_w=$max_w
    ((bound_h > max_h)) && bound_h=$max_h
    dims=$(get_image_pixel_dimensions "$img") || dims='1 1'
    if [[ "$dims" =~ ^([1-9][0-9]*)[[:space:]]+([1-9][0-9]*)$ ]]; then
        pw=${BASH_REMATCH[1]}; ph=${BASH_REMATCH[2]}
        # Reject absurd metadata before integer multiplication.
        ((pw > 1000000 || ph > 1000000)) && { pw=1; ph=1; }
    fi
    w=$bound_w
    h=$(((w * ph * cw + pw * ch / 2) / (pw * ch)))
    ((h < 1)) && h=1
    if ((h > bound_h)); then
        h=$bound_h
        w=$(((h * pw * ch + ph * cw / 2) / (ph * cw)))
        ((w > bound_w)) && w=$bound_w
        ((w < 1)) && w=1
    fi
    # Prefer the largest integer-cell rectangle within 5% of the pixel ratio.
    # This avoids avoidable rounding distortion for wide and tall images.
    local candidate_h candidate_w delta target area best_area=0 best_w=$w best_h=$h
    for ((candidate_h=1; candidate_h<=bound_h; candidate_h++)); do
        candidate_w=$(((candidate_h * pw * ch + ph * cw / 2) / (ph * cw)))
        ((candidate_w >= 1 && candidate_w <= bound_w)) || continue
        target=$((candidate_h * pw * ch))
        delta=$((candidate_w * ph * cw - target)); ((delta < 0)) && delta=$((-delta))
        area=$((candidate_w * candidate_h))
        if ((delta * 100 <= target * 5 && area > best_area)); then
            best_area=$area; best_w=$candidate_w; best_h=$candidate_h
        fi
    done
    w=$best_w; h=$best_h
    printf '%s %s\n' "$w" "$h"
}

configured_logo_dimensions() {
    local image=$1 height=22 width=36 size
    size=$(get_config_value fastfetch logo_size)
    case "$size" in
        small) width=20; height=14 ;;
        large) width=52; height=32 ;;
    esac
    compute_logo_dimensions "$image" "$width" "$height"
}

ui_clip_text() {
    # Conservative cell widths keep emoji/combining sequences from wrapping.
    local text=$1 budget=$2 ch cp width used=0 output='' i
    for ((i=0; i<${#text}; i++)); do
        ch=${text:i:1}
        printf -v cp '%d' "'$ch"
        ((cp < 32 || (cp >= 127 && cp <= 159))) && continue
        width=1
        if (( (cp >= 768 && cp <= 879) || cp == 8205 || (cp >= 65024 && cp <= 65039) )); then
            width=0
        elif (( (cp >= 4352 && cp <= 4607) || (cp >= 9728 && cp <= 10175) || (cp >= 11904 && cp <= 42191) || (cp >= 44032 && cp <= 55203) || (cp >= 63744 && cp <= 64255) || (cp >= 65040 && cp <= 65049) || (cp >= 65072 && cp <= 65135) || (cp >= 65280 && cp <= 65376) || (cp >= 65504 && cp <= 65510) || (cp >= 127744 && cp <= 129791) )); then
            width=2
        fi
        ((used + width > budget)) && break
        output+=$ch; used=$((used + width))
    done
    printf '%s' "$output"
}

show_header() {
    local cols rows
    read -r cols rows < <(ui_terminal_size)
    printf '\033[H\033[2J\033[1;36m'
    ui_clip_text 'TermFetch Studio 1.0.0' "$((cols - 1))"
    printf '\033[0m\n\n'
}

# Separate decorative icons from labels; labels always start at one cell anchor.
ui_option_parts() {
    local value=$1 first=${1%% *} cp
    UI_OPTION_ICON='' UI_OPTION_LABEL=$value
    printf -v cp '%d' "'${first:0:1}"
    if [[ "$value" == *' '* ]] && ((cp > 127)); then
        UI_OPTION_ICON=$first
        UI_OPTION_LABEL=${value#* }
        while [[ "$UI_OPTION_LABEL" == ' '* ]]; do UI_OPTION_LABEL=${UI_OPTION_LABEL# }; done
    fi
}

_ui_put() {
    local row=$1 col=$2 budget=$3 text=$4 style=${5:-'\033[0m'}
    ((row >= 1 && row <= _ui_height && col >= 1 && col <= _ui_width && budget > 0)) || return 0
    printf '\033[%d;%dH%b' "$row" "$col" "$style"
    ui_clip_text "$text" "$budget"
    printf '\033[0m'
}

ui_session_begin() {
    [[ "${_UI_SESSION_ACTIVE:-0}" == 1 ]] && return 0
    [ -t 0 ] && [ -t 1 ] || return 0
    _UI_SESSION_STTY=$(stty -g 2>/dev/null) || _UI_SESSION_STTY=''
    _UI_SESSION_ACTIVE=1
    printf '\033[?1049h'
}

ui_session_end() {
    [[ "${_UI_SESSION_ACTIVE:-0}" == 1 ]] || return 0
    _UI_SESSION_ACTIVE=0
    printf '\033[?2026l\033[0m\033[?25h\033[?1049l'
    [ -n "${_UI_SESSION_STTY:-}" ] && stty "$_UI_SESSION_STTY" 2>/dev/null || true
}

_ui_teardown() {
    [ "${_ui_active:-0}" = 1 ] || return 0
    _ui_active=0
    local image_id
    for image_id in "${_ui_image_current:-0}" "${_ui_image_pending:-0}"; do
        ((image_id > 0)) && printf '\033_Ga=d,d=I,i=%s,q=2\033\\' "$image_id"
    done
    [ -n "${_ui_workdir:-}" ] && rm -rf -- "$_ui_workdir"
    printf '\033[?2026l\033[0m\033[?25h'
    [[ "${_UI_SESSION_ACTIVE:-0}" == 1 ]] || printf '\033[?1049l'
    [ -n "$_ui_stty" ] && stty "$_ui_stty" 2>/dev/null || true
    trap - EXIT INT TERM HUP WINCH
    local saved
    for saved in "$_ui_old_exit" "$_ui_old_int" "$_ui_old_term" "$_ui_old_hup" "$_ui_old_winch"; do
        [ -n "$saved" ] && eval "$saved"
    done
    return 0
}

_ui_draw_menu() {
    # Prepare the complete frame off screen, including slow image conversion.
    # Never clear the visible preview while waiting for its replacement.
    if [[ "$_ui_frame_geometry" != "$_ui_width:$_ui_height" ]]; then
        # Clear before uploading: Kitty can discard uploaded data on ED2.
        printf '\033[H\033[2J'
        _ui_frame_geometry="$_ui_width:$_ui_height"
    fi
    : > "$_ui_upload_file"
    _ui_image_pending=0
    _ui_render_menu > "$_ui_frame_file"
    # Upload a new Kitty image WITHOUT placing it while the old one stays visible.
    [ -s "$_ui_upload_file" ] && cat -- "$_ui_upload_file"
    cat -- "$_ui_frame_file"
    _ui_image_current=$_ui_image_pending
}

_ui_render_menu() {
    local list_start=5 cols=1 logical_rows sel_row visible start end row index col cell_w
    local list_width=$_ui_width preview_x=1 preview_y=0 preview_cols=$((_ui_width - 2)) side=0
    _ui_preview_rows=0
    if [ -n "$_ui_preview" ] && { ((_ui_height >= 16)) || { [[ "$_ui_preview" == preview_color_sample ]] && ((_ui_height >= 12)); }; }; then
        if ((_ui_width >= 110)); then
            side=1; list_width=$((_ui_width * 55 / 100))
            preview_x=$((list_width + 3)); preview_y=4
            preview_cols=$((_ui_width - preview_x - 1))
            _ui_preview_rows=$((_ui_height - 7))
            ((_ui_preview_rows > 14)) && _ui_preview_rows=14
        else
            _ui_preview_rows=3
            if [[ "$_ui_preview" == preview_color_sample ]] && ((_ui_height >= 14)); then _ui_preview_rows=5; fi
            [[ "$_ui_preview" == preview_image_logo ]] && _ui_preview_rows=6
            [[ "$_ui_preview" != preview_color_sample ]] && ((_ui_height < 24 && _ui_preview_rows > 3)) && _ui_preview_rows=3
        fi
    fi
    ((_ui_two && list_width >= 68)) && cols=2
    logical_rows=$(((_ui_total + cols - 1) / cols))
    visible=$((_ui_height - list_start - 1))
    ((!side && _ui_preview_rows > 0)) && visible=$((visible - _ui_preview_rows - 2))
    ((visible < 1)) && visible=1
    ((visible > logical_rows)) && visible=$logical_rows
    sel_row=$((_ui_selected % logical_rows))
    ((_ui_start > sel_row)) && _ui_start=$sel_row
    ((sel_row >= _ui_start + visible)) && _ui_start=$((sel_row - visible + 1))
    ((_ui_start > logical_rows - visible)) && _ui_start=$((logical_rows - visible))
    ((_ui_start < 0)) && _ui_start=0
    start=$_ui_start; end=$((start + visible)); cell_w=$((list_width / cols))
    printf '\033[?2026h'
    if ((_ui_height < 8 || _ui_width < 20)); then
        _ui_put 1 1 "$((_ui_width - 1))" 'Resize terminal; Q to back'
        printf '\033[?2026l'
        return
    fi
    _ui_put 1 1 "$((_ui_width - 1))" 'TermFetch Studio 1.0.0' '\033[1;36m'
    # Navigation belongs in the footer, not in every menu title.
    local title=${_ui_prompt%% \[*}
    _ui_put 3 1 "$((_ui_width - 1))" "$title" '\033[1;33m'
    local UI_OPTION_ICON UI_OPTION_LABEL style x y
    for ((row=list_start; row<list_start + visible; row++)); do
        printf '\033[%d;1H%*s' "$row" "$((list_width - 1))" ''
    done
    for ((row=start; row<end; row++)); do
        for ((col=0; col<cols; col++)); do
            index=$((row + col * logical_rows))
            ((index >= _ui_total)) && continue
            x=$((col * cell_w + 1)); y=$((list_start + row - start)); style='\033[0m'
            if ((index == _ui_selected)); then
                _ui_put "$y" "$x" 2 '▶' '\033[1;32m'
                style='\033[7m'
            fi
            ui_option_parts "${_ui_options[index]}"
            _ui_put "$y" "$((x + 3))" 2 "$UI_OPTION_ICON"
            _ui_put "$y" "$((x + 7))" "$((cell_w - 8))" "$UI_OPTION_LABEL" "$style"
        done
    done
    if ((_ui_preview_rows > 0)); then
        ((!side)) && preview_y=$((list_start + visible + 1))
        local UI_PREVIEW_COLS=$preview_cols UI_PREVIEW_ROWS=$_ui_preview_rows
        local UI_PREVIEW_X=$preview_x UI_PREVIEW_Y=$((preview_y + 1))
        if [[ "$_ui_preview" != preview_image_logo ]]; then
            for ((row=UI_PREVIEW_Y; row<UI_PREVIEW_Y + UI_PREVIEW_ROWS; row++)); do
                printf '\033[%d;%dH%*s' "$row" "$preview_x" "$preview_cols" ''
            done
        fi
        _ui_put "$preview_y" "$preview_x" "$preview_cols" 'Preview' '\033[1;33m'
        printf '\033[%d;%dH' "$UI_PREVIEW_Y" "$preview_x"
        if [[ "$_ui_preview" == preview_image_logo || "$_ui_preview" == preview_theme || "$_ui_preview" == preview_color_sample ]]; then
            "$_ui_preview" "$_ui_selected" 2>/dev/null || true
        else
            local output line n=0
            output=$("$_ui_preview" "$_ui_selected" 2>/dev/null | head -n "$_ui_preview_rows")
            output=$(printf '%s' "$output" | sed $'s/\033\\[[0-9;]*[A-Za-z]//g')
            while IFS= read -r line && ((n < _ui_preview_rows)); do
                _ui_put "$((UI_PREVIEW_Y + n))" "$preview_x" "$preview_cols" "$line"
                n=$((n + 1))
            done <<< "$output"
        fi
    fi
    printf '\033[%d;1H\033[2K\033[%d;1H\033[2K' "$((_ui_height - 1))" "$_ui_height"
    _ui_put "$((_ui_height - 1))" 1 "$((_ui_width - 1))" "Items $((start + 1))-$end/$logical_rows | Selected $((_ui_selected + 1))/$_ui_total" '\033[2m'
    local keys='↑↓ Select · Enter Apply · Q Back'
    ((cols == 2)) && keys='↑↓←→ Select · Enter Apply · Q Back'
    ((_ui_width < 36)) && keys='↑↓ Enter Q:Back'
    ((_ui_height < 16 && _ui_width >= 50 && ${#_ui_preview} > 0)) && keys+=' | preview hidden'
    [ -n "${UI_MENU_KEYS:-}" ] && keys=$UI_MENU_KEYS
    if [[ "${UI_MENU_ON_KEY:-}" == color_editor_key ]] && ((_ui_width < 60)); then
        keys='↑↓ Field ←→ Color Enter Q:Cancel'
    fi
    _ui_put "$_ui_height" 1 "$((_ui_width - 1))" "$keys" '\033[2m'
    if ((_ui_image_current > 0 && _ui_image_current != _ui_image_pending)); then
        printf '\033_Ga=d,d=I,i=%s,q=2\033\\' "$_ui_image_current"
    fi
    printf '\033[?2026l'
    _ui_cols=$cols; _ui_rows=$logical_rows
}

_ui_menu() {
    local _ui_two=$1 _ui_prompt=$2
    shift 2
    local _ui_options=("$@") _ui_preview='' _ui_total=$# _ui_selected=0 _ui_start=0
    if (($# > 0)) && declare -F "${!#}" >/dev/null; then
        _ui_preview=${!#}; unset '_ui_options[-1]'; _ui_total=$((_ui_total - 1))
    fi
    ((_ui_total > 0)) && [ -t 0 ] && [ -t 1 ] || return 255
    local _ui_width=0 _ui_height=0 _ui_cols=1 _ui_rows=1 _ui_active=1 _ui_resize=1 _ui_signal=0
    local _ui_workdir _ui_frame_file _ui_upload_file _ui_frame_geometry=''
    local _ui_image_current=0 _ui_image_pending=0
    _ui_workdir=$(mktemp -d "${TMPDIR:-/tmp}/tfs-menu.XXXXXXXX") || return 255
    _ui_frame_file="$_ui_workdir/frame"; _ui_upload_file="$_ui_workdir/upload"
    local _ui_stty _ui_old_exit _ui_old_int _ui_old_term _ui_old_hup _ui_old_winch
    _ui_stty=$(stty -g 2>/dev/null) || _ui_stty=''
    _ui_old_exit=$(trap -p EXIT); _ui_old_int=$(trap -p INT); _ui_old_term=$(trap -p TERM)
    _ui_old_hup=$(trap -p HUP); _ui_old_winch=$(trap -p WINCH)
    trap '_ui_teardown' EXIT
    trap '_ui_signal=130' INT
    trap '_ui_signal=143' TERM
    trap '_ui_signal=129' HUP
    trap '_ui_resize=1' WINCH
    # Keep input mode stable across resize/poll cycles; per-read mode changes
    # can discard a key arriving between redraw and the next read.
    stty -echo -icanon min 1 time 0 2>/dev/null || { _ui_teardown; return 255; }
    [[ "${_UI_SESSION_ACTIVE:-0}" == 1 ]] || printf '\033[?1049h'
    printf '\033[?25l'
    local key rest status w h result=255 pending='' chunk
    while true; do
        ((_ui_signal)) && break
        read -r w h < <(ui_terminal_size)
        if ((_ui_resize || w != _ui_width || h != _ui_height)); then
            _ui_width=$w; _ui_height=$h; _ui_resize=0
            _ui_draw_menu
        fi
        if [[ -z "$pending" ]]; then
            # Read raw input in a short batch without Bash's per-character
            # terminal-mode changes. Timed reads retain their partial input.
            IFS= read -r -t 0.1 -d '' pending; status=$?
            ((_ui_signal)) && break
            [[ -z "$pending" ]] && { ((status == 1)) && break; continue; }
        fi
        key=${pending:0:1}; pending=${pending:1}
        if [[ "$key" == $'\004' || "$key" == q || "$key" == Q ]]; then break
        elif ((_ui_height < 8 || _ui_width < 20)); then continue
        elif [[ "$key" == $'\n' || "$key" == $'\r' ]]; then
            if [ -n "${UI_MENU_ON_KEY:-}" ] && "$UI_MENU_ON_KEY" enter; then _ui_resize=1; continue; fi
            result=$_ui_selected; break
        elif [[ "$key" == $'\033' ]]; then
            if ((${#pending} < 2)); then
                chunk=''; IFS= read -r -t 0.1 -d '' chunk || true
                pending+=$chunk
            fi
            [[ -z "$pending" ]] && break
            rest=${pending:0:2}; pending=${pending:2}
            local row=$((_ui_selected % _ui_rows)) base=$((_ui_selected / _ui_rows * _ui_rows)) count=$_ui_rows
            ((base + count > _ui_total)) && count=$((_ui_total - base))
            local old_selected=$_ui_selected direction=''
            case "$rest" in '[C') direction=right ;; '[D') direction=left ;; esac
            if [ -n "$direction" ] && [ -n "${UI_MENU_ON_KEY:-}" ] && "$UI_MENU_ON_KEY" "$direction"; then
                _ui_resize=1; continue
            fi
            case "$rest" in
                '[A') _ui_selected=$((base + (row + count - 1) % count)) ;;
                '[B') _ui_selected=$((base + (row + 1) % count)) ;;
                '[C') ((_ui_cols == 2 && _ui_selected < _ui_rows && _ui_selected + _ui_rows < _ui_total)) && _ui_selected=$((_ui_selected + _ui_rows)) ;;
                '[D') ((_ui_cols == 2 && _ui_selected >= _ui_rows)) && _ui_selected=$((_ui_selected - _ui_rows)) ;;
            esac
            ((_ui_selected != old_selected)) && _ui_resize=1
        fi
    done
    _ui_teardown
    ((_ui_signal)) && exit "$_ui_signal"
    return "$result"
}

select_option() { _ui_menu 0 "$@"; }
select_option_2col() { _ui_menu 1 "$@"; }
select_option_2col_with_preview() { _ui_menu 1 "$@"; }

# Preview fastfetch preset function
preview_fastfetch_preset() {
    local selection=$1
    
    # Preset mapping (same order as in fastfetch_menu)
    local presets=(full minimal focused developer gaming custom)
    
    # Don't preview if selection is "Back" or other options
    if [ $selection -ge ${#presets[@]} ] || [ $selection -lt 0 ]; then
        return
    fi
    
    local preset="${presets[$selection]}"
    
    # Show text-based preview without modifying any config files
    echo ""
    echo -e "\033[1;33mPreview:\033[0m"
    
    case "$preset" in
        "full")
            echo "  📋 Full Info: OS, Kernel, Packages, Shell, WM, DE, Theme"
            echo "      CPU, GPU, Memory, Swap, Disk, Battery"
            echo "      Local IP, Public IP, WiFi, DateTime, Uptime"
            ;;
        "minimal")
            echo "  📝 Minimal: OS, Kernel, Shell, Uptime"
            ;;
        "focused")
            echo "  🎯 Focused: OS, Kernel, Packages, Uptime"
            echo "      Terminal, Shell, CPU, Memory, Disk"
            ;;
        "developer")
            echo "  💻 Developer: OS, Kernel, Packages, Shell"
            echo "      Terminal, Font, DE, WM"
            echo "      CPU, GPU, Memory, Swap, Disk"
            echo "      Local IP, Public IP"
            ;;
        "gaming")
            echo "  🎮 Gaming: OS, Kernel, Uptime"
            echo "      CPU, GPU, Memory, Swap, Disk, Display, Battery"
            echo "      Local IP, WiFi"
            ;;
        "custom")
            echo "  🧩 Custom: Your own module selection"
            echo "      Use 'Configure Custom Preset' to select modules"
            ;;
    esac
}

# Preview theme function for theme menu
preview_theme() {
    local selection=$1
    local themes=(dracula nord gruvbox-dark tokyo-night one-dark oceanic-next monochrome
                  gruvbox-light solarized-light one-light ayu-light synthwave monokai-pro palenight sakura lavender candy
                  matrix cyberpunk naruto pokemon spiderman doom valorant minecraft)
    ((selection >= 0 && selection < ${#themes[@]})) || return 0
    local theme=${themes[selection]} budget=${UI_PREVIEW_COLS:-60}
    apply_terminal_colors "$theme" 2>/dev/null || true
    ui_clip_text "Theme: $theme" "$budget"
    if ((${UI_PREVIEW_ROWS:-3} >= 2)); then
        printf '\033[%d;%dH' "$((${UI_PREVIEW_Y:-1} + 1))" "${UI_PREVIEW_X:-1}"
        local color
        for ((color=31; color<=37 && (color-30)*3<=budget; color++)); do printf '\033[%sm██ ' "$color"; done
        printf '\033[0m'
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

terminal_from_ancestors() {
    # Follow this shell's process tree, stopping at the nearest terminal.
    # This also overrides a stale KITTY_WINDOW_ID inherited by another terminal.
    local pid=$PPID comm line parent i
    for ((i=0; i<16; i++)); do
        [[ "$pid" =~ ^[0-9]+$ ]] && [ "$pid" -gt 1 ] || break
        [ -r "/proc/$pid/comm" ] && [ -r "/proc/$pid/status" ] || break
        IFS= read -r comm < "/proc/$pid/comm"
        case "$comm" in
            kitty|ghostty) echo kitty; return ;;
            konsole) echo konsole; return ;;
            wezterm-gui|wezterm) echo wezterm; return ;;
            gnome-terminal*|kgx|alacritty|xterm|xfce4-terminal*|foot)
                echo none; return ;;
        esac
        parent=""
        while IFS= read -r line; do
            case "$line" in PPid:*) parent=${line#PPid:}; parent=${parent//[[:space:]]/}; break ;; esac
        done < "/proc/$pid/status"
        [ -n "$parent" ] && [ "$parent" != "$pid" ] || break
        pid=$parent
    done
}

check_terminal_image_support() {
    # Only signals belonging to this session are useful. A process elsewhere on
    # the machine says nothing about the terminal controlling this shell.
    local ancestor_terminal
    ancestor_terminal=$(terminal_from_ancestors)
    if [ -n "$ancestor_terminal" ]; then
        echo "$ancestor_terminal"
        return
    fi
    if [ -n "${KITTY_WINDOW_ID:-}" ] || [ "${TERM:-}" = "xterm-kitty" ]; then
        echo "kitty"
    elif [ -n "${GHOSTTY_RESOURCES_DIR:-}" ] || [ "${TERM_PROGRAM:-}" = "ghostty" ]; then
        echo "kitty"
    elif [ -n "${WEZTERM_EXECUTABLE:-}" ] || [ "${TERM_PROGRAM:-}" = "WezTerm" ]; then
        echo "wezterm"
    elif [ "${TERM_PROGRAM:-}" = "iTerm.app" ]; then
        echo "iterm2"
    elif [ -n "${KONSOLE_VERSION:-}" ] || [ -n "${KONSOLE_DBUS_SERVICE:-}" ]; then
        echo "konsole"
    elif [ "${TERM_PROGRAM:-}" = "foot" ] || [ "${TERM:-}" = "foot" ]; then
        echo "foot"
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
                text) echo "#ebdbb2" ;;
                background) echo "#282828" ;;
                *) echo "#d79921" ;;
            esac
            ;;
        tokyo-night)
            case $color_type in
                primary) echo "#7aa2f7" ;;
                secondary) echo "#bb9af7" ;;
                accent) echo "#9ece6a" ;;
                text) echo "#c0caf5" ;;
                background) echo "#1a1b26" ;;
                *) echo "#7aa2f7" ;;
            esac
            ;;
        one-dark)
            case $color_type in
                primary) echo "#61afef" ;;
                secondary) echo "#c678dd" ;;
                accent) echo "#98c379" ;;
                text) echo "#abb2bf" ;;
                background) echo "#282c34" ;;
                *) echo "#61afef" ;;
            esac
            ;;
        synthwave)
            case $color_type in
                primary) echo "#36f9f6" ;;
                secondary) echo "#ff7edb" ;;
                accent) echo "#72f1b8" ;;
                text) echo "#ffffff" ;;
                background) echo "#2a2139" ;;
                *) echo "#36f9f6" ;;
            esac
            ;;
        oceanic-next)
            case $color_type in
                primary) echo "#6699cc" ;;
                secondary) echo "#5fb3b3" ;;
                accent) echo "#99c794" ;;
                text) echo "#c0c5ce" ;;
                background) echo "#1b2b34" ;;
                *) echo "#6699cc" ;;
            esac
            ;;
        monochrome)
            case $color_type in
                primary) echo "#a0a0a0" ;;
                secondary) echo "#8e8e8e" ;;
                accent) echo "#7c7c7c" ;;
                text) echo "#ffffff" ;;
                background) echo "#000000" ;;
                *) echo "#a0a0a0" ;;
            esac
            ;;
        gruvbox-light)
            case $color_type in
                primary) echo "#ff69b4" ;;    # hot pink
                secondary) echo "#dda0dd" ;;  # plum
                accent) echo "#98fb98" ;;     # pale green
                text) echo "#000000" ;;       # pure black for maximum contrast
                background) echo "#fff0f5" ;; # lavender blush
                *) echo "#ff69b4" ;;
            esac
            ;;
        solarized-light)
            case $color_type in
                primary) echo "#ff85ad" ;;    # soft pink
                secondary) echo "#d9b3ff" ;;  # light purple
                accent) echo "#a8e6cf" ;;     # mint green
                text) echo "#000000" ;;       # pure black for maximum contrast
                background) echo "#f8f4ff" ;; # very light lavender
                *) echo "#ff85ad" ;;
            esac
            ;;
        one-light)
            case $color_type in
                primary) echo "#ff9ecf" ;;    # rose pink
                secondary) echo "#e6b3ff" ;;  # light violet
                accent) echo "#b3ffb3" ;;     # light green
                text) echo "#000000" ;;       # pure black for maximum contrast
                background) echo "#fff5f8" ;; # very light pink
                *) echo "#ff9ecf" ;;
            esac
            ;;
        ayu-light)
            case $color_type in
                primary) echo "#ff8fa3" ;;    # soft pink
                secondary) echo "#e6a8e6" ;;  # light pink
                accent) echo "#a8e6a8" ;;     # light mint green
                text) echo "#000000" ;;       # pure black for maximum contrast
                background) echo "#fef7ff" ;; # very light lavender
                *) echo "#ff8fa3" ;;
            esac
            ;;
        monokai-pro)
            case $color_type in
                primary) echo "#fc9867" ;;
                secondary) echo "#ab9df2" ;;
                accent) echo "#a9dc76" ;;
                text) echo "#fcfcfa" ;;
                background) echo "#2d2a2e" ;;
                *) echo "#fc9867" ;;
            esac
            ;;
        palenight)
            case $color_type in
                primary) echo "#82aaff" ;;
                secondary) echo "#c792ea" ;;
                accent) echo "#c3e88d" ;;
                text) echo "#a6accd" ;;
                background) echo "#292d3e" ;;
                *) echo "#82aaff" ;;
            esac
            ;;
        sakura)
            case $color_type in
                primary) echo "#ff69b4" ;;
                secondary) echo "#ff1493" ;;
                accent) echo "#98d8c8" ;;
                text) echo "#2d2a2e" ;;
                background) echo "#fff5f7" ;;
                *) echo "#ff69b4" ;;
            esac
            ;;
        lavender)
            case $color_type in
                primary) echo "#9370db" ;;
                secondary) echo "#ba55d3" ;;
                accent) echo "#b19cd9" ;;
                text) echo "#2d2a2e" ;;
                background) echo "#f5f3ff" ;;
                *) echo "#9370db" ;;
            esac
            ;;
        candy)
            case $color_type in
                primary) echo "#ff85ad" ;;
                secondary) echo "#d9b3ff" ;;
                accent) echo "#c2f0c2" ;;
                text) echo "#5d4e6d" ;;
                background) echo "#ffe6f0" ;;
                *) echo "#ff85ad" ;;
            esac
            ;;
        strawberry)
            case $color_type in
                primary) echo "#ff69b4" ;;    # hot pink
                secondary) echo "#ffa0b4" ;;  # strawberry pink
                accent) echo "#ffb6c1" ;;     # light pink
                text) echo "#000000" ;;       # pure black for maximum contrast
                background) echo "#fff0f5" ;; # strawberry cream
                *) echo "#ff69b4" ;;
            esac
            ;;
        *)
            echo "#569cd6" ;;
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
                ui_session_end
                clear
                # Run fastfetch directly with current config
                if [ -f "$CONFIG_DIR/fastfetch.jsonc" ]; then
                    if [ -x "$HOME/.local/bin/tfs-fastfetch" ]; then
                        "$HOME/.local/bin/tfs-fastfetch"
                    else
                        fastfetch --config "$CONFIG_DIR/fastfetch.jsonc"
                    fi
                fi
                echo -e "\n\033[1;32m✨ Thank you for using TermFetch Studio! ✨\033[0m\n"
                exit 0
                ;;
        esac
    done
}

# Color settings use the single implementation below.

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
        head -n 8 "$logo_path" 2>/dev/null | cut -c1-40 || echo -e "\033[1;31mPreview failed\033[0m"
    else
        echo -e "\033[1;33mLogo file not found: $logo_path\033[0m"
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
ui_clear_preview_text() {
    local n
    for ((n=0; n<${UI_PREVIEW_ROWS:-0}; n++)); do
        printf '\033[%d;%dH%*s' "$((${UI_PREVIEW_Y:-1} + n))" "${UI_PREVIEW_X:-1}" "${UI_PREVIEW_COLS:-1}" ''
    done
    printf '\033[%d;%dH' "${UI_PREVIEW_Y:-1}" "${UI_PREVIEW_X:-1}"
}

preview_image_logo() {
    local selected=$1 width=${UI_PREVIEW_COLS:-28} height=${UI_PREVIEW_ROWS:-5}
    ((selected >= 0 && selected < ${#PREVIEW_PRESET_IMAGES[@]})) || { ui_clear_preview_text; return 0; }
    local image_path=${PREVIEW_PRESET_IMAGES[selected]} format=symbols cw ch header image_w image_h
    [ -f "$image_path" ] || { ui_clear_preview_text; ui_clip_text 'Image unavailable' "$width"; return 0; }
    if command -v chafa >/dev/null 2>&1; then
        read -r cw ch < <(get_terminal_cell_dimensions)
        if [ "$(check_terminal_image_support)" = kitty ] && [ -z "${TMUX:-}${STY:-}" ]; then
            _ui_image_pending=$((2147483648 + RANDOM * 65536 + RANDOM * 2))
            # Chafa emits base64 Kitty chunks. Keep the payload byte-for-byte;
            # change only the first control header to upload without display.
            if (set -o pipefail; chafa -f kitty --passthrough=none --relative=on --animate=off --font-ratio "$cw/$ch" -s "${width}x${height}" "$image_path" </dev/null 2>/dev/null |
                sed -E $'s/\033\\[[0-9;?]*[A-Za-z]//g; s/\033D//g' |
                sed "s/_Ga=T,/_Ga=t,i=${_ui_image_pending},/") > "$_ui_upload_file"; then
                header=$(head -c 200 "$_ui_upload_file")
                if [[ "$header" =~ c=([0-9]+),r=([0-9]+) ]]; then
                    image_w=${BASH_REMATCH[1]}; image_h=${BASH_REMATCH[2]}
                    printf '\033_Ga=p,i=%s,p=1,c=%s,r=%s,C=1,q=2\033\\' "$_ui_image_pending" "$image_w" "$image_h"
                    return
                fi
            fi
            _ui_image_pending=0; : > "$_ui_upload_file"
        fi
        ui_clear_preview_text
        chafa -f "$format" --relative=on --animate=off --font-ratio "$cw/$ch" -s "${width}x${height}" "$image_path" </dev/null 2>/dev/null || ui_clip_text 'Preview unavailable' "$width"
    else
        ui_clear_preview_text
        ui_clip_text "Image: ${image_path##*/} (install chafa for preview)" "$width"
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
                if has_image_magick &> /dev/null; then
                    image_convert "$selected_image" PNG32:"$target_image" 2>/dev/null
                else
                    cp "$selected_image" "$IMAGES_DIR/preset_logo.$ext_lower"
                    target_image="$IMAGES_DIR/preset_logo.$ext_lower"
                fi
            else
                cp "$selected_image" "$target_image"
            fi

            # Resize if too large
            if has_image_magick &> /dev/null && [ -f "$target_image" ]; then
                local img_width=$(image_identify -format "%w" "$target_image" 2>/dev/null)
                if [ -n "$img_width" ] && [ "$img_width" -gt 800 ]; then
                    image_convert "$target_image" -resize 800x PNG32:"$target_image" 2>/dev/null
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
                        if has_image_magick &> /dev/null; then
                            echo -e "\033[1;33m🔄 Converting $ext_lower to PNG...\033[0m"
                            image_convert "$image_path" PNG32:"$IMAGES_DIR/custom_logo.png" 2>/dev/null
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
                    if has_image_magick &> /dev/null && [ -f "$IMAGES_DIR/custom_logo.png" ]; then
                        # Get image dimensions
                        img_width=$(image_identify -format "%w" "$IMAGES_DIR/custom_logo.png" 2>/dev/null)
                        if [ -n "$img_width" ] && [ "$img_width" -gt 800 ]; then
                            echo -e "\033[1;33m📐 Resizing large image...\033[0m"
                            image_convert "$IMAGES_DIR/custom_logo.png" -resize 800x PNG32:"$IMAGES_DIR/custom_logo.png" 2>/dev/null
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

logo_size_menu() {
    local current choice size
    current=$(get_config_value fastfetch logo_size)
    select_option "Image Logo Size (Current: ${current:-medium})" 'Small' 'Medium' 'Large' '← Back'
    choice=$?
    case "$choice" in
        0) size=small ;; 1) size=medium ;; 2) size=large ;; *) return ;;
    esac
    set_config_value fastfetch logo_size "$size"
    apply_logo_config
}

logo_menu() {
    while true; do
        local current=$(get_config_value "fastfetch" "logo_type")
        local custom_image=$(get_config_value "fastfetch" "custom_image")
        local terminal_support=$(check_terminal_image_support)
        local size=$(get_config_value fastfetch logo_size)
        local fallback=$(get_config_value "fastfetch" "logo_fallback")
        [ -z "$fallback" ] && fallback="auto"

        local options=(
            "🐧 Auto (Distribution Logo)"
            "📝 ASCII Logos"
            "🖼️ Image Logos"
            "🔄 Image Fallback (Unsupported Terminals): $fallback"
            "❌ No Logo"
            "↔ Image Size: ${size:-medium}"
            "← Back"
        )

        select_option "🖼️  Logo Settings (Current: $current | Terminal: $terminal_support)" "${options[@]}"
        local choice=$?

        [ $choice -eq 6 ] || [ $choice -eq 255 ] && return

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
            5) logo_size_menu ;;
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
        "custom") create_custom_config "$logo_type" "$custom_image" "$theme" ;;
        *) create_full_config "$logo_type" "$custom_image" "$theme" ;;
    esac
    
    # Cleanup: remove invalid/empty source fields for auto/builtin
    if [ -f "$CONFIG_DIR/fastfetch.jsonc" ]; then
        sed -i -E '/"source"[[:space:]]*:[[:space:]]*""/d; /"source"[[:space:]]*:[[:space:]]*"auto"/d' "$CONFIG_DIR/fastfetch.jsonc" 2>/dev/null || true
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
        local demo_mode=$(get_config_value "fastfetch" "demo_mode")
        [ -z "$demo_mode" ] && demo_mode="false"
        
        local demo_status="❌ Off"
        [ "$demo_mode" = "true" ] && demo_status="✅ On"
        
        local options=(
            "🎭 Demo Mode: $demo_status"
            "🔧 Edit Config File"
            "🗑️ Reset All Settings"
            "← Back"
        )

        select_option "⚙️  Advanced Settings (Demo Mode shows fake system info for screenshots)" "${options[@]}"
        local choice=$?

        [ $choice -eq 3 ] || [ $choice -eq 255 ] && return

        case $choice in
            0) toggle_demo_mode ;;
            1) edit_config_file ;;
            2) reset_all_settings ;;
        esac
    done
}

# Toggle demo mode for screenshots (shows fake system info)
toggle_demo_mode() {
    local current=$(get_config_value "fastfetch" "demo_mode")
    if [ "$current" = "true" ]; then
        set_config_value "fastfetch" "demo_mode" "false"
        echo -e "\n\033[1;32m✓ Demo Mode disabled - Real system info will be shown\033[0m"
    else
        set_config_value "fastfetch" "demo_mode" "true"
        echo -e "\n\033[1;33m🎭 Demo Mode enabled - Fake info will be shown for screenshots\033[0m"
    fi
    
    # Re-apply current preset to regenerate config
    local preset=$(get_config_value "fastfetch" "preset")
    [ -z "$preset" ] && preset="full"
    apply_preset "$preset"
    
    sleep 1
}

# Get demo/fake format string for a module type
get_demo_format() {
    local module_type=$1
    case "$module_type" in
        "title") echo "hacker@linuxbox" ;;
        "os") echo "Arch Linux x86_64" ;;
        "kernel") echo "6.12.1-arch1-1" ;;
        "host") echo "Custom Gaming PC" ;;
        "cpu") echo "AMD Ryzen 9 7950X3D (32) @ 5.7GHz" ;;
        "gpu") echo "NVIDIA GeForce RTX 4090" ;;
        "memory") echo "32.00 GiB / 64.00 GiB (50%)" ;;
        "disk") echo "512.00 GiB / 2.00 TiB (25%)" ;;
        "swap") echo "8.00 GiB / 16.00 GiB" ;;
        "localip") echo "192.168.1.100" ;;
        "publicip") echo "203.0.113.42" ;;
        "wifi") echo "MyNetwork (802.11ac)" ;;
        "uptime") echo "5 days, 12 hours, 34 mins" ;;
        "packages") echo "1337 (pacman), 42 (flatpak)" ;;
        "shell") echo "zsh 5.9" ;;
        "terminal") echo "kitty" ;;
        "terminalfont") echo "JetBrainsMono Nerd Font (12pt)" ;;
        "wm") echo "Hyprland" ;;
        "de") echo "none" ;;
        "wmtheme") echo "Catppuccin Mocha" ;;
        "icons") echo "Papirus-Dark" ;;
        "cursor") echo "Bibata-Modern-Ice" ;;
        "display") echo "3840x2160 @ 144Hz" ;;
        "battery") echo "100% [Full]" ;;
        "datetime") echo "2024-12-06 15:30:45" ;;
        "media") echo "Spotify - Daft Punk - Around the World" ;;
        *) echo "" ;;
    esac
}

# Resolve representative inherited colors from the selected generator, in a private
# config copy. Cache once per editor session; navigation never writes user files.
color_preview_base_colors() (
    local original_config=$CONFIG_FILE temporary preset theme group module icon text
    temporary=$(mktemp -d "${TMPDIR:-/tmp}/tfs-colors.XXXXXX") || return 1
    trap 'rm -rf -- "$temporary"' EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    local CONFIG_DIR=$temporary CONFIG_FILE="$temporary/config.ini"
    cp -- "$original_config" "$CONFIG_FILE" || return 1
    set_config_value fastfetch use_theme_colors true || return 1
    for group in os desktop hardware network time; do set_config_value group_colors "$group" inherit || return 1; done
    preset=$(get_config_value fastfetch preset); theme=$(get_config_value colors theme)
    case "$preset" in full|minimal|focused|developer|gaming|custom) ;; *) preset=full ;; esac
    "create_${preset}_config" none "" "${theme:-dracula}" >/dev/null || return 1
    while IFS=$'\t' read -r module icon text; do
        group=$(module_color_group "$module")
        [[ -n "$group" ]] && printf '%s\t%s\t%s\n' "$group" "$icon" "$text"
    done < <(awk -F '"' '
        /"type"[[:space:]]*:/ { module=$4; icon="" }
        /"keyColor"[[:space:]]*:/ { icon=$4 }
        /"outputColor"[[:space:]]*:/ { if (icon != "") printf "%s\t%s\t%s\n", module, icon, $4 }
    ' "$CONFIG_DIR/fastfetch.jsonc")
)

# Color settings menu for customizing fastfetch colors
color_settings_menu() {
    local _color_icon_draft _color_text_draft mode
    local -a _color_group_keys=(os desktop hardware network time)
    local -a _color_group_names=(OS Desktop Hardware Network Time) _color_group_draft
    local group i icon text
    local -A _color_inherit_icons=() _color_inherit_texts=()
    while IFS=$'\t' read -r group icon text; do
        if [[ -z "${_color_inherit_icons[$group]:-}" ]]; then
            _color_inherit_icons[$group]=$icon; _color_inherit_texts[$group]=$text
        fi
    done < <(color_preview_base_colors)
    mode=$(get_config_value fastfetch use_theme_colors)
    _color_icon_draft=$(get_config_value fastfetch key_color)
    _color_text_draft=$(get_config_value fastfetch value_color)
    if [[ "$mode" != false ]]; then _color_icon_draft=theme; _color_text_draft=theme; fi
    local -a _color_values=(theme default 31 32 33 34 35 36 37 90 91 92 93 94 95 96 97)
    local -a _color_names=(Theme Default Red Green Yellow Blue Magenta Cyan White 'Bright Black' 'Bright Red' 'Bright Green' 'Bright Yellow' 'Bright Blue' 'Bright Magenta' 'Bright Cyan' 'Bright White')
    _color_values+=( '#ff8c00' '#ff7f50' '#ff6347' '#ffd700' '#adff2f' '#40e0d0' '#00bfa5' '#87ceeb' '#4169e1' '#6a5acd' '#8a2be2' '#da70d6' '#ff69b4' '#dc143c' '#e6e6fa' '#c0c0c0')
    _color_names+=(Orange Coral Tomato Gold Lime Turquoise Teal Sky 'Royal Blue' Slate Violet Orchid Pink Crimson Lavender Silver)
    for group in "${_color_group_keys[@]}"; do
        _color_group_draft+=("$(get_config_value group_colors "$group")")
    done
    local UI_MENU_ON_KEY=color_editor_key
    local UI_MENU_KEYS='↑↓ Field · ←→ Color · Enter Select · Q Cancel'
    local options=("Icons: $(color_label "$_color_icon_draft")" "Text: $(color_label "$_color_text_draft")")
    for i in "${!_color_group_keys[@]}"; do options+=("${_color_group_names[i]} group: $(color_label "${_color_group_draft[i]:-inherit}")"); done
    options+=('Reset to Theme' 'Save & Back')
    select_option 'Icon & Text Colors' "${options[@]}" preview_color_sample
    local choice=$?
    ((choice == 8)) || return 0
    if [[ "$_color_icon_draft" == theme && "$_color_text_draft" == theme ]]; then mode=true; else mode=false; fi
    set_config_value fastfetch key_color "$_color_icon_draft"
    set_config_value fastfetch value_color "$_color_text_draft"
    set_config_value fastfetch use_theme_colors "$mode"
    for i in "${!_color_group_keys[@]}"; do set_config_value group_colors "${_color_group_keys[i]}" "${_color_group_draft[i]:-inherit}"; done
    apply_color_changes >/dev/null
}

color_label() {
    [[ "$1" == inherit || -z "$1" ]] && { printf Inherit; return; }
    local i
    for i in "${!_color_values[@]}"; do
        [[ "$1" == "${_color_values[i]}" ]] && { printf '%s' "${_color_names[i]}"; return; }
    done
    printf '%s' "$1"
}

color_editor_key() {
    local direction=$1 value i=0 delta=1 group_index=-1
    local -a values=("${_color_values[@]}")
    ((_ui_selected == 8)) && return 1
    if ((_ui_selected == 7)); then
        [[ "$direction" == enter ]] || return 0
        _color_icon_draft=theme; _color_text_draft=theme
        for i in "${!_color_group_keys[@]}"; do _color_group_draft[i]=inherit; done
    else
        if ((_ui_selected >= 2)); then
            group_index=$((_ui_selected - 2)); value=${_color_group_draft[group_index]:-inherit}
            values=(inherit "${values[@]}")
        elif ((_ui_selected == 0)); then value=$_color_icon_draft
        else value=$_color_text_draft; fi
        for i in "${!values[@]}"; do [[ "$value" == "${values[i]}" ]] && break; done
        [[ "$direction" == left ]] && delta=-1
        i=$(((i + delta + ${#values[@]}) % ${#values[@]}))
        if ((group_index >= 0)); then _color_group_draft[group_index]=${values[i]}
        elif ((_ui_selected == 0)); then _color_icon_draft=${values[i]}
        else _color_text_draft=${values[i]}; fi
    fi
    _ui_options[0]="Icons: $(color_label "$_color_icon_draft")"
    _ui_options[1]="Text: $(color_label "$_color_text_draft")"
    for i in "${!_color_group_keys[@]}"; do
        _ui_options[i+2]="${_color_group_names[i]} group: $(color_label "${_color_group_draft[i]:-inherit}")"
    done
    return 0
}

color_sample_sgr() {
    local value=$1 position=$2 theme
    theme=${_color_preview_theme:-}
    if [[ "$value" == theme || -z "$value" ]]; then
        [[ -n "$theme" ]] || theme=$(get_config_value colors theme)
        value=$(get_theme_ansi_color "${theme:-dracula}" "$position")
    fi
    if [[ "$value" =~ ^#[[:xdigit:]]{6}$ ]]; then
        printf '38;2;%d;%d;%d' "$((16#${value:1:2}))" "$((16#${value:3:2}))" "$((16#${value:5:2}))"
    elif [[ "$value" =~ ^(3[0-9]|9[0-7])$ ]]; then printf '%s' "$value"
    else printf '39'; fi
}

preview_color_sample() {
    local icon text n row group_index start=0 override position base_icon base_text
    local _color_preview_theme
    _color_preview_theme=$(get_config_value colors theme)
    base_icon=$(color_sample_sgr "$_color_icon_draft" primary)
    base_text=$(color_sample_sgr "$_color_text_draft" primary)
    local -a labels=('● OS' '● Desktop' '● Hardware' '● Network' '● Time')
    local -a values=('Linux' 'Terminal sample' '8 GiB / 16 GiB' 'Network sample' '12:34')
    local -a positions=(primary secondary accent highlight info)
    # Fixed semantic order. Tiny windows scroll a contiguous slice, never wrap.
    if ((UI_PREVIEW_ROWS < 5 && _ui_selected >= 2 && _ui_selected <= 6)); then
        start=$((_ui_selected - 2 - UI_PREVIEW_ROWS + 1))
        ((start < 0)) && start=0
        ((start > 5 - UI_PREVIEW_ROWS)) && start=$((5 - UI_PREVIEW_ROWS))
    fi
    for ((n=0; n<5 && n<UI_PREVIEW_ROWS; n++)); do
        group_index=$((start + n)); row=$((UI_PREVIEW_Y + n)); position=${positions[group_index]}
        override=${_color_group_draft[group_index]:-inherit}
        if [[ "$override" == inherit ]]; then
            if [[ "$_color_icon_draft" == theme && "$_color_text_draft" == theme ]]; then
                icon=$(color_sample_sgr "${_color_inherit_icons[${_color_group_keys[group_index]}]:-theme}" "$position")
                text=$(color_sample_sgr "${_color_inherit_texts[${_color_group_keys[group_index]}]:-theme}" "$position")
            else
                icon=$base_icon; text=$base_text
            fi
        else
            icon=$(color_sample_sgr "$override" "$position"); text=$icon
        fi
        _ui_put "$row" "$UI_PREVIEW_X" "$UI_PREVIEW_COLS" "${labels[group_index]}" "\033[${icon}m"
        if ((UI_PREVIEW_COLS >= 24)); then
            _ui_put "$row" "$((UI_PREVIEW_X + 13))" "$((UI_PREVIEW_COLS - 13))" "${values[group_index]}" "\033[${text}m"
        fi
    done
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
    
    # Set defaults if values are empty
    [ -z "$theme" ] && theme="dracula"
    [ -z "$logo_type" ] && logo_type="auto"
    
    # Ensure config directory exists
    mkdir -p "$CONFIG_DIR"
    
    # Save defaults if they were not set
    set_config_value "colors" "theme" "$theme"
    set_config_value "fastfetch" "logo_type" "$logo_type"
    
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
logo_size=medium
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
            local tmp_file
            tmp_file=$(mktemp "$CONFIG_DIR/.config.XXXXXX") || return 1
            cp -p -- "$CONFIG_FILE" "$tmp_file" || { rm -f -- "$tmp_file"; return 1; }
            if awk -v section="$section" -v key="$key" -v value="$value" '
                BEGIN { in_section=0 }
                /^\[.*\]$/ {
                    in_section = ($0 == "["section"]")
                }
                in_section && $0 ~ "^"key"=" {
                    print key"="value
                    next
                }
                { print }
            ' "$CONFIG_FILE" > "$tmp_file" && mv -- "$tmp_file" "$CONFIG_FILE"; then
                :
            else
                rm -f -- "$tmp_file"
                printf '%s\n' "Failed to save configuration; existing file preserved" >&2
                return 1
            fi
        else
            sed -i "/^\[$section\]/a $key=$escaped_value" "$CONFIG_FILE"
        fi
    fi
}

create_backup() {
    local backup_file
    for tool in tar gzip mktemp; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            echo "Backup requires $tool" >&2
            return 1
        fi
    done
    mkdir -p "$BACKUP_DIR" || return 1
    backup_file=$(mktemp "$BACKUP_DIR/backup_$(date +%Y%m%d_%H%M%S)_XXXXXX.tar.gz") || return 1
    if tar --exclude='./backups' -czf "$backup_file" -C "$CONFIG_DIR" .; then
        echo -e "\n\033[1;32m✓ Backup created: $backup_file\033[0m"
    else
        rm -f -- "$backup_file"
        echo -e "\n\033[1;31m✗ Backup failed!\033[0m"
        return 1
    fi
}

restore_backup() {
    local backups=() backup_file choice member
    for tool in tar gzip; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            echo "Restore requires $tool" >&2
            return 1
        fi
    done
    for backup_file in "$BACKUP_DIR"/*.tar.gz; do
        [ -f "$backup_file" ] && backups+=("$backup_file")
    done
    
    if [ ${#backups[@]} -eq 0 ]; then
        echo -e "\n\033[1;33m⚠️  No backups found!\033[0m"
        return 1
    fi
    
    echo -e "\n\033[1;33m📂 Available backups:\033[0m"
    for i in "${!backups[@]}"; do
        echo "  $((i+1))) $(basename "${backups[$i]}")"
    done
    
    echo -ne "\n\033[1;36mEnter backup number to restore: \033[0m"
    read -r choice
    
    if [[ "$choice" =~ ^[0-9]+$ ]] && (( 10#$choice >= 1 && 10#$choice <= ${#backups[@]} )); then
        backup_file="${backups[$((10#$choice-1))]}"
        if ! tar -tzf "$backup_file" >/dev/null; then
            print_error "Invalid backup archive"
            return 1
        fi
        while IFS= read -r member; do
            case "$member" in
                /*|../*|*/../*|*/..|./backups|./backups/*)
                    print_error "Unsafe backup entry: $member"
                    return 1 ;;
            esac
        done < <(tar -tzf "$backup_file")
        
        echo -e "\n\033[1;33m🔄 Restoring backup...\033[0m"
        if tar -xzf "$backup_file" -C "$CONFIG_DIR"; then
            echo -e "\033[1;32m✓ Backup restored!\033[0m"
        else
            echo -e "\033[1;31m✗ Restore failed!\033[0m"
            return 1
        fi
    else
        echo -e "\033[1;31m✗ Invalid choice!\033[0m"
        return 1
    fi
}

list_backups() {
    local backups=() backup_file
    for backup_file in "$BACKUP_DIR"/*.tar.gz; do
        [ -f "$backup_file" ] && backups+=("$backup_file")
    done
    
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
            # A valid type is needed in the saved config as well as the runtime copy.
            echo "auto" ;;
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
                    "ascii") echo "auto" ;;
                    "none") echo "none" ;;
                    *) echo "auto" ;;
                esac
            else
                # Terminal supports images - check if custom image exists
                if [ -n "$custom_image" ] && [ -f "$custom_image" ]; then
                    # Use appropriate protocol for this terminal
                    case $terminal_support in
                        "kitty") echo "kitty-direct" ;;
                        "wezterm") echo "kitty-direct" ;;
                        "iterm2"|"konsole") echo "iterm" ;;
                        "foot") echo "sixel" ;;
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

logo_dimensions_json_fields() {
    case "$1" in
        kitty-direct|kitty|iterm|sixel)
            local w h
            read -r w h < <(configured_logo_dimensions "${2:-}" "${3:-14}")
            printf '        "width": %s,\n        "height": %s,\n' "$w" "$h"
            ;;
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
    
    # Use explicit RGB instead of relying on mutable terminal ANSI palettes.
    local rgb
    rgb=$(get_theme_color "$theme" "$color_type")
    if [[ "$rgb" =~ ^#[[:xdigit:]]{6}$ ]]; then
        printf '%s\n' "$rgb"
        return
    fi

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
module_color_group() {
    case "$1" in
        os|kernel|packages|shell|title) echo os ;;
        wm|de|theme|wmtheme|icons|cursor|terminal|terminalfont) echo desktop ;;
        host|cpu|gpu|memory|swap|disk|display|battery) echo hardware ;;
        localip|publicip|wifi|netio) echo network ;;
        datetime|uptime) echo time ;;
    esac
}

get_module_color() {
    local module=$1 theme=$2 fallback=$3 group value position
    group=$(module_color_group "$module")
    [[ -n "$group" ]] || { printf '%s' "$fallback"; return; }
    value=$(get_config_value group_colors "$group")
    case "$value" in
        ''|inherit) printf '%s' "$fallback" ;;
        theme)
            case "$group" in os) position=primary ;; desktop) position=secondary ;; hardware) position=accent ;; network) position=highlight ;; time) position=info ;; esac
            get_theme_ansi_color "$theme" "$position" ;;
        default) printf '39' ;;
        *) if [[ "$value" =~ ^#[[:xdigit:]]{6}$ || "$value" =~ ^(3[0-9]|9[0-7])$ ]]; then printf '%s' "$value"; else printf '%s' "$fallback"; fi ;;
    esac
}

get_custom_fastfetch_color() {
    local field=$1 theme=$2 value
    value=$(get_config_value fastfetch "$field")
    case "$value" in
        theme|'') get_theme_ansi_color "$theme" primary ;;
        default) printf '39\n' ;;
        *) printf '%s\n' "$value" ;;
    esac
}

get_fastfetch_color() {
    if [[ "$(get_config_value fastfetch use_theme_colors)" == false ]]; then
        get_custom_fastfetch_color key_color "$2"
    else
        get_theme_ansi_color "$2" "$1"
    fi
}

get_fastfetch_text_color() {
    if [[ "$(get_config_value fastfetch use_theme_colors)" == false ]]; then
        get_custom_fastfetch_color value_color "$2"
    else
        get_theme_ansi_color "$2" "$1"
    fi
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

# Helper function to generate format line for demo mode
# Returns empty string if demo mode is off, or JSON format line if on
get_demo_format_line() {
    local module_type=$1
    local demo_mode=$(get_config_value "fastfetch" "demo_mode")
    
    if [ "$demo_mode" = "true" ]; then
        local demo_value=$(get_demo_format "$module_type")
        if [ -n "$demo_value" ]; then
            echo "            \"format\": \"$demo_value\","
        fi
    fi
}

# Check if demo mode is active
is_demo_mode() {
    local demo_mode=$(get_config_value "fastfetch" "demo_mode")
    [ "$demo_mode" = "true" ]
}

# Fastfetch JSON colors accept hex; terminal SGR parameters require RGB numbers.
theme_dot_format() {
    local theme=$1 position color
    local -a positions=(primary secondary accent highlight info primary secondary accent)
    for position in "${positions[@]}"; do
        color=$(get_theme_ansi_color "$theme" "$position")
        if [[ "$color" =~ ^#[[:xdigit:]]{6}$ ]]; then
            printf -v color '38;2;%d;%d;%d' "$((16#${color:1:2}))" "$((16#${color:3:2}))" "$((16#${color:5:2}))"
        elif [[ ! "$color" =~ ^[0-9]+(\;[0-9]+)*$ ]]; then
            color=39
        fi
        printf '\\u001b[1;%sm●\\u001b[0m ' "$color"
    done
}

create_full_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3
    local dot_format
    dot_format=$(theme_dot_format "$theme")

    local logo_source=$(get_logo_source "$logo_type" "$custom_image")
    local logo_actual_type=$(get_logo_type "$logo_type" "$custom_image")

    # Get theme-based or custom ANSI colors
    local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme_colors" ] && use_theme_colors="true"

    local color1 color2 color3 color4 color5
    local output_color
    local show_labels=$(get_config_value "fastfetch" "labels")
    [ -z "$show_labels" ] && show_labels="false"
    
    
    if [ "$use_theme_colors" = "false" ]; then
        local custom_key=$(get_custom_fastfetch_color key_color "$theme")
        local custom_value=$(get_custom_fastfetch_color value_color "$theme")
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

    # Demo mode: prepare format overrides for each module
    local demo_mode=$(get_config_value "fastfetch" "demo_mode")
    local fmt_title=""
    local fmtline_os="" fmtline_kernel="" fmtline_packages="" fmtline_shell=""
    local fmtline_wm="" fmtline_de="" fmtline_wmtheme="" fmtline_icons="" fmtline_cursor=""
    local fmtline_terminal="" fmtline_terminalfont="" fmtline_host="" fmtline_cpu="" fmtline_gpu=""
    local fmtline_memory="" fmtline_swap="" fmtline_disk="" fmtline_battery="" fmtline_display=""
    local fmtline_localip="" fmtline_publicip="" fmtline_wifi="" fmtline_datetime="" fmtline_uptime="" fmtline_media=""
    
    if [ "$demo_mode" = "true" ]; then
        fmt_title="hacker@linuxbox"
        fmtline_os="\"format\": \"$(get_demo_format os)\","
        fmtline_kernel="\"format\": \"$(get_demo_format kernel)\","
        fmtline_packages="\"format\": \"$(get_demo_format packages)\","
        fmtline_shell="\"format\": \"$(get_demo_format shell)\","
        fmtline_wm="\"format\": \"$(get_demo_format wm)\","
        fmtline_de="\"format\": \"$(get_demo_format de)\","
        fmtline_wmtheme="\"format\": \"$(get_demo_format wmtheme)\","
        fmtline_icons="\"format\": \"$(get_demo_format icons)\","
        fmtline_cursor="\"format\": \"$(get_demo_format cursor)\","
        fmtline_terminal="\"format\": \"$(get_demo_format terminal)\","
        fmtline_terminalfont="\"format\": \"$(get_demo_format terminalfont)\","
        fmtline_host="\"format\": \"$(get_demo_format host)\","
        fmtline_cpu="\"format\": \"$(get_demo_format cpu)\","
        fmtline_gpu="\"format\": \"$(get_demo_format gpu)\","
        fmtline_memory="\"format\": \"$(get_demo_format memory)\","
        fmtline_swap="\"format\": \"$(get_demo_format swap)\","
        fmtline_disk="\"format\": \"$(get_demo_format disk)\","
        fmtline_battery="\"format\": \"$(get_demo_format battery)\","
        fmtline_display="\"format\": \"$(get_demo_format display)\","
        fmtline_localip="\"format\": \"$(get_demo_format localip)\","
        fmtline_publicip="\"format\": \"$(get_demo_format publicip)\","
        fmtline_wifi="\"format\": \"$(get_demo_format wifi)\","
        fmtline_datetime="\"format\": \"$(get_demo_format datetime)\","
        fmtline_uptime="\"format\": \"$(get_demo_format uptime)\","
        fmtline_media="\"format\": \"$(get_demo_format media)\","
    else
        fmt_title="{user-name-colored}@{host-name-colored}"
    fi

    cat > "$CONFIG_DIR/fastfetch.jsonc" << EOF
{
    // TermFetch Studio - Full Info Preset
    // Theme: $theme
    "\$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",

    "logo": {
        "type": "$logo_actual_type",
        "source": "$logo_source",
$(logo_dimensions_json_fields "$logo_actual_type" "$logo_source" 14)
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
            "format": "$fmt_title",
            "key": ""
        },
        "break",
        {
            "type": "os",
            $fmtline_os
            "key": " ├ 󰣇 ",
            "keyColor": "$(get_module_color "os" "$theme" "$color1")",
            "outputColor": "$(get_module_color "os" "$theme" "$tcolor1")",
            "keyWidth": 10
        },
        {
            "type": "kernel",
            $fmtline_kernel
            "key": " ├ 󰌢 ",
            "keyColor": "$(get_module_color "kernel" "$theme" "$color1")",
            "outputColor": "$(get_module_color "kernel" "$theme" "$tcolor1")",
            "keyWidth": 10
        },
        {
            "type": "packages",
            $fmtline_packages
            "key": " ├ 󰏖 ",
            "keyColor": "$(get_module_color "packages" "$theme" "$color1")",
            "outputColor": "$(get_module_color "packages" "$theme" "$tcolor1")",
            "keyWidth": 10
        },
        {
            "type": "shell",
            $fmtline_shell
            "key": " ├ 󰆍 ",
            "keyColor": "$(get_module_color "shell" "$theme" "$color1")",
            "outputColor": "$(get_module_color "shell" "$theme" "$tcolor1")",
            "keyWidth": 10
        },
        "break",
        {
            "type": "wm",
            $fmtline_wm
            "key": " ├ 󰖲 ",
            "keyColor": "$(get_module_color "wm" "$theme" "$color2")",
            "outputColor": "$(get_module_color "wm" "$theme" "$tcolor2")",
            "keyWidth": 10
        },
        {
            "type": "de",
            $fmtline_de
            "key": " ├ 󱂬 ",
            "keyColor": "$(get_module_color "de" "$theme" "$color2")",
            "outputColor": "$(get_module_color "de" "$theme" "$tcolor2")",
            "keyWidth": 10
        },
        {
            "type": "wmtheme",
            $fmtline_wmtheme
            "key": " ├ 󰉼 ",
            "keyColor": "$(get_module_color "wmtheme" "$theme" "$color2")",
            "outputColor": "$(get_module_color "wmtheme" "$theme" "$tcolor2")",
            "keyWidth": 10
        },
        {
            "type": "icons",
            $fmtline_icons
            "key": " ├ 󰀻 ",
            "keyColor": "$(get_module_color "icons" "$theme" "$color2")",
            "outputColor": "$(get_module_color "icons" "$theme" "$tcolor2")",
            "keyWidth": 10
        },
        {
            "type": "cursor",
            $fmtline_cursor
            "key": " ├ 󰘔 ",
            "keyColor": "$(get_module_color "cursor" "$theme" "$color2")",
            "outputColor": "$(get_module_color "cursor" "$theme" "$tcolor2")",
            "keyWidth": 10
        },
        {
            "type": "terminal",
            $fmtline_terminal
            "key": " ├ 󰆍 ",
            "keyColor": "$(get_module_color "terminal" "$theme" "$color2")",
            "outputColor": "$(get_module_color "terminal" "$theme" "$tcolor2")",
            "keyWidth": 10
        },
        {
            "type": "terminalfont",
            $fmtline_terminalfont
            "key": " ├ 󰛖 ",
            "keyColor": "$(get_module_color "terminalfont" "$theme" "$color2")",
            "outputColor": "$(get_module_color "terminalfont" "$theme" "$tcolor2")",
            "keyWidth": 10
        },
        "break",
        {
            "type": "host",
            $fmtline_host
            "key": " ├ 󰌢 ",
            "keyColor": "$(get_module_color "host" "$theme" "$color3")",
            "outputColor": "$(get_module_color "host" "$theme" "$tcolor3")",
            "keyWidth": 10
        },
        {
            "type": "cpu",
            $fmtline_cpu
            "key": " ├ 󰻠 ",
            "keyColor": "$(get_module_color "cpu" "$theme" "$color3")",
            "outputColor": "$(get_module_color "cpu" "$theme" "$tcolor3")",
            "keyWidth": 10
        },
        {
            "type": "gpu",
            $fmtline_gpu
            "key": " ├ 󰢮 ",
            "keyColor": "$(get_module_color "gpu" "$theme" "$color3")",
            "outputColor": "$(get_module_color "gpu" "$theme" "$tcolor3")",
            "keyWidth": 10
        },
        {
            "type": "memory",
            $fmtline_memory
            "key": " ├ 󰍛 ",
            "keyColor": "$(get_module_color "memory" "$theme" "$color3")",
            "outputColor": "$(get_module_color "memory" "$theme" "$tcolor3")",
            "keyWidth": 10
        },
        {
            "type": "swap",
            $fmtline_swap
            "key": " ├ 󰓡 ",
            "keyColor": "$(get_module_color "swap" "$theme" "$color3")",
            "outputColor": "$(get_module_color "swap" "$theme" "$tcolor3")",
            "keyWidth": 10
        },
        {
            "type": "disk",
            $fmtline_disk
            "key": " ├ 󰋊 ",
            "keyColor": "$(get_module_color "disk" "$theme" "$color3")",
            "outputColor": "$(get_module_color "disk" "$theme" "$tcolor3")",
            "keyWidth": 10
        },
        {
            "type": "battery",
            $fmtline_battery
            "key": " ├ 󰁹 ",
            "keyColor": "$(get_module_color "battery" "$theme" "$color3")",
            "outputColor": "$(get_module_color "battery" "$theme" "$tcolor3")",
            "keyWidth": 10
        },
        {
            "type": "display",
            $fmtline_display
            "key": " ├ 󰍹 ",
            "keyColor": "$(get_module_color "display" "$theme" "$color3")",
            "outputColor": "$(get_module_color "display" "$theme" "$tcolor3")",
            "keyWidth": 10
        },
        "break",
        {
            "type": "localip",
            $fmtline_localip
            "key": " ├ 󰣺 ",
            "keyColor": "$(get_module_color "localip" "$theme" "$color4")",
            "outputColor": "$(get_module_color "localip" "$theme" "$tcolor4")",
            "keyWidth": 10
        },
        {
            "type": "publicip",
            $fmtline_publicip
            "key": " ├ 󰞉 ",
            "keyColor": "$(get_module_color "publicip" "$theme" "$color4")",
            "outputColor": "$(get_module_color "publicip" "$theme" "$tcolor4")",
            "keyWidth": 10
        },
        {
            "type": "wifi",
            $fmtline_wifi
            "key": " ├ 󰖩 ",
            "keyColor": "$(get_module_color "wifi" "$theme" "$color4")",
            "outputColor": "$(get_module_color "wifi" "$theme" "$tcolor4")",
            "keyWidth": 10
        },
        "break",
        {
            "type": "datetime",
            $fmtline_datetime
            "key": " ├ 󰅐 ",
            "keyColor": "$(get_module_color "datetime" "$theme" "$color5")",
            "outputColor": "$(get_module_color "datetime" "$theme" "$tcolor5")",
            "keyWidth": 10
        },
        {
            "type": "uptime",
            $fmtline_uptime
            "key": " ├ 󰔚 ",
            "keyColor": "$(get_module_color "uptime" "$theme" "$color5")",
            "outputColor": "$(get_module_color "uptime" "$theme" "$tcolor5")",
            "keyWidth": 10
        },
        {
            "type": "media",
            $fmtline_media
            "key": " ├ 󰎈 ",
            "keyColor": "$(get_module_color "media" "$theme" "$color5")",
            "outputColor": "$(get_module_color "media" "$theme" "$tcolor5")",
            "keyWidth": 10
        },
        "break",
        "break"
    ]
}
EOF

    # Clean up empty lines from JSON (when demo mode is off, fmtline vars are empty)
    sed -i '/^[[:space:]]*$/d' "$CONFIG_DIR/fastfetch.jsonc"
}

create_minimal_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3
    local dot_format
    dot_format=$(theme_dot_format "$theme")

    local logo_source=$(get_logo_source "$logo_type" "$custom_image")
    local logo_actual_type=$(get_logo_type "$logo_type" "$custom_image")

    # Get theme-based or custom ANSI colors
    local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme_colors" ] && use_theme_colors="true"

    local color1 color2 color3 color4 color5 color6 color7 color8
    local output_color
    local show_labels=$(get_config_value "fastfetch" "labels")
    [ -z "$show_labels" ] && show_labels="false"
    
    
    if [ "$use_theme_colors" = "false" ]; then
        local custom_key=$(get_custom_fastfetch_color key_color "$theme")
        local custom_value=$(get_custom_fastfetch_color value_color "$theme")
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

    # Demo mode: prepare format overrides
    local demo_mode=$(get_config_value "fastfetch" "demo_mode")
    local fmt_title="" fmtline_os="" fmtline_kernel="" fmtline_shell="" fmtline_uptime=""
    
    if [ "$demo_mode" = "true" ]; then
        fmt_title="hacker@linuxbox"
        fmtline_os="\"format\": \"$(get_demo_format os)\","
        fmtline_kernel="\"format\": \"$(get_demo_format kernel)\","
        fmtline_shell="\"format\": \"$(get_demo_format shell)\","
        fmtline_uptime="\"format\": \"$(get_demo_format uptime)\","
    else
        fmt_title="{user-name-colored}@{host-name-colored}"
    fi

    cat > "$CONFIG_DIR/fastfetch.jsonc" << EOF
{
    // TermFetch Studio - Minimal Preset
    // Theme: $theme
    "\$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",

    "logo": {
        "type": "$logo_actual_type",
        "source": "$logo_source",
$(logo_dimensions_json_fields "$logo_actual_type" "$logo_source" 10)
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
            "format": "$fmt_title",
            "key": ""
        },
        "break",
        {
            "type": "custom",
            "format": "${dot_format}"
        },
        "break",
        {
            "type": "os",
            $fmtline_os
            "key": " ├ 󰣇 ",
            "keyColor": "$(get_module_color "os" "$theme" "$color1")",
            "outputColor": "$(get_module_color "os" "$theme" "$output_color")"
        },
        {
            "type": "kernel",
            $fmtline_kernel
            "key": " ├ 󰌢 ",
            "keyColor": "$(get_module_color "kernel" "$theme" "$color1")",
            "outputColor": "$(get_module_color "kernel" "$theme" "$output_color")"
        },
        {
            "type": "shell",
            $fmtline_shell
            "key": " ├ 󰆍 ",
            "keyColor": "$(get_module_color "shell" "$theme" "$color1")",
            "outputColor": "$(get_module_color "shell" "$theme" "$output_color")"
        },
        {
            "type": "uptime",
            $fmtline_uptime
            "key": " ├ 󰔚 ",
            "keyColor": "$(get_module_color "uptime" "$theme" "$color1")",
            "outputColor": "$(get_module_color "uptime" "$theme" "$output_color")"
        },
        "break",
        {
            "type": "custom",
            "format": "${dot_format}"
        },
        "break"
    ]
}
EOF

    # Clean up empty lines from JSON
    sed -i '/^[[:space:]]*$/d' "$CONFIG_DIR/fastfetch.jsonc"
}

create_focused_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3
    local dot_format
    dot_format=$(theme_dot_format "$theme")

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
        local custom_key=$(get_custom_fastfetch_color key_color "$theme")
        local custom_value=$(get_custom_fastfetch_color value_color "$theme")
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

    # Demo mode: prepare format overrides
    local demo_mode=$(get_config_value "fastfetch" "demo_mode")
    local fmt_title="" fmtline_os="" fmtline_kernel="" fmtline_packages="" fmtline_uptime=""
    local fmtline_terminal="" fmtline_shell="" fmtline_cpu="" fmtline_memory="" fmtline_disk=""
    
    if [ "$demo_mode" = "true" ]; then
        fmt_title="hacker@linuxbox"
        fmtline_os="\"format\": \"$(get_demo_format os)\","
        fmtline_kernel="\"format\": \"$(get_demo_format kernel)\","
        fmtline_packages="\"format\": \"$(get_demo_format packages)\","
        fmtline_uptime="\"format\": \"$(get_demo_format uptime)\","
        fmtline_terminal="\"format\": \"$(get_demo_format terminal)\","
        fmtline_shell="\"format\": \"$(get_demo_format shell)\","
        fmtline_cpu="\"format\": \"$(get_demo_format cpu)\","
        fmtline_memory="\"format\": \"$(get_demo_format memory)\","
        fmtline_disk="\"format\": \"$(get_demo_format disk)\","
    else
        fmt_title="{user-name-colored}@{host-name-colored}"
    fi

    cat > "$CONFIG_DIR/fastfetch.jsonc" << EOF
{
    // TermFetch Studio - Focused Preset
    // Theme: $theme
    "\$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",

    "logo": {
        "type": "$logo_actual_type",
        "source": "$logo_source",
$(logo_dimensions_json_fields "$logo_actual_type" "$logo_source" 12)
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
            "format": "$fmt_title",
            "key": ""
        },
        "break",
        {
            "type": "custom",
            "format": "${dot_format}"
        },
        "break",
        {
            "type": "os",
            $fmtline_os
            "key": " ├ 󰣇 ",
            "keyColor": "$(get_module_color "os" "$theme" "$color1")",
            "outputColor": "$(get_module_color "os" "$theme" "$output_color")"
        },
        {
            "type": "kernel",
            $fmtline_kernel
            "key": " ├ 󰌢 ",
            "keyColor": "$(get_module_color "kernel" "$theme" "$color1")",
            "outputColor": "$(get_module_color "kernel" "$theme" "$output_color")"
        },
        {
            "type": "packages",
            $fmtline_packages
            "key": " ├ 󰏖 ",
            "keyColor": "$(get_module_color "packages" "$theme" "$color1")",
            "outputColor": "$(get_module_color "packages" "$theme" "$output_color")"
        },
        {
            "type": "uptime",
            $fmtline_uptime
            "key": " ├ 󰔟 ",
            "keyColor": "$(get_module_color "uptime" "$theme" "$color1")",
            "outputColor": "$(get_module_color "uptime" "$theme" "$output_color")"
        },
        "break",
        {
            "type": "terminal",
            $fmtline_terminal
            "key": " ├ 󰆍 ",
            "keyColor": "$(get_module_color "terminal" "$theme" "$color2")",
            "outputColor": "$(get_module_color "terminal" "$theme" "$output_color")"
        },
        {
            "type": "shell",
            $fmtline_shell
            "key": " ├ 󰆍 ",
            "keyColor": "$(get_module_color "shell" "$theme" "$color2")",
            "outputColor": "$(get_module_color "shell" "$theme" "$output_color")"
        },
        "break",
        {
            "type": "cpu",
            $fmtline_cpu
            "key": " ├ 󰻠 ",
            "keyColor": "$(get_module_color "cpu" "$theme" "$color3")",
            "outputColor": "$(get_module_color "cpu" "$theme" "$output_color")"
        },
        {
            "type": "memory",
            $fmtline_memory
            "key": " ├ 󰍛 ",
            "keyColor": "$(get_module_color "memory" "$theme" "$color3")",
            "outputColor": "$(get_module_color "memory" "$theme" "$output_color")"
        },
        {
            "type": "disk",
            $fmtline_disk
            "key": " ├ 󰋊 ",
            "keyColor": "$(get_module_color "disk" "$theme" "$color3")",
            "outputColor": "$(get_module_color "disk" "$theme" "$output_color")"
        },
        "break",
        {
            "type": "custom",
            "format": "${dot_format}"
        },
        "break"
    ]
}
EOF

    # Clean up empty lines from JSON
    sed -i '/^[[:space:]]*$/d' "$CONFIG_DIR/fastfetch.jsonc"
}

create_developer_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3
    local dot_format
    dot_format=$(theme_dot_format "$theme")

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
        local custom_key=$(get_custom_fastfetch_color key_color "$theme")
        local custom_value=$(get_custom_fastfetch_color value_color "$theme")
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

    # Demo mode: prepare format overrides
    local demo_mode=$(get_config_value "fastfetch" "demo_mode")
    local fmt_title="" fmtline_os="" fmtline_kernel="" fmtline_packages="" fmtline_shell=""
    local fmtline_terminal="" fmtline_terminalfont="" fmtline_de="" fmtline_wm=""
    local fmtline_cpu="" fmtline_gpu="" fmtline_memory="" fmtline_swap="" fmtline_disk=""
    local fmtline_localip="" fmtline_publicip=""
    
    if [ "$demo_mode" = "true" ]; then
        fmt_title="hacker@linuxbox"
        fmtline_os="\"format\": \"$(get_demo_format os)\","
        fmtline_kernel="\"format\": \"$(get_demo_format kernel)\","
        fmtline_packages="\"format\": \"$(get_demo_format packages)\","
        fmtline_shell="\"format\": \"$(get_demo_format shell)\","
        fmtline_terminal="\"format\": \"$(get_demo_format terminal)\","
        fmtline_terminalfont="\"format\": \"$(get_demo_format terminalfont)\","
        fmtline_de="\"format\": \"$(get_demo_format de)\","
        fmtline_wm="\"format\": \"$(get_demo_format wm)\","
        fmtline_cpu="\"format\": \"$(get_demo_format cpu)\","
        fmtline_gpu="\"format\": \"$(get_demo_format gpu)\","
        fmtline_memory="\"format\": \"$(get_demo_format memory)\","
        fmtline_swap="\"format\": \"$(get_demo_format swap)\","
        fmtline_disk="\"format\": \"$(get_demo_format disk)\","
        fmtline_localip="\"format\": \"$(get_demo_format localip)\","
        fmtline_publicip="\"format\": \"$(get_demo_format publicip)\","
    else
        fmt_title="{user-name-colored}@{host-name-colored}"
    fi

    cat > "$CONFIG_DIR/fastfetch.jsonc" << EOF
{
    // TermFetch Studio - Developer Preset
    // Theme: $theme
    "\$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",

    "logo": {
        "type": "$logo_actual_type",
        "source": "$logo_source",
$(logo_dimensions_json_fields "$logo_actual_type" "$logo_source" 14)
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
            "format": "$fmt_title",
            "key": ""
        },
        "break",
        {
            "type": "custom",
            "format": "${dot_format}"
        },
        "break",
        {
            "type": "os",
            $fmtline_os
            "key": " ├ 󰣇 ",
            "keyColor": "$(get_module_color "os" "$theme" "$color1")",
            "outputColor": "$(get_module_color "os" "$theme" "$output_color")"
        },
        {
            "type": "kernel",
            $fmtline_kernel
            "key": " ├ 󰌢 ",
            "keyColor": "$(get_module_color "kernel" "$theme" "$color1")",
            "outputColor": "$(get_module_color "kernel" "$theme" "$output_color")"
        },
        {
            "type": "packages",
            $fmtline_packages
            "key": " ├ 󰏖 ",
            "keyColor": "$(get_module_color "packages" "$theme" "$color1")",
            "outputColor": "$(get_module_color "packages" "$theme" "$output_color")"
        },
        {
            "type": "shell",
            $fmtline_shell
            "key": " ├ 󰆍 ",
            "keyColor": "$(get_module_color "shell" "$theme" "$color1")",
            "outputColor": "$(get_module_color "shell" "$theme" "$output_color")"
        },
        "break",
        {
            "type": "terminal",
            $fmtline_terminal
            "key": " ├ 󰆍 ",
            "keyColor": "$(get_module_color "terminal" "$theme" "$color2")",
            "outputColor": "$(get_module_color "terminal" "$theme" "$output_color")"
        },
        {
            "type": "terminalfont",
            $fmtline_terminalfont
            "key": " ├ 󰛖 ",
            "keyColor": "$(get_module_color "terminalfont" "$theme" "$color2")",
            "outputColor": "$(get_module_color "terminalfont" "$theme" "$output_color")"
        },
        {
            "type": "de",
            $fmtline_de
            "key": " ├ 󱂬 ",
            "keyColor": "$(get_module_color "de" "$theme" "$color2")",
            "outputColor": "$(get_module_color "de" "$theme" "$output_color")"
        },
        {
            "type": "wm",
            $fmtline_wm
            "key": " ├ 󰖲 ",
            "keyColor": "$(get_module_color "wm" "$theme" "$color2")",
            "outputColor": "$(get_module_color "wm" "$theme" "$output_color")"
        },
        "break",
        {
            "type": "cpu",
            $fmtline_cpu
            "key": " ├ 󰻠 ",
            "keyColor": "$(get_module_color "cpu" "$theme" "$color3")",
            "outputColor": "$(get_module_color "cpu" "$theme" "$output_color")"
        },
        {
            "type": "gpu",
            $fmtline_gpu
            "key": " ├ 󰢮 ",
            "keyColor": "$(get_module_color "gpu" "$theme" "$color3")",
            "outputColor": "$(get_module_color "gpu" "$theme" "$output_color")"
        },
        {
            "type": "memory",
            $fmtline_memory
            "key": " ├ 󰍛 ",
            "keyColor": "$(get_module_color "memory" "$theme" "$color3")",
            "outputColor": "$(get_module_color "memory" "$theme" "$output_color")"
        },
        {
            "type": "swap",
            $fmtline_swap
            "key": " ├ 󰓡 ",
            "keyColor": "$(get_module_color "swap" "$theme" "$color3")",
            "outputColor": "$(get_module_color "swap" "$theme" "$output_color")"
        },
        {
            "type": "disk",
            $fmtline_disk
            "key": " ├ 󰋊 ",
            "keyColor": "$(get_module_color "disk" "$theme" "$color3")",
            "outputColor": "$(get_module_color "disk" "$theme" "$output_color")"
        },
        "break",
        {
            "type": "localip",
            $fmtline_localip
            "key": " ├ 󰩟 ",
            "keyColor": "$(get_module_color "localip" "$theme" "$color4")",
            "outputColor": "$(get_module_color "localip" "$theme" "$output_color")"
        },
        {
            "type": "publicip",
            $fmtline_publicip
            "key": " ├ 󰞉 ",
            "keyColor": "$(get_module_color "publicip" "$theme" "$color4")",
            "outputColor": "$(get_module_color "publicip" "$theme" "$output_color")"
        },
        "break",
        {
            "type": "custom",
            "format": "${dot_format}"
        },
        "break"
    ]
}
EOF

    # Clean up empty lines from JSON
    sed -i '/^[[:space:]]*$/d' "$CONFIG_DIR/fastfetch.jsonc"
}

create_gaming_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3
    local dot_format
    dot_format=$(theme_dot_format "$theme")

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
        local custom_key=$(get_custom_fastfetch_color key_color "$theme")
        local custom_value=$(get_custom_fastfetch_color value_color "$theme")
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

    # Demo mode: prepare format overrides
    local demo_mode=$(get_config_value "fastfetch" "demo_mode")
    local fmt_title="" fmtline_os="" fmtline_kernel="" fmtline_uptime=""
    local fmtline_cpu="" fmtline_gpu="" fmtline_memory="" fmtline_swap="" fmtline_disk=""
    local fmtline_display="" fmtline_battery="" fmtline_localip="" fmtline_wifi=""
    
    if [ "$demo_mode" = "true" ]; then
        fmt_title="hacker@linuxbox"
        fmtline_os="\"format\": \"$(get_demo_format os)\","
        fmtline_kernel="\"format\": \"$(get_demo_format kernel)\","
        fmtline_uptime="\"format\": \"$(get_demo_format uptime)\","
        fmtline_cpu="\"format\": \"$(get_demo_format cpu)\","
        fmtline_gpu="\"format\": \"$(get_demo_format gpu)\","
        fmtline_memory="\"format\": \"$(get_demo_format memory)\","
        fmtline_swap="\"format\": \"$(get_demo_format swap)\","
        fmtline_disk="\"format\": \"$(get_demo_format disk)\","
        fmtline_display="\"format\": \"$(get_demo_format display)\","
        fmtline_battery="\"format\": \"$(get_demo_format battery)\","
        fmtline_localip="\"format\": \"$(get_demo_format localip)\","
        fmtline_wifi="\"format\": \"$(get_demo_format wifi)\","
    else
        fmt_title="{user-name-colored}@{host-name-colored}"
    fi

    cat > "$CONFIG_DIR/fastfetch.jsonc" << EOF
{
    // TermFetch Studio - Gaming Preset
    // Theme: $theme
    "\$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",

    "logo": {
        "type": "$logo_actual_type",
        "source": "$logo_source",
$(logo_dimensions_json_fields "$logo_actual_type" "$logo_source" 14)
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
            "format": "$fmt_title",
            "key": ""
        },
        "break",
        {
            "type": "custom",
            "format": "${dot_format}"
        },
        "break",
        {
            "type": "os",
            $fmtline_os
            "key": " ├ 󰣇 ",
            "keyColor": "$(get_module_color "os" "$theme" "$color1")",
            "outputColor": "$(get_module_color "os" "$theme" "$output_color")"
        },
        {
            "type": "kernel",
            $fmtline_kernel
            "key": " ├ 󰌢 ",
            "keyColor": "$(get_module_color "kernel" "$theme" "$color1")",
            "outputColor": "$(get_module_color "kernel" "$theme" "$output_color")"
        },
        {
            "type": "uptime",
            $fmtline_uptime
            "key": " ├ 󰔟 ",
            "keyColor": "$(get_module_color "uptime" "$theme" "$color1")",
            "outputColor": "$(get_module_color "uptime" "$theme" "$output_color")"
        },
        "break",
        {
            "type": "cpu",
            $fmtline_cpu
            "key": " ├ 󰻠 ",
            "keyColor": "$(get_module_color "cpu" "$theme" "$color2")",
            "outputColor": "$(get_module_color "cpu" "$theme" "$output_color")"
        },
        {
            "type": "gpu",
            $fmtline_gpu
            "key": " ├ 󰢮 ",
            "keyColor": "$(get_module_color "gpu" "$theme" "$color2")",
            "outputColor": "$(get_module_color "gpu" "$theme" "$output_color")"
        },
        {
            "type": "memory",
            $fmtline_memory
            "key": " ├ 󰍛 ",
            "keyColor": "$(get_module_color "memory" "$theme" "$color2")",
            "outputColor": "$(get_module_color "memory" "$theme" "$output_color")"
        },
        {
            "type": "swap",
            $fmtline_swap
            "key": " ├ 󰓡 ",
            "keyColor": "$(get_module_color "swap" "$theme" "$color2")",
            "outputColor": "$(get_module_color "swap" "$theme" "$output_color")"
        },
        {
            "type": "disk",
            $fmtline_disk
            "key": " ├ 󰋊 ",
            "keyColor": "$(get_module_color "disk" "$theme" "$color2")",
            "outputColor": "$(get_module_color "disk" "$theme" "$output_color")"
        },
        {
            "type": "display",
            $fmtline_display
            "key": " ├ 󰍹 ",
            "keyColor": "$(get_module_color "display" "$theme" "$color2")",
            "outputColor": "$(get_module_color "display" "$theme" "$output_color")"
        },
        {
            "type": "battery",
            $fmtline_battery
            "key": " ├ 󰂎 ",
            "keyColor": "$(get_module_color "battery" "$theme" "$color2")",
            "outputColor": "$(get_module_color "battery" "$theme" "$output_color")"
        },
        "break",
        {
            "type": "localip",
            $fmtline_localip
            "key": " ├ 󰩟 ",
            "keyColor": "$(get_module_color "localip" "$theme" "$color3")",
            "outputColor": "$(get_module_color "localip" "$theme" "$output_color")"
        },
        {
            "type": "wifi",
            $fmtline_wifi
            "key": " ├ 󰖩 ",
            "keyColor": "$(get_module_color "wifi" "$theme" "$color3")",
            "outputColor": "$(get_module_color "wifi" "$theme" "$output_color")"
        },
        "break",
        {
            "type": "custom",
            "format": "${dot_format}"
        },
        "break"
    ]
}
EOF

    # Clean up empty lines from JSON
    sed -i '/^[[:space:]]*$/d' "$CONFIG_DIR/fastfetch.jsonc"
}

# Create custom config using a saved list of modules in config.ini
create_custom_config() {
    local logo_type=$1
    local custom_image=$2
    local theme=$3
    local dot_format
    dot_format=$(theme_dot_format "$theme")

    local logo_source=$(get_logo_source "$logo_type" "$custom_image")
    local logo_actual_type=$(get_logo_type "$logo_type" "$custom_image")

    local color1=$(get_theme_ansi_color "$theme" "primary")

    local color2 color3 color4 color5 color6 color7 color8
    local output_color
    local use_theme_colors=$(get_config_value "fastfetch" "use_theme_colors")
    [ -z "$use_theme_colors" ] && use_theme_colors="true"

    if [ "$use_theme_colors" = "false" ]; then
        local custom_key=$(get_custom_fastfetch_color key_color "$theme")
        local custom_value=$(get_custom_fastfetch_color value_color "$theme")
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
$(logo_dimensions_json_fields "$logo_actual_type" "$logo_source" 14)
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
            "format": "${dot_format}"
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
            "keyColor": "$(get_module_color "$m" "$theme" "$color1")",
            "outputColor": "$(get_module_color "$m" "$theme" "$output_color")"
        }
EOF
    done

    # Add closing color dots
    cat >> "$CONFIG_DIR/fastfetch.jsonc" << EOF
        ,"break"
        ,{
            "type": "custom",
            "format": "${dot_format}"
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

    # Existing configuration may contain private files and user-selected modes.
    # It was created by this non-root installer, so leave its ownership intact.

    print_success "Permissions configured"
}

has_external_fetch_startup() {
    [ -f "$1" ] && grep -Eq '^[[:space:]]*((command|exec)[[:space:]]+)?("?[^[:space:]]*/)?(fastfetch|tfs-fastfetch)("?([[:space:]]|$))|^[[:space:]]*termfetch-studio[[:space:]]+--preview' "$1"
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
        local external_startup=false
        has_external_fetch_startup "$HOME/.bashrc" && external_startup=true
        printf '\n# TermFetch Studio - START\n' >> "$HOME/.bashrc"
        printf '_tfs_external_startup=%s\n' "$external_startup" >> "$HOME/.bashrc"
        cat >> "$HOME/.bashrc" << 'BASHRC'

export PATH="$HOME/.local/bin:$PATH"
if [[ $- == *i* ]] && [ -t 1 ] && [ "${_tfs_external_startup:-false}" = true ]; then export TFS_STARTUP_TTY="$(tty 2>/dev/null)"; fi
if [[ $- == *i* ]] && [ -t 1 ] && [ "${_tfs_external_startup:-false}" != true ]; then
_tfs_tty=$(tty 2>/dev/null)
if [ "${TFS_STARTUP_TTY:-}" != "$_tfs_tty" ]; then
export TFS_STARTUP_TTY="$_tfs_tty"
[ -f ~/.config/termfetch-studio/theme.sh ] && source ~/.config/termfetch-studio/theme.sh
if [ -x "$HOME/.local/bin/tfs-fastfetch" ]; then
    "$HOME/.local/bin/tfs-fastfetch"
elif command -v fastfetch &> /dev/null && [ -f ~/.config/termfetch-studio/fastfetch.jsonc ]; then
    fastfetch --config ~/.config/termfetch-studio/fastfetch.jsonc
fi
fi
fi
unset _tfs_tty _tfs_external_startup
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
        local external_startup=false
        has_external_fetch_startup "$HOME/.zshrc" && external_startup=true
        printf '\n# TermFetch Studio - START\n' >> "$HOME/.zshrc"
        printf '_tfs_external_startup=%s\n' "$external_startup" >> "$HOME/.zshrc"
        cat >> "$HOME/.zshrc" << 'ZSHRC'

export PATH="$HOME/.local/bin:$PATH"
if [[ -o interactive ]] && [ -t 1 ] && [ "${_tfs_external_startup:-false}" = true ]; then export TFS_STARTUP_TTY="$(tty 2>/dev/null)"; fi
if [[ -o interactive ]] && [[ -t 1 ]] && [[ "${_tfs_external_startup:-false}" != true ]]; then
_tfs_tty=$(tty 2>/dev/null)
if [[ "${TFS_STARTUP_TTY:-}" != "$_tfs_tty" ]]; then
export TFS_STARTUP_TTY="$_tfs_tty"
[ -f ~/.config/termfetch-studio/theme.sh ] && source ~/.config/termfetch-studio/theme.sh
if [ -x "$HOME/.local/bin/tfs-fastfetch" ]; then
    "$HOME/.local/bin/tfs-fastfetch"
elif command -v fastfetch &> /dev/null && [ -f ~/.config/termfetch-studio/fastfetch.jsonc ]; then
    fastfetch --config ~/.config/termfetch-studio/fastfetch.jsonc
fi
fi
fi
unset _tfs_tty _tfs_external_startup
# TermFetch Studio - END
ZSHRC
        print_success "Added to ~/.zshrc"
    fi

    if [ -f "$HOME/.config/fish/config.fish" ]; then
        # Remove any existing TermFetch Studio block
        sed -i '/# TermFetch Studio - START/,/# TermFetch Studio - END/d' "$HOME/.config/fish/config.fish" 2>/dev/null || true

        # Add new integration block with markers
        local external_startup=false
        has_external_fetch_startup "$HOME/.config/fish/config.fish" && external_startup=true
        printf '\n# TermFetch Studio - START\n' >> "$HOME/.config/fish/config.fish"
        printf 'set -l _tfs_external_startup %s\n' "$external_startup" >> "$HOME/.config/fish/config.fish"
        cat >> "$HOME/.config/fish/config.fish" << 'FISHRC'

fish_add_path ~/.local/bin
if status is-interactive; and test -t 1; and test "$_tfs_external_startup" = true
set -gx TFS_STARTUP_TTY (tty 2>/dev/null)
end
if status is-interactive; and test -t 1; and test "$_tfs_external_startup" != true
set -l _tfs_tty (tty 2>/dev/null)
if test "$TFS_STARTUP_TTY" != "$_tfs_tty"
set -gx TFS_STARTUP_TTY "$_tfs_tty"
test -f ~/.config/termfetch-studio/theme.sh && bash ~/.config/termfetch-studio/theme.sh
if test -x ~/.local/bin/tfs-fastfetch
    ~/.local/bin/tfs-fastfetch
else if command -v fastfetch &> /dev/null && test -f ~/.config/termfetch-studio/fastfetch.jsonc
    fastfetch --config ~/.config/termfetch-studio/fastfetch.jsonc
end
end
end
set -e _tfs_external_startup
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
║             TermFetch Studio v1.0.0               ║
║                 Uninstall Script                  ║
║                                                   ║
╚═══════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

confirm_uninstall() {
    echo -e "${YELLOW}⚠️  This will completely remove TermFetch Studio from your system.${NC}"
    echo -e "${YELLOW}The following will be removed:${NC}"
    echo "  - $HOME/.local/bin/termfetch-studio and tfs-fastfetch"
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

remove_legacy_fish_alias() {
    local file=$1 edited backup result
    [ -f "$file" ] || return 0
    edited=$(mktemp "$file.tfs.XXXXXX") || return 1
    cp -p -- "$file" "$edited" || { rm -f -- "$edited"; return 1; }
    if awk '
        {
            line=$0
            if (sub(/^[[:space:]]*alias[[:space:]]+fastfetch[[:space:]]*(=[[:space:]]*)?/, "", line)) {
                sub(/[[:space:]]+$/, "", line)
                quote=substr(line,1,1)
                if ((quote == "\"" || quote == sprintf("%c",39)) && substr(line,length(line),1) == quote)
                    line=substr(line,2,length(line)-2)
                if (line == "termfetch-studio --preview") { removed=1; next }
            }
            print
        }
        END { if (!removed) exit 3 }
    ' "$file" > "$edited"; then result=0; else result=$?; fi
    if ((result != 0)); then
        rm -f -- "$edited"
        ((result == 3)) && return 0
        return 1
    fi
    backup=$(mktemp "$file.termfetch-backup.XXXXXX") || { rm -f -- "$edited"; return 1; }
    cp -p -- "$file" "$backup" || { rm -f -- "$edited" "$backup"; return 1; }
    mv -- "$edited" "$file" || { rm -f -- "$edited"; return 1; }
    print_info "Removed exact legacy Fish alias (original saved beside config.fish)"
}

remove_shell_integration() {
    print_info "Removing shell integration..."

    # Resolve target user and home even when run via sudo
    TARGET_USER="${SUDO_USER:-$USER}"
    TARGET_HOME=$(getent passwd "$TARGET_USER" | cut -d: -f6)
    [ -z "$TARGET_HOME" ] && TARGET_HOME="$HOME"

    # Remove only the block installed by TermFetch Studio.
    if [ -f "$TARGET_HOME/.bashrc" ]; then
        if grep -q "TermFetch Studio" "$TARGET_HOME/.bashrc" 2>/dev/null; then
            sed -i '/# TermFetch Studio - START/,/# TermFetch Studio - END/d' "$TARGET_HOME/.bashrc"
            print_info "Removed from .bashrc"
        fi
    fi

    # Remove from zsh
    if [ -f "$TARGET_HOME/.zshrc" ]; then
        if grep -q "TermFetch Studio" "$TARGET_HOME/.zshrc" 2>/dev/null; then
            sed -i '/# TermFetch Studio - START/,/# TermFetch Studio - END/d' "$TARGET_HOME/.zshrc"
            print_info "Removed from .zshrc"
        fi
    fi

    # Remove from fish
    if [ -f "$TARGET_HOME/.config/fish/config.fish" ]; then
        if grep -q "TermFetch Studio" "$TARGET_HOME/.config/fish/config.fish" 2>/dev/null; then
            sed -i '/# TermFetch Studio - START/,/# TermFetch Studio - END/d' "$TARGET_HOME/.config/fish/config.fish"
            print_info "Removed from config.fish"
        fi
    fi
    remove_legacy_fish_alias "$TARGET_HOME/.config/fish/config.fish"
    echo "An already-open Fish keeps loaded functions. In that Fish session run:"
    echo "  functions --erase fastfetch"
    echo "Custom Fish functions/plugins are preserved; review them if they still call TermFetch Studio."

}

remove_installation() {
    local auto_yes="${1:-false}"
    
    print_info "Removing TermFetch Studio..."

    # The installer owns the per-user executable, never /usr/local/bin.
    rm -f -- "$HOME/.local/bin/termfetch-studio" "$HOME/.local/bin/tfs-fastfetch"

    # Remove installation directory
    if [ -d "$HOME/.local/share/termfetch-studio" ]; then
        rm -rf "$HOME/.local/share/termfetch-studio"
        print_info "Removed installation directory"
    fi

    # Remove shell integration
    remove_shell_integration

    # Ask about config directory (or auto-remove if -y flag)
    TARGET_USER="${SUDO_USER:-$USER}"
    TARGET_HOME=$(getent passwd "$TARGET_USER" | cut -d: -f6)
    [ -z "$TARGET_HOME" ] && TARGET_HOME="$HOME"
    
    if [ "$auto_yes" = "true" ]; then
        # Auto-remove config with -y flag
        if [ -d "$TARGET_HOME/.config/termfetch-studio" ]; then
            rm -rf "$TARGET_HOME/.config/termfetch-studio"
            print_info "Removed configuration directory"
        fi
    else
        echo ""
        echo -e "${YELLOW}Do you want to remove the configuration directory?${NC}"
        echo -e "${YELLOW}This will delete all your settings and backups.${NC}"
        read -p "Remove config directory? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if [ -d "$TARGET_HOME/.config/termfetch-studio" ]; then
                rm -rf "$TARGET_HOME/.config/termfetch-studio"
                print_info "Removed configuration directory"
            fi
        else
            print_info "Configuration directory preserved at $TARGET_HOME/.config/termfetch-studio"
        fi
    fi
}

main() {
    local auto_yes=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -y|--yes)
                auto_yes=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done
    
    print_banner
    
    if [ "$auto_yes" = "true" ]; then
        # Skip confirmation, proceed directly
        echo -e "${YELLOW}⚠️  Auto-confirming uninstall (-y flag)${NC}"
    else
        confirm_uninstall
    fi
    
    remove_installation "$auto_yes"
    
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

    # Generate the initial configuration during installation, not on first run.
    source "$INSTALL_DIR/lib/core.sh"
    source "$INSTALL_DIR/lib/fastfetch.sh"
    init_config

    # Migrate the unsupported legacy keyShow field without replacing user edits.
    # Keep an exact copy before changing an existing JSONC configuration.
    if grep -q '"keyShow"[[:space:]]*:' "$CONFIG_DIR/fastfetch.jsonc"; then
        local original migrated
        mkdir -p "$CONFIG_DIR/backups"
        original=$(mktemp "$CONFIG_DIR/backups/legacy-fastfetch.XXXXXX.jsonc") || exit 1
        cp -p -- "$CONFIG_DIR/fastfetch.jsonc" "$original" || exit 1
        migrated=$(mktemp "$CONFIG_DIR/.fastfetch.XXXXXX") || exit 1
        cp -p -- "$CONFIG_DIR/fastfetch.jsonc" "$migrated" || exit 1
        if awk '
            /"keyShow"[[:space:]]*:/ {
                sub(/,[[:space:]]*$/, "", previous)
                next
            }
            { if (NR > 1) print previous; previous=$0 }
            END { print previous }
        ' "$CONFIG_DIR/fastfetch.jsonc" > "$migrated"; then
            mv -- "$migrated" "$CONFIG_DIR/fastfetch.jsonc" || exit 1
        else
            rm -f -- "$migrated"
            print_error "Legacy configuration migration failed; original retained"
            exit 1
        fi
    fi

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

