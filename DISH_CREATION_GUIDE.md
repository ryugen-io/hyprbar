# Comprehensive Guide to Creating Kitchnsink Dishes 🍽️

This guide will walk you through the process of creating a powerful, dynamic plugin (Dish) for `kitchnsink`. Let's build an "Advanced Dish" together, exploring how to access system state, handle configuration, and render beautiful UI along the way.

## 1. Setting up the Environment

To get started, we need to prepare our workspace. A Dish is simply a single Rust source file that `kitchnsink` compiles into a dynamic library.

Create a new file named `advanced_dish.rs` and bring the necessary tools into scope:

```rust
use ks_core::prelude::*; // Provides Dish, BarState, ratatui types, etc.
```

## 2. Defining Metadata

Now, we need to ensure the system recognizes your creation. Every dish **must** begin with a specific set of metadata comments.

Add the following block to the top of your file to identify your plugin:

```rust
//! Name: Advanced Dish
//! Version: 1.0.0
//! Author: Your Name
//! Description: Demonstrating all features
//! Dependency: chrono = "0.4"
```

### Dependency Management
Use `//! Dependency:` lines to include external crates. The build tool (`wash`) will automatically add them to the temporary `Cargo.toml`.

Format: `//! Dependency: crate = "version"`

### Adding Dependencies
You can now define external crate dependencies directly in your source file using the `//! Dependency:` syntax. These will be automatically injected during the build process.

**Format:** `//! Dependency: crate_name = "version"`

Example:
```rust
//! Dependency: serde = { version = "1.0", features = ["derive"] }
//! Dependency: reqwest = { version = "0.11", features = ["blocking"] }
```

> **Note**: `kitchnsink` pre-includes `ks-core`, `ratatui`, and `tachyonfx`. You only need to declare *extra* crates.

## 3. Creating the State Struct

With the setup complete, it's time to define your dish's memory. The struct you create here will hold the state that needs to persist between frames, and it must be thread-safe (`Send + Sync`).

For this walkthrough, let's track a simple counter and cache a configuration string:

```rust
struct MyDish {
    // Internal state to track time and updates
    counter: usize,
    last_update: Duration,
    // Configuration cache to avoid looking up config every frame
    label: String,
}
```

## 4. Implementing the Dish Trait

This is where the magic happens. Implementing the `Dish` trait provides the core interface that `kitchnsink` uses to interact with your plugin.

### Naming and Sizing

First, let's identify the dish and tell the bar how much screen real estate we require.

```rust
impl Dish for MyDish {
    fn name(&self) -> &str { "MyAdvancedDish" }

    fn width(&self, _state: &BarState) -> u16 {
        // We can calculate width dynamically here.
        // We'll reserve 14 cells: 10 for the label + 4 for the counter.
        14 
    }
```

### Handling Updates

Moving on to the heartbeat of your plugin: the update loop. This method fires every frame (tick), giving you the perfect place to handle animation timing or internal logic.

**Note:** This runs on the main thread, so keep it lightweight—avoid blocking operations like network requests here.

```rust
    fn update(&mut self, dt: Duration) {
        self.last_update += dt;
        
        // Let's maximize the excitement by updating the counter once every second
        if self.last_update.as_secs() >= 1 {
            self.counter += 1;
            self.last_update = Duration::ZERO;
        }
    }
```

### Rendering the UI

Now for the visual payoff. The `render` method hands you a `ratatui` buffer, which is your canvas for drawing.

You have access to the global `state` object, which is your window into the rest of the system:
- `state.cpu`, `state.mem`: Real-time system stats
- `state.config`: `kitchnsink` specific settings
- `state.cookbook`: Global `kitchn` ecosystem settings (themes, icons)

