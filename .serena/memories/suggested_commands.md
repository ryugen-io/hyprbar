# Suggested Commands for kitchnsink

**Build & Check**:
-   `cargo check`: Check compilation.
-   `cargo check -p ks-wayland`: Check specific crate.
-   `cargo build`: Build dev.
-   `cargo build --release`: Build release.

**Running**:
-   `cargo run -p ks-bin`: Run the binary.
-   `cargo run --bin ks-bin`: Alternative.

**Code Quality**:
-   `cargo fmt`: Format code.
-   `cargo clippy`: Lint.

**Testing**:
-   `cargo test`: Run tests.

**Debugging**:
-   Use `RUST_LOG=debug cargo run ...` for logging.
