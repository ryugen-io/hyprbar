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
