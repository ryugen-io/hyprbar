//! Name: Battery Widget
//! Version: 1.2.0
//! Author: Ryu
//! Description: Shows battery status with configurable colors

use ks_core::prelude::*;

pub struct BatteryDish {
    percent: u8,
    charging: bool,
    last_update: Duration,
    battery_path: Option<std::path::PathBuf>,
}

impl BatteryDish {
    pub fn new() -> Self {
        let battery_path = Self::find_battery();
        let (percent, charging) = Self::read_battery(&battery_path);
        Self {
            percent,
            charging,
            last_update: Duration::ZERO,
            battery_path,
        }
    }

    fn find_battery() -> Option<std::path::PathBuf> {
        let power_supply = std::path::Path::new("/sys/class/power_supply");
        if let Ok(entries) = std::fs::read_dir(power_supply) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("BAT") {
                    return Some(entry.path());
                }
            }
        }
        None
    }

    fn read_battery(path: &Option<std::path::PathBuf>) -> (u8, bool) {
        let Some(bat_path) = path else {
            return (0, false);
        };

        let percent = std::fs::read_to_string(bat_path.join("capacity"))
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok())
            .unwrap_or(0);

        let status = std::fs::read_to_string(bat_path.join("status")).unwrap_or_default();
        let charging = matches!(status.trim(), "Charging" | "Full");

        (percent, charging)
    }
}

impl Dish for BatteryDish {
    fn name(&self) -> &str {
        "Battery"
    }

    fn width(&self, _state: &BarState) -> u16 {
        18
    }

    fn update(&mut self, dt: Duration) {
        self.last_update += dt;
        if self.last_update.as_secs() >= 30 {
            let (percent, charging) = Self::read_battery(&self.battery_path);
            self.percent = percent;
            self.charging = charging;
            self.last_update = Duration::ZERO;
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, _dt: Duration) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let _ = state.cookbook.help_me_find_icons;

        // All colors from config - NO hardcoded values
        let fg_color = Some(state.config.style.fg.as_str()).map(|s| {
            let c = ColorResolver::hex_to_color(s);
            Color::Rgb(c.r, c.g, c.b)
        });

        let bg_color = Some(state.config.style.bg.as_str()).map(|s| {
            let c = ColorResolver::hex_to_color(s);
            Color::Rgb(c.r, c.g, c.b)
        });

        // Check for config overrides in [dish.battery]
        let battery_config = state.config.dish.get("battery").and_then(|v| v.as_table());

        let resolve_override = |key: &str, fallback: Option<Color>| -> Option<Color> {
            battery_config
                .and_then(|t| t.get(key))
                .and_then(|v| v.as_str())
                .map(|s| {
                    let c = ColorResolver::hex_to_color(s);
                    Color::Rgb(c.r, c.g, c.b)
                })
                .or(fallback)
        };

        let success_color = resolve_override(
            "color_high",
            state.config.style.success.as_deref().map(|s| {
                let c = ColorResolver::hex_to_color(s);
                Color::Rgb(c.r, c.g, c.b)
            }),
        );

        let warning_color = resolve_override(
            "color_medium",
            state.config.style.secondary.as_deref().map(|s| {
                let c = ColorResolver::hex_to_color(s);
                Color::Rgb(c.r, c.g, c.b)
            }),
        );

        let error_color = resolve_override(
            "color_low",
            state.config.style.error.as_deref().map(|s| {
                let c = ColorResolver::hex_to_color(s);
                Color::Rgb(c.r, c.g, c.b)
            }),
        );

        let accent_color = state.config.style.accent.as_deref().map(|s| {
            let c = ColorResolver::hex_to_color(s);
            Color::Rgb(c.r, c.g, c.b)
        });

        // Choose bar color based on battery level - fallback chain through config colors
        let bar_color = if self.charging {
            success_color.or(fg_color)
        } else if self.percent > 50 {
            success_color.or(fg_color)
        } else if self.percent > 20 {
            warning_color.or(fg_color)
        } else {
            error_color.or(fg_color)
        };

        // Empty bar uses accent, falls back to bg
        let empty_color = accent_color.or(bg_color);

        // Build btop-style display: "⚡ ████████░░ 85%"
        let icon = if self.charging { "⚡" } else { "" };

        // Create bar: 8 chars wide
        let bar_width = 8;
        let filled = (self.percent as usize * bar_width) / 100;
        let empty = bar_width - filled;

        let bar_filled: String = "█".repeat(filled);
        let bar_empty: String = "░".repeat(empty);
        let percent_str = format!("{:>3}%", self.percent);

        // Render character by character with config colors
        let mut x = area.x;
        let y = area.y;

        // Icon
        for ch in icon.chars() {
            if x >= area.right() {
                break;
            }
            let cell = &mut buf[(x, y)];
            cell.set_char(ch);
            if let Some(c) = fg_color {
                cell.set_fg(c);
            }
            x += 1;
        }

        // Space after icon
        if x < area.right() && !icon.is_empty() {
            let cell = &mut buf[(x, y)];
            cell.set_char(' ');
            if let Some(c) = fg_color {
                cell.set_fg(c);
            }
            x += 1;
        }

        // Filled bar
        for ch in bar_filled.chars() {
            if x >= area.right() {
                break;
            }
            let cell = &mut buf[(x, y)];
            cell.set_char(ch);
            if let Some(c) = bar_color {
                cell.set_fg(c);
            }
            x += 1;
        }

        // Empty bar
        for ch in bar_empty.chars() {
            if x >= area.right() {
                break;
            }
            let cell = &mut buf[(x, y)];
            cell.set_char(ch);
            if let Some(c) = empty_color {
                cell.set_fg(c);
            }
            x += 1;
        }

        // Space before percent
        if x < area.right() {
            let cell = &mut buf[(x, y)];
            cell.set_char(' ');
            if let Some(c) = fg_color {
                cell.set_fg(c);
            }
            x += 1;
        }

        // Percent
        for ch in percent_str.chars() {
            if x >= area.right() {
                break;
            }
            let cell = &mut buf[(x, y)];
            cell.set_char(ch);
            if let Some(c) = fg_color {
                cell.set_fg(c);
            }
            x += 1;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_dish() -> Box<dyn Dish> {
    Box::new(BatteryDish::new())
}
