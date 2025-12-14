# kitchnsink Project Summary (Updated 2025-12-14)

**Purpose**: A terminal-based system monitor or toolbar (sink) integrated with the `kitchn` ecosystem and Wayland.

**Tech Stack**:
-   **Language**: Rust (2024 edition)
-   **Core**: `ks-core` (rendering logic, state, `ratatui`, `Dish` trait)
-   **Wayland**: `ks-wayland` (SCTK 0.19, `wayland-client`, layer shell)
-   **Binary**: `ks-bin` (entry point, tokio runtime).
-   **Tooling**: Combined Rust + Mojo workflow.
    - Helper scripts (`stats`, `wash`, `ksdev`) located in `../utils/kitchnsink` (compiled binaries).
    - `justfile` provides comprehensive CLI wrappers (`just start`, `just stop`, etc.).

## Dish System (Modular Widgets)
The bar uses a plugin-based "Dish" architecture (`ks-core/src/dish.rs`) with a robust Registry and Metadata system:

### 1. Metadata Headers
Plugins (Dishes) declare metadata directly in their source (`.rs`) files using comments.

### 2. Registry (`registry.bin`)
`ks-bin` maintains a persistent binary registry of installed plugins at `~/.local/share/kitchnsink/dishes/registry.bin`.

### 3. Dish Trait
-   **Dish Trait**: Defines `render()` and `width()` for all components.
-   **Layout**: `sink.toml` defines `modules_left`, `modules_center`, `modules_right` lists.

## Configuration Pattern

`kitchnsink` follows the **Dumb Receiver** pattern:
1.  **Layout & Style**: Reads `~/.config/kitchnsink/sink.toml`.
2.  **Context & Logging**: Uses `k-lib`.

**Structure**:
-   Workspace with 3 crates: `ks-core`, `ks-wayland`, `ks-bin`.
-   `ks-wayland` handles Wayland protocols.
-   `ks-bin` drives the main loop.

### Logging Architecture
-   **Tracing**: Primary logging facade.
-   **File Logging**: Integrated with `kitchn_lib`.

## Development Tools
- `ksdev`, `wash`, `stats`: Compiled binaries in `../utils/kitchnsink`.
- `justfile`: Central command runner for build, test, run, and CLI management.
