# Kitchn Sink

A versatile modular status bar and widget system for Wayland, powered by `kitchn` and `ratatui`.

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
