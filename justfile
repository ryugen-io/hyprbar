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
    cargo install --path ks-bin --force

# Show project statistics (LOC, binary sizes)
stats:
    ../utils/kitchn/stats.sh .

# Dev helper: wash plugins from .wash to .load
dwash:
    @mojo tools/ksdev.mojo --wash

# Dev helper: load plugins from .load
dload:
    @mojo tools/ksdev.mojo --load

# Run debug inspector (Screenshot + Config)
inspect:
    @test -d tools/.venv || (python3 -m venv tools/.venv && tools/.venv/bin/pip install -r tools/requirements.txt)
    @tools/.venv/bin/python3 tools/debug_view.py

# Clean build artifacts (keeps .wash sources safe)
clean:
    cargo clean
    cargo cache -a
