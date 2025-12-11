# kitchnsink Project Summary

**Purpose**: A terminal-based system monitor or toolbar (sink) integrated with the `kitchn` ecosystem and Wayland. It uses `ratatui` for TUI rendering and `smithay-client-toolkit` for Wayland layer shell integration.

**Tech Stack**:
-   **Language**: Rust (2024 edition)
-   **Core**: `ks-core` (rendering logic, state, `ratatui`, `Dish` trait)
-   **Wayland**: `ks-wayland` (SCTK 0.19, `wayland-client`, layer shell)
-   **Binary**: `ks-bin` (entry point, tokio runtime).
    -   Modularized architecture (`modules/`): `cli`, `logging`, `config`, `daemon`, `watcher`, `build`, `install`, `runner`.
    -   Fully async IO using `tokio::fs` and `tokio::process`.

## Dish System (Modular Widgets)
The bar uses a plugin-based "Dish" architecture (`ks-core/src/dish.rs`):
-   **Dish Trait**: Defines `render()` and `width()` for all components.
-   **Layout**: `sink.toml` defines `modules_left`, `modules_center`, `modules_right` lists.
-   **Configuration**: Specific settings (e.g., text content, symbols) live in `[dish.<name>]` sections of `sink.toml`.

## Configuration Pattern

`kitchnsink` follows the **Dumb Receiver** pattern:
1.  **Layout & Style**: Reads `~/.config/kitchnsink/sink.toml`.
    -   Contains resolved colors (`window_bg` vs `bg`) and layout/dish settings.
    -   Expected to be generated/updated by the `kitchn` tool's "cook" process.
2.  **Context & Logging**: Uses `k-lib` (via `ryugen-io/kitchN`) for logging context.
    -   **String Externalization**: All user-facing logs use the `Cookbook` dictionary (keys: `sink_startup`, `sink_exit`, `dish_loaded`, etc.).
    -   Users customize these messages in `cookbook.toml`.

```rust
// CORRECT: Read from SinkConfig
let bg_hex = &state.config.style.bg;

// CORRECT: Read log strings from Cookbook Dictionary
let msg = cookbook.dictionary.presets.get("sink_startup").map(|p| p.msg.clone())...;
```

**Structure**:
-   Workspace with 3 crates: `ks-core`, `ks-wayland`, `ks-bin`.
-   `ks-wayland` handles Wayland protocols and event loop integration.
-   `ks-bin` drives the main loop, handling `tokio` async flow and signal handling (SIGTERM for graceful debug toggle).

**Key Conventions**:
-   See `code-style-guide.md` (PascalCase structs, snake_case fns, `anyhow` errors).
-   `kitchn` presets for logging/UI.
