# hyprbar

A modular Wayland status bar built with ratatui and smithay-client-toolkit.

## Features

- **Dynamic Plugin System**: Load Rust-based widgets (.so) at runtime
- **Flex-Grid Layout**: Modern layout engine with dynamic sizing
- **Popup Support**: Widgets can display floating popups on hover/click
- **Smart Scaling**: Pixel-perfect font scaling based on bar height

## Structure

```
src/
├── bin/hyprbar.rs    # CLI entry point
├── renderer/         # Rendering logic (modular)
├── wayland/          # Wayland layer shell integration
├── modules/          # Bootstrap, config, logging
├── widget.rs         # Widget trait
└── ...
```

## Widget System

Widgets are dynamic libraries (.so) implementing the `Widget` trait.
Located in `~/.local/share/hyprbar/widgets/`.

## Configuration

Config file: `~/.config/hypr/hyprbar.conf` (TOML)

## Development

```bash
just build      # Build
just install    # Install
just start      # Start daemon
just stop       # Stop daemon
just debug      # Debug mode with log terminal
```
