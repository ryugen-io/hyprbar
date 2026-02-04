default:
    @just --list

# Build the binary
build:
    cargo build --release

# Build and compress with UPX
build-upx:
    cargo build --release
    upx --best --lzma target/release/hyprbar
    ls -lh target/release/hyprbar

# Run the binary with debug logging
run:
    RUST_LOG=debug cargo run

# Check for errors and lints
check:
    cargo fmt -- --check
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Pre-commit check
pre-commit: fmt check

# Install the binary
install: pre-commit
    cargo install --path . --force
    ./install.sh

# Clean build artifacts
clean:
    cargo clean

# -----------------------------------------------------------------------------
# Hyprbar CLI Wrappers (Targeting release binary)
# -----------------------------------------------------------------------------
BIN := "target/release/hyprbar"

# Start the bar daemon
start:
    {{BIN}} --start

# Stop the bar daemon
stop:
    {{BIN}} --stop

# Restart the bar daemon
restart:
    {{BIN}} --restart

# Configure autostart
autostart:
    {{BIN}} --autostart

# Restart bar with debug logging (opens debug terminal)
debug:
    {{BIN}} --restart --debug

# Run the TUI / Main executable
launch:
    {{BIN}}

# List installed widgets
list:
    {{BIN}} list

# Enable a widget
enable name:
    {{BIN}} enable {{name}}

# Disable a widget
disable name:
    {{BIN}} disable {{name}}

# Compile a widget from .rs source
compile path:
    {{BIN}} compile {{path}}

# Install a compiled widget (.so)
install-widget path:
    {{BIN}} install {{path}}

# Show version
version:
    {{BIN}} --version
