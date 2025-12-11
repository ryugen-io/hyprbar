# KitchnSink Suggested Commands

## Build & Run
- `cargo build --release`: Build specialized release binary (opt-level "z", stripped).
- `cargo run`: Run locally (might fail if Wayland socket not present, use in Sway/Hyprland).

## Plugin Management
- `mojo tools/wash.mojo <file.rs>`: Build a plugin standalone (Mojo script).
- `ks-bin list`: List installed plugins and their status/metadata.
- `ks-bin enable/disable <name>`: Toggle plugin state.
- `ks-bin load <file.dish>`: Install and register a plugin.

## Install
- `just install`: Full install (fmt, check, build, install, pre-commit).
- `cargo install --path .`: Manual install.
- `kitchn cook`: Update configuration (sink.toml) from Kitchn theme.

## Utils
- `just check`: Run cargo check.
- `just fmt`: Format code.
