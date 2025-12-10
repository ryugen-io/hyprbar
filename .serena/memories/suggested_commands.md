# KitchnSink Suggested Commands

## Build & Run
- `cargo build --release`: Build specialized release binary (opt-level "z", stripped).
- `cargo run`: Run locally (might fail if Wayland socket not present, use in Sway/Hyprland).

## Install
- `cargo install --path .`: Install locally.
- `kitchn cook`: Update configuration (sink.toml) from Kitchn theme.

## Utils
- `just check`: Run cargo check.
- `just fmt`: Format code.
