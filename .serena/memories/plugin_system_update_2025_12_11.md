# Plugin System Update (2025-12-11)

## Plugin Registry
- **Persistent State:** Added `registry.bin` using `bincode` serialization for installed plugins.
- **Commands:** Added `ks-bin list`, `ks-bin enable/disable`, and updated `load`.
- **Runtime:** `PluginManager` now respects enabled/disabled state.

## Metadata System
- **Headers:** Plugins use `//! Name: Value` style headers.
- **Injection:** `wash` and `build.rs` inject a `_plugin_metadata` extern function returning JSON.
- **Verification:** `ks-bin list` shows proper metadata.

## Standalone Mojo Builder
- **Tool:** `tools/wash.mojo`.
- **Function:** Builds `.dish` plugins standalone without a full `ks-bin` install.
- **Interop:** Uses Python (`shutil`, `subprocess`) for build orchestration.
- **Fix:** Implements `#[unsafe(no_mangle)]` for Rust 2024 compliance.
