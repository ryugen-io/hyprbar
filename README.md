# Kitchn Sink

A versatile modular status bar and widget system for Wayland, powered by `kitchn` and `ratatui`.

## Features
- **Dynamic Plugin System**: Load Rust-based plugins (`.dish`) at runtime.
- **Flex-Grid Layout**: Modern layout engine with dynamic sizing and floating center.
- **Deep Theming**: Integration with `kitchn` for unified system styling.
- **Smart Scaling**: Pixel-perfect font scaling based on bar height.

## Structure

```
.
├── crates/
│   ├── ks-bin/      # Main Application
│   ├── ks-lib/      # Core Library (Interfaces, Config, State)
│   ├── ks-ui/       # UI Components (TUI widgets)
│   └── ks-wayland/  # Wayland Integration (Layer Shell)
├── examples/        # Plugin Examples
└── tools/           # Dev tools (wash/load)
```

## Plugin System

Plugins are dynamic libraries (`.dish`) compiled from Rust code.
They link against `ks-lib` and implement the `Dish` trait.

### Creating a Dish

See `examples/` for reference settings.
Plugins must expose `_create_dish` symbol.

## Development

```bash
# Build
just build

# Run (Debug)
just run

# Install
just install
```
