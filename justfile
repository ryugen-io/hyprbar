default:
    @just --list

# Build the binary
build:
    cargo build -p ks-bin --release

# Build and compress with UPX
build-upx:
    cargo build -p ks-bin --release
    upx --best --lzma target/release/ks-bin
    ls -lh target/release/ks-bin

# Run the binary with debug logging
run:
    RUST_LOG=debug cargo run -p ks-bin

# Check for errors and lints
check:
    cargo fmt --all -- --check
    cargo clippy --workspace -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Pre-commit check
pre-commit: fmt check

# Install the binary
install: pre-commit
    cargo install --path crates/ks-bin --force
    ./install.sh

# Show project statistics (LOC, binary sizes)
stats:
    @../utils/kitchnsink/stats .

# Dev helper: wash plugins from .wash to .load
dwash:
    @../utils/kitchnsink/ksdev --wash

# Dev helper: load plugins from .load
dload:
    @../utils/kitchnsink/ksdev --load

# Run debug inspector (Screenshot + Config)
inspect:
    @test -d ../utils/kitchnsink/.venv || (python3 -m venv ../utils/kitchnsink/.venv && ../utils/kitchnsink/.venv/bin/pip install -r ../utils/kitchnsink/requirements.txt)
    @../utils/kitchnsink/.venv/bin/python3 ../utils/kitchnsink/debug_view.py

# Clean build artifacts (keeps .wash sources safe)
clean:
    cargo clean
    cargo cache -a

# -----------------------------------------------------------------------------
# KitchnSink CLI Wrappers (Targeting release binary)
# -----------------------------------------------------------------------------
BIN := "target/release/ks-bin"

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

# Run in debug mode (separate terminal recommended)
debug:
    {{BIN}} --debug

# Run the TUI / Main executable
launch:
    {{BIN}}

# List installed plugins
list:
    {{BIN}} list

# Enable a plugin
enable name:
    {{BIN}} enable {{name}}

# Disable a plugin
disable name:
    {{BIN}} disable {{name}}

# Compile a .rs dish file
wash path:
    {{BIN}} wash {{path}}

# Load/Install a .dish plugin
load path:
    {{BIN}} load {{path}}

# Show version
version:
    {{BIN}} --version