```rust
    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, _dt: Duration) {
        // 1. Resolve Colors from Theme
        // Let's grab the color palette from the global configuration
        let fg_hex = &state.config.style.fg;
        let bg_hex = &state.config.style.bg;
        let accent_hex = state.config.style.accent.as_deref().unwrap_or("#ff00ff");
        
        let fg = ColorResolver::hex_to_color(fg_hex);
        let accent = ColorResolver::hex_to_color(accent_hex);

        // 2. Custom Config Check
        // We can also check if the user has customized our header in sink.toml
        let header = state.config.dish.get("my_dish")
            .and_then(|t| t.get("header"))
            .and_then(|v| v.as_str())
            .unwrap_or("SYS");

        // 3. Render
        // Time to compose our final string
        let text = format!("{} {}", header, self.counter);

        // Draw the text to the buffer using the resolved colors form our theme
        buf.set_string(area.x, area.y, text, Style::default().fg(fg.into()));
        
        // 4. Dynamic Visuals
        // Finally, let's add a dynamic touch: a status dot that turns red under high load
        // We retrieve the icon from k_lib matching the active set (nerdfont/ascii)
        let icon = if state.cookbook.theme.settings.active_icons == "nerdfont" {
            state.cookbook.icons.nerdfont.get("lightning").map(|s| s.as_str()).unwrap_or("⚡")
        } else {
            state.cookbook.icons.ascii.get("lightning").map(|s| s.as_str()).unwrap_or("L")
        };

        // If CPU is high, override color to red, otherwise use accent
        let dot_color = if state.cpu > 90.0 { Color::Red } else { accent.into() };

        if area.width > 2 {
            // Draw the retrieved icon
            buf.set_string(area.x + area.width - 1, area.y, icon, Style::default().fg(dot_color));
        }
    }
}
```

## 5. Exporting the Dish

To wrap things up, we need to export a creator function. This is how `kitchnsink` discovers and instantiates your dish.

```rust
#[no_mangle]
pub extern "Rust" fn _create_dish() -> Box<dyn Dish> {
    Box::new(MyDish { 
        counter: 0,
        last_update: Duration::ZERO,
        label: "My Dish".to_string(),
    })
}
    })
}
```

## 6. Supporting Multiple Instances (Optional)

If you want users to be able to use your Dish multiple times with different configurations (e.g. `[dish.Clock1]` and `[dish.Clock2]`), you can implement the `set_instance_config` method.

In `sink.toml`, users can then write:
```toml
# Layout
modules_right = ["MyDish#Instance1", "MyDish#Instance2"]

[dish.Instance1]
header = "WORK"

[dish.Instance2]
header = "HOME"
```

In your code:
```rust
impl Dish for MyDish {
    // ...
    fn set_instance_config(&mut self, name: String) {
        // Store the alias (e.g. "Instance1") to look up config later
        self.config_key = name; 
    }
    
    fn render(...) {
        // Use self.config_key instead of "MyDish"
        let header = state.config.dish.get(&self.config_key)...
    }
}
```


## 6. Building & Installing

You've written the code, now let's bring it to life.

### The Manual Way
1.  **Build**: Run `kitchnsink wash advanced_dish.rs` to compile your work.
2.  **Install**: Run `kitchnsink load advanced_dish.dish` to move it to the plugins folder.
3.  **Activate**: Add `"MyAdvancedDish"` to your `sink.toml` configuration file to see it in action.

### The Developer Way (Recommended)
For a smoother development experience, you can use the helper Mojo script (`ksdev.mojo`). This automates the "wash then load" cycle.

1.  **Prep**: Place your `.rs` file in the `.wash/` directory.
2.  **Inspect**: Run `mojo ksdev.mojo --check-dish .wash/advanced_dish.rs` to verify your metadata is correct effectively preventing build errors.
3.  **Wash**: Run `mojo ksdev.mojo --wash`. This compiles your code and moves the artifact to the `.load/` directory.
4.  **Load**: Run `mojo ksdev.mojo --load`. This installs the dish from the `.load/` directory into your system.

> **Tip**: This workflow keeps your project root clean and ensures a consistent build process!

---

That's it! You have successfully created and deployed a custom dish.
