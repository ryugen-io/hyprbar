# KitchnSink Suggested Commands

## Build & Run
- `cargo build --release`: Build specialized release binary (opt-level "z", stripped).
- `cargo run`: Run locally (might fail if Wayland socket not present, use in Sway/Hyprland).

## Plugin Management
- `just wash <file.rs>`: Compile a plugin (.rs -> .dish).
- `just list`: List installed plugins and their status/metadata.
- `just enable/disable <name>`: Toggle plugin state.
- `just load <file.dish>`: Install and register a plugin.

## CLI Control
- `just start`: Start the bar daemon.
- `just stop`: Stop the bar daemon.
- `just restart`: Restart the bar daemon.
- `just debug`: Run in debug mode (separate terminal).
- `just launch`: Launch the TUI/Server manually.

## Install
- `just install`: Full install (fmt, check, build, install, pre-commit).
- `cargo install --path .`: Manual install.
- `kitchn cook`: Update configuration (sink.toml) from Kitchn theme.

## Utils
- `just check`: Run cargo check.
- `just fmt`: Format code.
