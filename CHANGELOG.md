# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-12-25

### Added

- **Event Bus**: Implemented inter-dish communication via `tokio::sync::broadcast` in `ks-lib`.
- **Animations**: Added declarative animation system using `tachyonfx`. Configurable via `[style.animation]` in `sink.toml`.
- **Hooks**: Added `on_load` and `on_unload` lifecycles to `BarRenderer`.

### Changed

- **API**: Updated `Dish::update` signature to include `&BarState` (Breaking Change).
- **Examples**: Updated all example dishes (`battery`, `datetime`, `separator`, `text_area`, `tray_space`) to match new API.

### Fixed

- **Runtime**: Solved SIGSEGV crash caused by stale `.dish` files (ABI mismatch).
- **Logic**: Fixed timer logic in `datetime.rs` and unused variable warnings in `battery.rs`.

## [0.2.0] - 2025-12-14

### Changed

- **Structure**: Moved all crates to `crates/` directory.
- **Renamed**: `ks-core` is now `ks-lib`.
- **Plugins**: Updated plugin system to use `ks-lib` imports.

### Fixed

- Fixed `battery` plugin compilation (syntax error).
- Fixed `PLUGIN_API` documentation references.

## [0.1.0] - Initial Release

### Added

- Core `ks-bin` application.
- `ks-ui` TUI widget library.
- `ks-wayland` layer shell integration.
- Plugin system with `wash` tool.
