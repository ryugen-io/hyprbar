//! Name: Tray Space
//! Version: 1.0.0
//! Author: kitchnsink contributors
//! Description: A dish that provides space for tray icons

use ks_lib::prelude::*;

/// TraySpace provides a reserved area for system tray icons
///
/// Configuration in sink.toml:
/// ```toml
/// [dish.tray_space]
/// icon_size = 2           # Width per icon (in chars)
/// max_icons = 5           # Maximum number of icons to display
/// separator = " "         # Separator between icons
/// show_placeholder = true # Show placeholder when no icons
/// ```
pub struct TraySpace {
    instance_name: Option<String>,
    /// Currently displayed icons (placeholder for now, can be populated from system tray later)
    icons: Vec<String>,
}

impl TraySpace {
    pub fn new() -> Self {
        Self {
            instance_name: None,
            icons: vec![],
        }
    }

    fn icon_size(&self, state: &BarState) -> u16 {
        let base_config = state.config.dish.get("tray_space").and_then(|v| v.as_table());

        // Try instance config first
        if let Some(alias) = &self.instance_name {
            if let Some(size) = base_config
                .and_then(|t| t.get(alias))
                .and_then(|v| v.get("icon_size"))
                .and_then(|v| v.as_integer())
            {
                return size as u16;
            }
        }

        // Fallback to base config
        base_config
            .and_then(|t| t.get("icon_size"))
            .and_then(|v| v.as_integer())
            .unwrap_or(2) as u16
    }

    fn max_icons(&self, state: &BarState) -> usize {
        let base_config = state.config.dish.get("tray_space").and_then(|v| v.as_table());

        if let Some(alias) = &self.instance_name {
            if let Some(count) = base_config
                .and_then(|t| t.get(alias))
                .and_then(|v| v.get("max_icons"))
                .and_then(|v| v.as_integer())
            {
                return count as usize;
            }
        }

        base_config
            .and_then(|t| t.get("max_icons"))
            .and_then(|v| v.as_integer())
            .unwrap_or(5) as usize
    }

    fn separator(&self, state: &BarState) -> String {
        let base_config = state.config.dish.get("tray_space").and_then(|v| v.as_table());

        if let Some(alias) = &self.instance_name {
            if let Some(sep) = base_config
                .and_then(|t| t.get(alias))
                .and_then(|v| v.get("separator"))
                .and_then(|v| v.as_str())
            {
                return sep.to_string();
            }
        }

        base_config
            .and_then(|t| t.get("separator"))
            .and_then(|v| v.as_str())
            .unwrap_or(" ")
            .to_string()
    }

    fn show_placeholder(&self, state: &BarState) -> bool {
        let base_config = state.config.dish.get("tray_space").and_then(|v| v.as_table());

        if let Some(alias) = &self.instance_name {
            if let Some(show) = base_config
                .and_then(|t| t.get(alias))
                .and_then(|v| v.get("show_placeholder"))
                .and_then(|v| v.as_bool())
            {
                return show;
            }
        }

        base_config
            .and_then(|t| t.get("show_placeholder"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    }
}

impl Dish for TraySpace {
    fn name(&self) -> &str {
        "tray_space"
    }

    fn set_instance_config(&mut self, name: String) {
        self.instance_name = Some(name);
    }

    fn width(&self, state: &BarState) -> u16 {
        let icon_size = self.icon_size(state);
        let max_icons = self.max_icons(state);
        let separator_len = self.separator(state).chars().count() as u16;

        // Calculate total width: (icon_size * max_icons) + (separator * (max_icons - 1))
        let base_width = icon_size * (max_icons as u16);
        let separator_width = if max_icons > 1 {
            separator_len * (max_icons as u16 - 1)
        } else {
            0
        };

        base_width + separator_width
    }

    fn update(&mut self, _dt: std::time::Duration, _state: &BarState) {
        // TODO: In the future, this could poll system tray state
        // For now, we keep it as a static placeholder
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, _dt: Duration) {
        let icon_size = self.icon_size(state) as usize;
        let max_icons = self.max_icons(state);
        let separator = self.separator(state);
        let show_placeholder = self.show_placeholder(state);

        // Get foreground and accent colors from theme
        let _fg = ColorResolver::hex_to_color(&state.config.style.fg);
        let _accent = state.config.style.accent
            .as_ref()
            .map(|hex| ColorResolver::hex_to_color(hex))
            .unwrap_or(_fg);

        // If we have no icons and placeholder is disabled, render nothing
        if self.icons.is_empty() && !show_placeholder {
            return;
        }

        // Build the display string
        let mut display = String::new();

        if self.icons.is_empty() {
            // Show placeholder dots if enabled
            if show_placeholder {
                // Use icon from cookbook if available
                let placeholder_icon = if state.cookbook.theme.settings.active_icons == "nerdfont" {
                    state.cookbook.icons.nerdfont
                        .get("circle_outline")
                        .map(|s| s.as_str())
                        .unwrap_or("○")
                } else {
                    state.cookbook.icons.ascii
                        .get("circle_outline")
                        .map(|s| s.as_str())
                        .unwrap_or("o")
                };

                for i in 0..max_icons {
                    if i > 0 {
                        display.push_str(&separator);
                    }
                    // Pad to icon_size
                    let padded = format!("{:^width$}", placeholder_icon, width = icon_size);
                    display.push_str(&padded);
                }
            }
        } else {
            // Render actual icons
            for (i, icon) in self.icons.iter().take(max_icons).enumerate() {
                if i > 0 {
                    display.push_str(&separator);
                }
                let padded = format!("{:^width$}", icon, width = icon_size);
                display.push_str(&padded);
            }
        }

        // Render using Label for consistent styling
        Label::new(&display)
            .variant(TypographyVariant::Body)
            .render(area, buf, state.cookbook.as_ref());
    }

    fn handle_event(&mut self, _event: DishEvent) {
        // TODO: Handle clicks on individual icons
        // For now, just log that an event was received (in debug builds)
        #[cfg(debug_assertions)]
        {
            if _event.is_click_in((0, 0), (100, 100)) {
                // Click detected - future: determine which icon and trigger action
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_dish() -> Box<dyn Dish> {
    Box::new(TraySpace::new())
}
