# hyprsbar

A modular Wayland status bar built with ratatui and smithay-client-toolkit.

## Features

- **Dynamic Plugin System**: Load Rust-based widgets (.so) at runtime
- **Flex-Grid Layout**: Modern layout engine with dynamic sizing
- **Popup Support**: Widgets can display floating popups on hover/click
- **Smart Scaling**: Pixel-perfect font scaling based on bar height

## Structure

```
src/
├── bin/hyprsbar.rs    # CLI entry point
├── renderer/         # Rendering logic (modular)
├── wayland/          # Wayland layer shell integration
├── modules/          # Bootstrap, config, logging
├── widget.rs         # Widget trait
└── ...
```

## Widget System

Widgets are dynamic libraries (.so) implementing the `Widget` trait.
Located in `~/.local/share/hyprsbar/widgets/`.

## Configuration

Config file: `~/.config/hypr/hyprsbar.conf` (TOML)

`hyprsbar` reads only `hyprsbar.conf` (metadata `type = bar`).  
`hyprsink` may generate/update this file, but `hyprsbar` does not load `hyprsink.conf`.

Minimal header:

```conf
# hypr metadata
# type = bar
```

## Development

```bash
just build      # Build
just install    # Install
just start      # Start daemon
just stop       # Stop daemon
just debug      # Debug mode with log terminal
```
