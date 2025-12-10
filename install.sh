#!/usr/bin/env bash
# shellcheck disable=SC2155
# =============================================================================
# KitchnSink Install Script
# Sets up config directory and installs binaries
# =============================================================================

set -euo pipefail
IFS=$'\n\t'
shopt -s inherit_errexit 2>/dev/null || true

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || echo "")"
readonly CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/kitchnsink"
readonly INSTALL_DIR="${HOME}/.local/bin"

# Project Specifics
readonly BIN_NAME="ks-bin"
readonly TARGET_NAME="kitchnsink"

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
SINK_CONFIG='# KitchnSink Configuration
# Default configuration for the sink bar

[window]
height = 24
anchor = "bottom"
monitor = "primary"

[layout]
modules_left = ["workspaces"]
modules_center = ["clock"]
modules_right = ["systray", "cpu", "memory"]

[theme]
# Uses kitchn styles by default, override here if needed
opacity = 0.95
'

# -----------------------------------------------------------------------------
# Main Installation
# -----------------------------------------------------------------------------
install_from_source() {
    cd "$SCRIPT_DIR" || die "Failed to cd to script directory"

    if ! command_exists cargo; then
        die "Cargo not found. Install Rust: https://rustup.rs"
    fi

    log "Building release binary..."
    if ! cargo build --release -p "$BIN_NAME" 2>&1; then
        die "Build failed"
    fi
    success "Build complete"

    # Optimization
    compact_binary "target/release/$BIN_NAME"

    # Install
    local src="target/release/$BIN_NAME"
    if [[ -f "$src" ]]; then
        cp "$src" "${INSTALL_DIR}/${TARGET_NAME}"
        chmod +x "${INSTALL_DIR}/${TARGET_NAME}"
        success "Installed to ${INSTALL_DIR}/${TARGET_NAME}"
    else
        die "Binary not found: $src"
    fi
}

main() {
    echo -e "${PURPLE}[kitchnsink]${NC} INSTALL  starting installation"

    # Directories
    create_dir "$CONFIG_DIR"
    create_dir "$INSTALL_DIR"

    # Configs
    write_config "${CONFIG_DIR}/sink.toml" "$SINK_CONFIG"

    # Install
    install_from_source

    # Summary
    echo -e "${GREEN}[summary]${NC} summary  installed successfully"
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR not in PATH"
    fi
    
    # Version check
    if command_exists "${INSTALL_DIR}/${TARGET_NAME}"; then
        # Check versions via kitchn-log if possible for pretty output, else plain
        log "Installed version: $("${INSTALL_DIR}/${TARGET_NAME}" --version 2>/dev/null || echo "unknown")"
    fi
}

main "$@"
