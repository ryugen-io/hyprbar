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

        let status = std::fs::read_to_string(bat_path.join("status"))
            .unwrap_or_default();
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

    fn render(&self, area: Rect, buf: &mut Buffer, state: &BarState) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // All colors from config - NO hardcoded values
        let fg_color = state.config.style.fg
            .parse::<Color>()
            .ok();

        let bg_color = state.config.style.bg
            .parse::<Color>()
            .ok();

        let success_color = state.config.style.success
            .as_deref()
            .and_then(|c| c.parse::<Color>().ok());

        let warning_color = state.config.style.secondary
            .as_deref()
            .and_then(|c| c.parse::<Color>().ok());

        let error_color = state.config.style.error
            .as_deref()
            .and_then(|c| c.parse::<Color>().ok());

        let accent_color = state.config.style.accent
            .as_deref()
            .and_then(|c| c.parse::<Color>().ok());

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
            if x >= area.right() { break; }
            let cell = &mut buf[(x, y)];
            cell.set_char(ch);
            if let Some(c) = fg_color { cell.set_fg(c); }
            x += 1;
        }

        // Space after icon
        if x < area.right() && !icon.is_empty() {
            let cell = &mut buf[(x, y)];
            cell.set_char(' ');
            if let Some(c) = fg_color { cell.set_fg(c); }
            x += 1;
        }

        // Filled bar
        for ch in bar_filled.chars() {
            if x >= area.right() { break; }
            let cell = &mut buf[(x, y)];
            cell.set_char(ch);
            if let Some(c) = bar_color { cell.set_fg(c); }
            x += 1;
        }

        // Empty bar
        for ch in bar_empty.chars() {
            if x >= area.right() { break; }
            let cell = &mut buf[(x, y)];
            cell.set_char(ch);
            if let Some(c) = empty_color { cell.set_fg(c); }
            x += 1;
        }

        // Space before percent
        if x < area.right() {
            let cell = &mut buf[(x, y)];
            cell.set_char(' ');
            if let Some(c) = fg_color { cell.set_fg(c); }
            x += 1;
        }

        // Percent
        for ch in percent_str.chars() {
            if x >= area.right() { break; }
            let cell = &mut buf[(x, y)];
            cell.set_char(ch);
            if let Some(c) = fg_color { cell.set_fg(c); }
            x += 1;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_dish() -> Box<dyn Dish> {
    Box::new(BatteryDish::new())
}
