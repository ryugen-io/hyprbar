# KitchnSink Plugin API

KitchnSink supports dynamic plugins written in Rust. Plugins are compiled as shared libraries (`.so` / `.dylib`) and loaded at runtime.

## Core Concepts

Plugins implement the `Dish` trait. A plugin corresponds to a "dish" in the "kitchen sink".

### The `Dish` Trait

```rust
pub trait Dish: Send + Sync {
    fn name(&self) -> &str;
    fn update(&mut self, dt: Duration);
    /// Return the required width of the dish (e.g. for calculating layout).
    fn width(&self, state: &BarState) -> u16;
    fn render(&self, area: Rect, buf: &mut Buffer, state: &BarState);

    /// Optional: Set the instance configuration name.
    /// Implement this if your dish supports multiple instances (e.g. "TextArea#2").
    /// Store this name and use it to look up configuration instead of the hardcoded name.
    #[allow(unused_variables)]
    fn set_instance_config(&mut self, name: String) {}
}
```

### Supporting Multiple Instances (Aliasing)

If you want your Dish to be usable multiple times with different configurations (e.g. `[dish.Clock]` and `[dish.Clock2]`), implement `set_instance_config`:

```rust
struct MyDish {
    config_name: String,
}

impl Dish for MyDish {
    fn set_instance_config(&mut self, name: String) {
        self.config_name = name;
    }
    
    // In render/width, use self.config_name instead of "MyDish"
}
```
- **name()**: Returns the display name of the plugin (mostly for debugging).
- **update(dt)**: Called periodically with the delta time. Use this to poll system stats, but avoid blocking operations!
- **width(state)**: Returns the width in characters that the plugin needs.
- **render(area, buf, state)**: Renders the plugin content to the Ratatui buffer.

### Accessing State

The `BarState` struct provides access to the global configuration and system state.

```rust
pub struct BarState {
    pub config: SinkConfig,
    pub system: sysinfo::System, // access to CPU, RAM, etc.
}
```

## Styling Guidelines

**Plugins MUST use the theme colors defined in `sink.toml`.** Do not hardcode colors.

Access colors via `state.config.style`:

```toml
[style]
bg = "#161925"
fg = "#F8F8F2"
primary = "#FF79C6"
success = "#50FA7B"
error = "#FF5555"
```

### Standard Palette

Plugins should try to use the following semantic keys to stay consistent with the theme:

**Semantic:**
- `primary`, `secondary`
- `success`, `error`, `warn`, `info`, `orange`

**UI:**
- `bg`, `fg`, `selection_bg`, `selection_fg`, `cursor`

**Standard:**
- `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
- `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`

### Parsing Colors

The `Color` type from `ratatui` does NOT support parsing hex strings directly. Use the provided `ColorResolver` helper:

```rust
use ks_core::prelude::*; // Import commonly used types including ColorResolver

// ... inside render() ...

// Correct way to parse colors
let success_color = state.config.style.success
    .as_deref()
    .map(|s| {
        let c = ColorResolver::hex_to_color(s);
        Color::Rgb(c.r, c.g, c.b)
    });

// Apply to cell
if let Some(color) = success_color {
    buf[(x, y)].set_fg(color);
}
```

## Creating a Plugin

1. Create a new `.rs` file (e.g., `my_plugin.rs`).
2. Implement the `Dish` trait struct.
3. Export the `_create_dish` function:

```rust
use ks_core::prelude::*;

struct MyPlugin;

// Metadata must be included at the top of the file!
//! Name: My Plugin
//! Version: 0.1.0
//! Author: Me
//! Description: A test plugin
//! Dependency: chrono = "0.4"

impl Dish for MyPlugin {
    fn name(&self) -> &str { "MyPlugin" }
    fn update(&mut self, _: Duration) {}
    
    fn width(&self, _: &BarState) -> u16 { 
        10 
    }
    
    fn render(&self, area: Rect, buf: &mut Buffer, state: &BarState) {
        let fg = state.config.style.fg.as_str().map(|s| {
             let c = ColorResolver::hex_to_color(s);
             Color::Rgb(c.r, c.g, c.b)
        }).unwrap_or(Color::White);
        
        buf.set_string(area.x, area.y, "Hello!", Style::default().fg(fg));
    }
}

#[no_mangle]
pub extern "Rust" fn _create_dish() -> Box<dyn Dish> {
    Box::new(MyPlugin)
}
```

## Development Workflow

1. Place your plugin source in `.wash/`.
2. Run `just dwash` to compile and move it to `.load/`.
3. Use `just install` or run `ks-bin` to test.
