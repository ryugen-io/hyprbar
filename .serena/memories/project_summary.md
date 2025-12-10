# kitchnsink Project Summary

**Purpose**: A terminal-based system monitor or toolbar (sink) integrated with the `kitchn` ecosystem and Wayland. It uses `ratatui` for TUI rendering and `smithay-client-toolkit` for Wayland layer shell integration.

**Tech Stack**:
-   **Language**: Rust (2024 edition)
-   **Core**: `ks-core` (rendering logic, state, `ratatui`)
-   **Wayland**: `ks-wayland` (SCTK 0.19, `wayland-client`, layer shell)
-   **Binary**: `ks-bin` (entry point, tokio runtime).

## Configuration Pattern

`kitchnsink` follows the **Dumb Receiver** pattern:
1.  **Layout & Style**: Reads `~/.config/kitchnsink/sink.toml`.
    -   This file contains all resolved colors (BG/FG) and layout settings.
    -   It is expected to be generated/updated by the `kitchn` tool's "cook" process.
2.  **Context**: Uses `k-lib` (via `ryugen-io/kitchN`) for logging context and shared types.

```rust
// CORRECT: Read from SinkConfig
let bg_hex = &state.config.style.bg;

// INCORRECT: Do NOT read from Cookbook directly
// let bg_hex = state.cookbook.theme.colors.get("bg");
```

**Structure**:
-   Workspace with 3 crates: `ks-core`, `ks-wayland`, `ks-bin`.
-   `ks-wayland` handles Wayland protocols and event loop integration.
-   `ks-bin` drives the main loop, handling `tokio` async flow.

**Key Conventions**:
-   See `code-style-guide.md` (PascalCase structs, snake_case fns, `anyhow` errors).
-   `kitchn` presets for logging/UI.
