
//! Name: Battery Widget
//! Version: 1.2.0
//! Author: Ryu
//! Description: Shows battery status with configurable colors

use ks_lib::prelude::*;

pub struct BatteryDish {
    percent: u8,
    charging: bool,
    last_update: Duration,
    battery_path: Option<std::path::PathBuf>,
    instance_name: Option<String>,
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
            instance_name: None,
        }
    }
    fn find_battery() -> Option<std::path::PathBuf> {
        let base = std::path::Path::new("/sys/class/power_supply");
        if let Ok(entries) = std::fs::read_dir(base) {
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
        if let Some(path) = path {
            let capacity = std::fs::read_to_string(path.join("capacity"))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
            
            let status = std::fs::read_to_string(path.join("status"))
                .ok()
                .map(|s| s.trim().to_uppercase())
                .unwrap_or_default();
                
            (capacity, status == "CHARGING")
        } else {
            (0, false)
        }
    }
}

impl Dish for BatteryDish {
    fn name(&self) -> &str {
        "battery"
    }

    fn set_instance_config(&mut self, name: String) {
        self.instance_name = Some(name);
    }

    fn width(&self, _state: &BarState) -> u16 {
        18
    }

    fn update(&mut self, dt: Duration, _state: &BarState) {
        self.last_update += dt; // Assuming last_update is intended to be time_accumulator
        if self.last_update > Duration::from_secs(5) { // Assuming 5 seconds is the new update interval
            let (percent, charging) = Self::read_battery(&self.battery_path);
            self.percent = percent;
            self.charging = charging;
            self.last_update = Duration::from_secs(0);
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, _dt: Duration) {
        if area.width == 0 || area.height == 0 {
            return;
        }



        let fg_color = Some(state.cookbook.resolve_color("fg"));
        let bg_color = Some(state.cookbook.resolve_bg("bg"));
        let accent_color = Some(state.cookbook.resolve_color("accent"));

        // 2. Dish Config overrides (manual table lookup still needed for instance overrides)
        // We can keep the existing override logic but simplify the Color parsing
        let base_config = state.config.dish.get("battery").and_then(|v| v.as_table());
        let instance_config = if let Some(alias) = &self.instance_name {
            base_config.and_then(|t| t.get(alias)).and_then(|v| v.as_table())
        } else {
            None
        };

        let resolve_override = |key: &str, fallback: Option<Color>| -> Option<Color> {
             instance_config.and_then(|t| t.get(key))
                .or_else(|| base_config.and_then(|t| t.get(key)))
                .and_then(|v| v.as_str())
                .map(|s| {
                    let c = ColorResolver::hex_to_color(s);
                    Color::Rgb(c.r, c.g, c.b)
                })
                .or(fallback)
        };

        let success_color = resolve_override(
            "color_high",
            Some(state.cookbook.resolve_color("success")),
        );

        let warning_color = resolve_override(
            "color_medium",
            Some(state.cookbook.resolve_color("secondary")), // Use secondary as warning/medium often
        );

        let error_color = resolve_override(
            "color_low",
             Some(state.cookbook.resolve_color("error")),
        );

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
