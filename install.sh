#!/usr/bin/env bash
# shellcheck disable=SC2155
# =============================================================================
# Hyprbar Install Script
# Sets up config directory and installs binaries
# =============================================================================

set -euo pipefail
IFS=$'\n\t'
shopt -s inherit_errexit 2>/dev/null || true

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || echo "")"
readonly CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/hypr"
readonly DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/hyprbar"
readonly INSTALL_DIR="${HOME}/.local/bin"

# Project Specifics
readonly BIN_NAME="hyprbar"
readonly TARGET_NAME="hyprbar"

# Colors (Sweet Dracula palette - 24-bit true color)
readonly GREEN=$'\033[38;2;80;250;123m'
readonly YELLOW=$'\033[38;2;241;250;140m'
readonly CYAN=$'\033[38;2;139;233;253m'
readonly RED=$'\033[38;2;255;85;85m'
readonly PURPLE=$'\033[38;2;189;147;249m'
readonly NC=$'\033[0m'

# -----------------------------------------------------------------------------
# Logging Functions
# -----------------------------------------------------------------------------
log()     { echo -e "${CYAN}[info]${NC} INSTALL  $*"; }
success() { echo -e "${GREEN}[ok]${NC}   INSTALL  $*"; }
warn()    { echo -e "${YELLOW}[warn]${NC} INSTALL  $*" >&2; }
error()   { echo -e "${RED}[err]${NC}  INSTALL  $*" >&2; }
die()     { error "$*"; exit 1; }

# -----------------------------------------------------------------------------
# Utility Functions
# -----------------------------------------------------------------------------
command_exists() { command -v "$1" &>/dev/null; }

create_dir() {
    local dir="$1"
    if [[ ! -d "$dir" ]]; then
        mkdir -p "$dir" || die "Failed to create directory: $dir"
        success "Created $dir"
    fi
}

compact_binary() {
    local bin="$1"
    if [[ -f "$bin" ]] && command_exists upx; then
        local size_before=$(stat -c%s "$bin")
        upx --best --lzma --quiet "$bin" > /dev/null
        local size_after=$(stat -c%s "$bin")
        local saved=$(( size_before - size_after ))
        local percent=$(( (saved * 100) / size_before ))

        # Convert bytes to readable format
        local size_before_fmt=$(numfmt --to=iec-i --suffix=B "$size_before")
        local size_after_fmt=$(numfmt --to=iec-i --suffix=B "$size_after")

        log "Optimized $(basename "$bin"): ${size_before_fmt} -> ${size_after_fmt} (-${percent}%)"
    fi
}

write_config() {
    local file="$1"
    local content="$2"

    if [[ -f "$file" ]]; then
        log "Config exists, skipping: $(basename "$file")"
        return 0
    fi

    log "Creating $(basename "$file")"
    printf '%s\n' "$content" > "$file" || die "Failed to write: $file"
    success "Created $(basename "$file")"
}

# -----------------------------------------------------------------------------
# Config Templates
# -----------------------------------------------------------------------------
HYPRBAR_CONFIG='# Hyprbar Configuration

[window]
height = 30
anchor = "top"
monitor = ""

[style]
bg = "#1e1e2e"
fg = "#cdd6f4"

[layout]
left = 33
center = 34
right = 33
modules_left = ["separator"]
modules_center = ["datetime"]
modules_right = ["text_area"]

[logging]
level = "info"
'

# -----------------------------------------------------------------------------
# Main Installation
# -----------------------------------------------------------------------------
stop_running_bar() {
    if command_exists "${INSTALL_DIR}/${TARGET_NAME}"; then
        log "Stopping running bar..."
        "${INSTALL_DIR}/${TARGET_NAME}" --stop || true
        sleep 1
    elif pgrep -x "$TARGET_NAME" >/dev/null; then
        log "Stopping running bar (pkill)..."
        pkill -x "$TARGET_NAME" || true
        sleep 1
    fi
}

install_from_source() {
    cd "$SCRIPT_DIR" || die "Failed to cd to script directory"

    if ! command_exists cargo; then
        die "Cargo not found. Install Rust: https://rustup.rs"
    fi

    log "Building release binary..."
    if ! cargo build --release 2>&1; then
        die "Build failed"
    fi
    success "Build complete"

    # Optimization
    compact_binary "target/release/$BIN_NAME"

    # Install
    local src="target/release/$BIN_NAME"
    if [[ -f "$src" ]]; then
        stop_running_bar
        cp "$src" "${INSTALL_DIR}/${TARGET_NAME}"
        chmod +x "${INSTALL_DIR}/${TARGET_NAME}"
        success "Installed to ${INSTALL_DIR}/${TARGET_NAME}"
    else
        die "Binary not found: $src"
    fi
}

main() {
    echo -e "${PURPLE}[hyprbar]${NC} INSTALL  starting installation"

    # Directories
    create_dir "$CONFIG_DIR"
    create_dir "$DATA_DIR"
    create_dir "$DATA_DIR/widgets"
    create_dir "$INSTALL_DIR"

    # Configs
    write_config "${CONFIG_DIR}/hyprbar.conf" "$HYPRBAR_CONFIG"

    # Install
    install_from_source

    # Summary
    echo -e "${GREEN}[summary]${NC} installed successfully"
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR not in PATH"
    fi

    # Version check
    if command_exists "${INSTALL_DIR}/${TARGET_NAME}"; then
        log "Installed version: $("${INSTALL_DIR}/${TARGET_NAME}" --version 2>/dev/null || echo "unknown")"
    fi
}

main "$@"
