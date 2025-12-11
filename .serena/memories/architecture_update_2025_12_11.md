# Architecture & Documentation Update (2025-12-11)

## Overview
We have significantly refined the `kitchnsink` documentation to clarify its relationship with the `kitchn` ecosystem (`k_lib`).

## Key Artifacts
-   **`ARCHITECTURE.md`**: A new standalone document featuring a "Sweet Dracula" themed Mermaid diagram. It details the "Entangled Workflow" where `kitchnsink` relies on `k_lib` components.
-   **`DISH_CREATION_GUIDE.md`**: Rewritten for a better narrative flow, linking to the new architecture doc.

## Architectural Insights (The "Entangled Workflow")
1.  **Configuration**: `kitchnsink` has its own `sink.toml` with a `[style]` section, but this is **generated** by the `kitchn cook` command. It is a local operational config derived from the central `kitchn` theme.
2.  **Icons**: Retrieved dynamically from the shared `ARC<Cookbook>` (from `k_lib`). `kitchnsink` does *not* convert icons itself; it asks `k_lib` for "lightning" and gets a string back (Nerdfont/ASCII).
3.  **Logging**: Users logging strings (presets) are sourced from `k_lib`. Crucially, `k_lib` embeds a `defaults.toml` at compile time, which provides system presets (`boot_ok`, `shutdown`) even if the user's `cookbook.toml` is empty.
4.  **Plugin System**: Managed via `registry.bin` for persistence.

## Development Workflow
-   Use `ksdev.mojo` for `wash` (compile) and `load` (install) of dishes.
-   Metadata is checked via `ksdev.mojo --check-dish`.
