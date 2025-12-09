---
trigger: always_on
glob:
description: Code style guide for kitchnsink - consistent with kitchn ecosystem
---

# kitchnsink Code Style Guide

This guide ensures consistency with the kitchn ecosystem. kitchnsink depends on `kitchn_lib` and should follow the same patterns.

## Rust Edition & Tooling

- **Edition**: 2024 (Rust 2024)
- **Formatting**: `cargo fmt` before commits
- **Linting**: `cargo clippy` with no warnings
- **Release Profile**: LTO, strip, opt-level "z" for size optimization

## Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Functions | snake_case | `render_frame`, `load_config` |
| Variables | snake_case | `bar_state`, `effect_manager` |
| Structs/Enums | PascalCase | `BarRenderer`, `BarState` |
| Constants | SCREAMING_SNAKE | `SYSTEM_DICTIONARY` |
| Modules | snake_case | `layer_shell`, `blitter` |

### Suffixes

- Config structs: `*Config` (e.g., `ThemeConfig`, `LayoutConfig`)
- Error enums: `*Error` (e.g., `ConfigError`, `RenderError`)
- State structs: `*State` (e.g., `BarState`)
- Builder pattern: `*Builder`

## Import Organization

Group imports in this order with blank lines between groups:

```rust
// 1. External crates
use anyhow::{Context, Result};
use log::debug;
use ratatui::buffer::Buffer;

// 2. Standard library
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

// 3. Internal crates/modules
use crate::renderer::BarRenderer;
use crate::state::BarState;

// 4. kitchn_lib
use kitchn_lib::config::Cookbook;
use kitchn_lib::logger;
```

## Error Handling

### Libraries (ks-core, ks-wayland)

Use `thiserror` for custom error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Buffer allocation failed: {0}")]
    BufferAlloc(String),
    #[error("Effect error: {0}")]
    Effect(#[from] tachyonfx::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Binaries (ks-bin)

Use `anyhow` with context:

```rust
use anyhow::{Context, Result};

fn init_wayland() -> Result<()> {
    let display = Connection::connect_to_env()
        .context("Failed to connect to Wayland display")?;
    // ...
    Ok(())
}
```

## Struct Patterns

### Configuration Structs

Always derive Serde traits for config:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct BarConfig {
    pub height: u32,
    pub position: Position,
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate: u32,
}

fn default_refresh_rate() -> u32 {
    60
}
```

### State Structs

```rust
#[derive(Debug)]
pub struct BarState {
    pub cpu: f32,
    pub mem: f32,
    pub time: String,
    pub cookbook: Cookbook,
}
```

## Function Design

- **Small & focused**: One responsibility per function
- **Early returns**: Use `?` operator liberally
- **Private by default**: Only `pub` what's needed
- **Documentation**: `///` for public API

```rust
/// Renders a single frame to the internal buffer.
///
/// Updates the buffer with current state and applies effects.
pub fn render_frame(&mut self, state: &BarState, dt: Duration) -> Result<()> {
    self.update_widgets(state)?;
    self.apply_effects(dt)?;
    Ok(())
}

fn update_widgets(&mut self, state: &BarState) -> Result<()> {
    // Private helper - no docs needed
    // ...
    Ok(())
}
```

## Logging

Use the `log` crate facade:

```rust
use log::{debug, info, warn, error};

pub fn init_bar() -> Result<()> {
    debug!("Initializing bar renderer");

    // For verbose debug info
    if log::log_enabled!(log::Level::Debug) {
        debug!("Buffer size: {}x{}", width, height);
    }

    Ok(())
}
```

### kitchn Logger Integration

For user-visible messages, use kitchn_lib::logger:

```rust
use kitchn_lib::config::Cookbook;
use kitchn_lib::logger;

fn log_startup(config: &Cookbook) {
    logger::log_to_terminal(config, "info", "BAR", "kitchnsink started");
}
```

## Testing

Place tests at the bottom of each file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_mock_state() -> BarState {
        // Helper for tests
    }

    #[test]
    fn test_render_frame() {
        let mut renderer = BarRenderer::new(100, 24);
        let state = create_mock_state();

        let result = renderer.render_frame(&state, Duration::from_millis(16));
        assert!(result.is_ok());
    }
}
```

## kitchn Integration

### Loading Configuration

```rust
use kitchn_lib::config::Cookbook;

fn main() -> Result<()> {
    // Load kitchn configuration (themes, icons, logging settings)
    let cookbook = Cookbook::load()
        .context("Failed to load kitchn cookbook")?;

    // Access theme colors
    let bg_color = cookbook.theme.colors.get("bg")
        .map(|s| s.as_str())
        .unwrap_or("#1e1e2e");

    Ok(())
}
```

### Logging with kitchn Theme

```rust
use kitchn_lib::logger;

fn log_event(cookbook: &Cookbook, level: &str, msg: &str) {
    logger::log_to_terminal(cookbook, level, "SINK", msg);

    if cookbook.layout.logging.write_by_default {
        let _ = logger::log_to_file(cookbook, level, "SINK", msg, Some("kitchnsink"));
    }
}
```

## Wayland/Ratatui Patterns

### Buffer Management

```rust
pub struct BarRenderer {
    buffer: Buffer,
    effects: EffectManager,
}

impl BarRenderer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            buffer: Buffer::empty(Rect::new(0, 0, width, height)),
            effects: EffectManager::new(),
        }
    }
}
```

### Layer Shell Setup

```rust
// Prefer explicit configuration over magic defaults
let layer_surface = LayerSurface::builder()
    .size((width, height))
    .anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT)
    .exclusive_zone(height as i32)
    .namespace("kitchnsink")
    .build();
```

### Blitting Pattern

```rust
/// Converts ratatui Buffer cells to ARGB pixels.
pub fn blit_buffer_to_pixels(
    buffer: &Buffer,
    pixels: &mut [u8],
    cookbook: &Cookbook,
) {
    for (i, cell) in buffer.content().iter().enumerate() {
        let color = resolve_color(cell.fg, cookbook);
        let offset = i * 4;
        pixels[offset..offset + 4].copy_from_slice(&color.to_argb());
    }
}
```

## Async Patterns (ks-bin)

Use tokio for async runtime:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let cookbook = Cookbook::load()?;

    // Event loop
    loop {
        tokio::select! {
            _ = wayland_events() => {},
            _ = tick_interval.tick() => {
                render_frame(&mut state)?;
            }
        }
    }
}
```

## Don'ts

- **Don't** use `unwrap()` in production code - use `?` or `expect("reason")`
- **Don't** over-comment obvious code
- **Don't** add unnecessary abstractions for one-time operations
- **Don't** mix sync and async without good reason
- **Don't** ignore clippy warnings
- **Don't** use `unsafe` without thorough documentation

## Crate Structure

```
kitchnsink/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── ks-core/            # Rendering logic
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs # BarRenderer, Buffer management
│   │       └── state.rs    # BarState
│   ├── ks-wayland/         # Wayland integration
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── layer_shell.rs
│   │       └── blitter.rs
│   └── ks-bin/             # Binary entry point
│       └── src/
│           └── main.rs
```
