default:
    @just --list

# Build the binary
build:
    cargo build -p ks-bin

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
    git add .

# Install the binary
install: pre-commit
    cargo install --path ks-bin --force

# Show project statistics (LOC, binary sizes)
stats:
    ../utils/kitchn/stats.sh .

# Dev helper: wash plugins from .wash to .load
dwash:
    @./ksdev --wash

# Dev helper: load plugins from .load
dload:
    @./ksdev --load

# Clean build artifacts (keeps .wash sources safe)
clean:
    cargo clean
    rm -rf .load
