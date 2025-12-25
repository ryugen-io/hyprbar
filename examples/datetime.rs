//! Name: DateTime
//! Version: 1.0.0
//! Author: Ryu
//! Description: Configurable date and time display with weekday, week number, and timezone support
//! Dependency: chrono = "0.4"

use chrono::Local;
use ks_lib::prelude::*;

/// Configuration keys for [dish.datetime] in sink.toml
struct DateTimeConfig {
    // Date options
    date_format: String,
    show_date: bool,
    show_weekday: bool,
    weekday_format: String,
    show_week: bool,
    // Time options
    time_format: String,
    show_time: bool,
    show_seconds: bool,
    show_timezone: bool,
    timezone_format: String,
    // Layout
    separator: String,
}

impl Default for DateTimeConfig {
    fn default() -> Self {
        Self {
            date_format: "YYYY-MM-DD".to_string(),
            show_date: true,
            show_weekday: false,
            weekday_format: "short".to_string(),
            show_week: false,
            time_format: "24h".to_string(),
            show_time: true,
            show_seconds: false,
            show_timezone: false,
            timezone_format: "offset".to_string(),
            separator: " ".to_string(),
        }
    }
}

impl DateTimeConfig {
    fn from_state(state: &BarState) -> Self {
        let config = state.config.dish.get("datetime").and_then(|v| v.as_table());
        let mut cfg = Self::default();

        if let Some(t) = config {
            if let Some(v) = t.get("date_format").and_then(|v| v.as_str()) {
                cfg.date_format = v.to_string();
            }
            if let Some(v) = t.get("show_date").and_then(|v| v.as_bool()) {
                cfg.show_date = v;
            }
            if let Some(v) = t.get("show_weekday").and_then(|v| v.as_bool()) {
                cfg.show_weekday = v;
            }
            if let Some(v) = t.get("weekday_format").and_then(|v| v.as_str()) {
                cfg.weekday_format = v.to_string();
            }
            if let Some(v) = t.get("show_week").and_then(|v| v.as_bool()) {
                cfg.show_week = v;
            }
            if let Some(v) = t.get("time_format").and_then(|v| v.as_str()) {
                cfg.time_format = v.to_string();
            }
            if let Some(v) = t.get("show_time").and_then(|v| v.as_bool()) {
                cfg.show_time = v;
            }
            if let Some(v) = t.get("show_seconds").and_then(|v| v.as_bool()) {
                cfg.show_seconds = v;
            }
            if let Some(v) = t.get("show_timezone").and_then(|v| v.as_bool()) {
                cfg.show_timezone = v;
            }
            if let Some(v) = t.get("timezone_format").and_then(|v| v.as_str()) {
                cfg.timezone_format = v.to_string();
            }
            if let Some(v) = t.get("separator").and_then(|v| v.as_str()) {
                cfg.separator = v.to_string();
            }
        }
        cfg
    }

    /// Build the chrono format string based on config
    fn build_format_string(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Weekday
        if self.show_weekday {
            let weekday_fmt = match self.weekday_format.as_str() {
                "long" => "%A",
                _ => "%a", // short is default
            };
            parts.push(weekday_fmt.to_string());
        }

        // Date
        if self.show_date {
            let date_fmt = match self.date_format.as_str() {
                "DD/MM/YYYY" => "%d/%m/%Y",
                "MM/DD/YYYY" => "%m/%d/%Y",
                "DD.MM.YYYY" => "%d.%m.%Y",
                _ => "%Y-%m-%d", // YYYY-MM-DD is default
            };
            parts.push(date_fmt.to_string());
        }

        // Week number
        if self.show_week {
            parts.push("KW %V".to_string());
        }

        // Time
        if self.show_time {
            let time_fmt = match (self.time_format.as_str(), self.show_seconds) {
                ("12h", true) => "%I:%M:%S %p",
                ("12h", false) => "%I:%M %p",
                (_, true) => "%H:%M:%S", // 24h with seconds
                (_, false) => "%H:%M",   // 24h without seconds (default)
            };
            parts.push(time_fmt.to_string());
        }

        // Timezone
        if self.show_timezone {
            let tz_fmt = match self.timezone_format.as_str() {
                "name" => "%Z",
                _ => "%:z", // offset is default
            };
            parts.push(tz_fmt.to_string());
        }

        parts.join(&self.separator)
    }

    /// Calculate the expected width of the formatted string
    fn calculate_width(&self) -> u16 {
        let mut width: u16 = 0;
        let mut components: u16 = 0;

        if self.show_weekday {
            width += match self.weekday_format.as_str() {
                "long" => 9, // "Wednesday" is longest
                _ => 3,      // "Mon", "Tue", etc.
            };
            components += 1;
        }

        if self.show_date {
            width += 10; // All date formats are 10 chars
            components += 1;
        }

        if self.show_week {
            width += 5; // "KW 50"
            components += 1;
        }

        if self.show_time {
            width += match (self.time_format.as_str(), self.show_seconds) {
                ("12h", true) => 11, // "12:30:45 PM"
                ("12h", false) => 8, // "12:30 PM"
                (_, true) => 8,      // "14:30:45"
                (_, false) => 5,     // "14:30"
            };
            components += 1;
        }

        if self.show_timezone {
            width += match self.timezone_format.as_str() {
                "name" => 4, // "CEST" is longest common
                _ => 6,      // "+01:00"
            };
            components += 1;
        }

        // Add separator widths between components
        if components > 1 {
            width += (self.separator.chars().count() as u16) * (components - 1);
        }

        width.max(1) // Minimum width of 1
    }
}

pub struct DateTimeDish {
    cached_display: String,
    cached_width: u16,

    timer: Duration,
}

impl DateTimeDish {
    pub fn new() -> Self {
        Self {
            cached_display: String::new(),
            cached_width: 10,

            timer: Duration::from_secs(0),
        }
    }

    fn update_display(&mut self, config: &DateTimeConfig) {
        let now = Local::now();
        let format_str = config.build_format_string();
        self.cached_display = now.format(&format_str).to_string();
        self.cached_width = config.calculate_width();
    }
}

impl Dish for DateTimeDish {
    fn name(&self) -> &str {
        "DateTime"
    }

    fn width(&self, state: &BarState) -> u16 {
        let config = DateTimeConfig::from_state(state);
        config.calculate_width()
    }

    fn update(&mut self, dt: Duration, state: &BarState) {
        self.timer += dt;
        if self.timer.as_secs_f64() > 1.0 {
             let config = DateTimeConfig::from_state(state);
            self.update_display(&config);
            self.timer = Duration::from_secs(0);
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, _dt: Duration) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Read config and update display
        let config = DateTimeConfig::from_state(state);
        self.update_display(&config);

        // Check for config color override
        let _override_color_str = state
            .config
            .dish
            .get("datetime")
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("color"))
            .and_then(|v| v.as_str());

        // Render using Label
        Label::new(&self.cached_display)
             .variant(TypographyVariant::Body)
             .render(area, buf, state.cookbook.as_ref());
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_dish() -> Box<dyn Dish> {
    Box::new(DateTimeDish::new())
}
