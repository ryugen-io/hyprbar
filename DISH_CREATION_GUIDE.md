# Comprehensive Guide to Creating Kitchnsink Dishes 🍽️

This guide covers everything you need to create powerful, dynamic, and theme-aware plugins (Dishes) for `kitchnsink`.

## 1. Anatomy of a Dish

A Dish is a dynamic library (`.dish`) compiled from a single Rust source file. It must implement the `Dish` trait and export a creator function.

### Required Imports
```rust
use ks_core::prelude::*; // Provides Dish, BarState, ratatui types, etc.
```

### The Recipe (Metadata)
Every dish **must** start with these comments to be recognized:
```rust
//! Name: Advanced Dish
//! Version: 1.0.0
//! Author: Your Name
//! Description: Demonstrating all features
```

### The Structure
Your dish struct holds its own state. It must be `Send + Sync`.
```rust
struct MyDish {
    // Internal state
    counter: usize,
    last_update: Duration,
    // Configuration cache
    label: String,
}
```

## 2. Implementing the Dish Trait

### `name(&self)`
Returns the unique identifier for your dish. This is used for logging and debugging.
```rust
fn name(&self) -> &str { "MyAdvancedDish" }
```

### `width(&self, state: &BarState)`
Calculates how much horizontal space (in cells) your dish needs.
*   You can access `state` to make this dynamic (e.g., if you show more text when CPU is high).
```rust
fn width(&self, _state: &BarState) -> u16 {
    // 10 chars for label + 4 for counter
    14 
}
```

### `update(&mut self, dt: Duration)`
Called every frame tick (usually 60Hz). Use this for:
*   Animation timing
*   Polling external resources (carefully!)
*   Updating internal counters
**⚠️ Warning**: This runs on the main thread. Do **not** perform blocking I/O (like large file reads or network requests) here. Use `std::thread::spawn` or similar if you need to fetch data, and update your state via shared memory (Mutex/Atomic).
```rust
fn update(&mut self, dt: Duration) {
    self.last_update += dt;
    if self.last_update.as_secs() >= 1 {
        self.counter += 1;
        self.last_update = Duration::ZERO;
    }
}
```

### `render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, dt: Duration)`
The core visual logic. Draws your widget using `ratatui`.
*   `area`: The specific rectangle allocated to your dish.
*   `buf`: The target buffer to write to.
*   `state`: Global system state (CPU, Mem, Config).
*   `dt`: Time delta since last frame (for smooth animations).

## 3. Accessing "All Features" via `BarState`

The `state` parameter in `render` and `width` is your gateway to the system.

### System Stats
```rust
let cpu_usage = state.cpu; // f32 (0.0 - 100.0)
let ram_usage = state.mem; // f32 (0.0 - 100.0)
let time_str = &state.time; // Current time string
```

### Configuration (`state.config`)
Access `kitchnsink` specific settings (`sink.toml`).
```rust
// Access global style colors
let fg = state.config.style.fg; 
let bg = state.config.style.bg;
// Access success/error/warning colors
let ok_color = state.config.style.success.as_deref().unwrap_or("#00ff00");
```

### Global Kitchen Context (`state.cookbook`)
Access the wider `kitchn` ecosystem configuration (`layout.toml`, `theme.toml`).
```rust
// Access the active theme's detailed palette
let special_color = state.cookbook.theme.colors.get("purple").map(|s| s.as_str());

// Access icons based on active set (nerdfont/ascii)
let icon = if state.cookbook.theme.settings.active_icons == "nerdfont" {
    "⚡" 
} else {
    "P"
};
```

### Custom Dish Configuration
Users can configure your dish in `sink.toml`:
```toml
[dish.my_dish]
label = "CPU Core 1"
alert_threshold = 80
```

Access this in your code:
```rust
// In render or query methods
if let Some(config_table) = state.config.dish.get("my_dish").and_then(|v| v.as_table()) {
    if let Some(val) = config_table.get("alert_threshold").and_then(|v| v.as_integer()) {
        // Use custom threshold...
    }
}
```

## 4. Logging
You can use standard logging macros. These are captured by `kitchnsink` and written to the system log file.
```rust
log::info!("My dish initialized!");
log::warn!("Something strange happened");
```

## 5. Full Example: "PowerUser" Dish

```rust
//! Name: PowerUser
//! Version: 1.0.0
//! Author: Kitchn Master
//! Description: Advanced system monitor with theming and config

use ks_core::prelude::*;

struct PowerDish {
    last_tick: Duration,
    show_details: bool,
}

impl Dish for PowerDish {
    fn name(&self) -> &str { "PowerUser" }

    fn width(&self, state: &BarState) -> u16 {
        if self.show_details { 20 } else { 10 }
    }

    fn update(&mut self, dt: Duration) {
        self.last_tick += dt;
        // Toggle view every 5 seconds
        if self.last_tick.as_secs() >= 5 {
            self.show_details = !self.show_details;
            self.last_tick = Duration::ZERO;
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, _dt: Duration) {
        // 1. Resolve Colors from Theme
        let fg_hex = &state.config.style.fg;
        let bg_hex = &state.config.style.bg;
        let accent_hex = state.config.style.accent.as_deref().unwrap_or("#ff00ff");
        
        let fg = ColorResolver::hex_to_color(fg_hex);
        let accent = ColorResolver::hex_to_color(accent_hex);

        // 2. Custom Config Check
        let header = state.config.dish.get("power_user")
            .and_then(|t| t.get("header"))
            .and_then(|v| v.as_str())
            .unwrap_or("SYS");

        // 3. Render
        let text = if self.show_details {
            format!("{} CPU:{:.0}% MEM:{:.0}%", header, state.cpu, state.mem)
        } else {
            format!("{} OK", header)
        };

        // Use Ratatui to draw
        buf.set_string(area.x, area.y, text, Style::default().fg(fg.into()));
        
        // Draw a status dot
        let dot_color = if state.cpu > 90.0 { Color::Red } else { accent.into() };
        if area.width > 2 {
            buf.get_mut(area.x + area.width - 1, area.y).set_char('•').set_fg(dot_color);
        }
    }
}

#[no_mangle]
pub extern "Rust" fn _create_dish() -> Box<dyn Dish> {
    Box::new(PowerDish { 
        last_tick: Duration::ZERO,
        show_details: false 
    })
}
```

## 6. Building & Installing

1.  **Build**: `kitchnsink wash power_user.rs`
2.  **Install**: `kitchnsink load power_user.dish`
3.  **Activate**: Add `"PowerUser"` to `sink.toml`.
