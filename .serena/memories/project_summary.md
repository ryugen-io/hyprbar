# kitchnsink Project Summary

**Purpose**: A terminal-based system monitor or toolbar (sink) integrated with the `kitchn` ecosystem and Wayland. It uses `ratatui` for TUI rendering and `smithay-client-toolkit` for Wayland layer shell integration.

**Tech Stack**:
-   **Language**: Rust (2024 edition)
-   **Core**: `ks-core` (rendering logic, state, `ratatui`)
-   **Wayland**: `ks-wayland` (SCTK 0.19, `wayland-client`, layer shell)
-   **Binary**: `ks-bin` (entry point, tokio runtime, integration)
-   **Dependencies**: `kitchn_lib` (config/theme), `tachyonfx` (effects)

**Structure**:
-   Workspace with 3 crates: `ks-core`, `ks-wayland`, `ks-bin`.
-   `ks-wayland` handles Wayland protocols and event loop integration.
-   `ks-bin` drives the main loop, handling `tokio` (or similar) async flow.

**Key Conventions**:
-   See `code-style-guide.md` (PascalCase structs, snake_case fns, `anyhow` errors).
-   `kitchn` presets for logging/UI.
