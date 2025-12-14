# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
