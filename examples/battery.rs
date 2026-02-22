//! Name: Battery Widget
//! Version: 1.2.0
//! Author: Ryu
//! Description: Shows battery status with configurable colors

use hyprbar::prelude::*;
use std::sync::Mutex;
use tachyonfx::{Effect, Interpolation, Motion, fx, pattern::SweepPattern};

pub struct BatteryWidget {
    percent: u8,
    charging: bool,
    last_update: Duration,
    battery_path: Option<std::path::PathBuf>,
    instance_name: Option<String>,
    // Widget trait requires Sync, but Effect is not Sync on its own.
    effect: Mutex<Option<Effect>>,
    last_state: BatteryState,
}

#[derive(PartialEq, Clone, Copy)]
enum BatteryState {
    Normal,
    Low,
    Charging,
}

impl BatteryWidget {
    pub fn new() -> Self {
        let battery_path = Self::find_battery();
        let (percent, charging) = Self::read_battery(&battery_path);

        // Must match update_effect's classification so the first frame doesn't trigger a spurious state change.
        let initial_state = if charging {
            BatteryState::Charging
        } else if percent <= 20 {
            BatteryState::Low
        } else {
            BatteryState::Normal
        };

        let mut dish = Self {
            percent,
            charging,
            last_update: Duration::ZERO,
            battery_path,
            instance_name: None,
            effect: Mutex::new(None),
            last_state: initial_state,
        };

        // Defer effect creation to first update() so we have access to Cookbook
        // dish.update_effect();
        dish
    }

    fn update_effect(&mut self, state: &BarState) {
        let current_state = if self.charging {
            BatteryState::Charging
        } else if self.percent <= 20 {
            BatteryState::Low
        } else {
            BatteryState::Normal
        };

        let effect_is_none = self.effect.lock().unwrap().is_none();
        if current_state != self.last_state || effect_is_none {
            self.last_state = current_state;

            // Config overrides take priority over theme defaults so per-bar sink.toml is respected.
            let accent = if let Some(hex) = &state.config.style.accent {
                let c = ColorResolver::hex_to_color(hex);
                Color::Rgb(c.r, c.g, c.b)
            } else {
                state.config_ink.resolve_color("accent")
            };

            let low_color = state.config_ink.resolve_color("error");

            *self.effect.lock().unwrap() = match current_state {
                BatteryState::Charging => {
                    // FG-only fade prevents background color from bleeding into adjacent cells.
                    Some(fx::fade_to_fg(accent, 1500).with_pattern(SweepPattern::left_to_right(4)))
                }
                BatteryState::Low => {
                    // Pulsing draws the user's eye without being as jarring as a static alert.
                    Some(fx::ping_pong(fx::fade_from(
                        low_color,
                        Color::Reset,
                        (750, Interpolation::SineInOut),
                    )))
                }
                BatteryState::Normal => None,
            };
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

impl Widget for BatteryWidget {
    fn name(&self) -> &str {
        "battery"
    }

    fn set_instance_config(&mut self, name: String) {
        self.instance_name = Some(name);
    }

    // Increase width to accomodate effects better
    fn width(&self, _state: &BarState) -> u16 {
        18
    }

    fn update(&mut self, dt: Duration, _state: &BarState) {
        self.last_update += dt;
        if self.last_update > Duration::from_secs(5) {
            let (percent, charging) = Self::read_battery(&self.battery_path);
            self.percent = percent;
            self.charging = charging;
            self.last_update = Duration::from_secs(0);

            // Battery state may have changed — update effect to match new charge/charging level.
            self.update_effect(_state);
        } else {
            // Constructor can't create effects (no BarState yet), so first update must initialize.
            if self.effect.lock().unwrap().is_none() && self.charging {
                self.update_effect(_state);
            }
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, dt: Duration) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Colors and config must be resolved before any drawing so themed overrides apply everywhere.
        let fg_color = Some(state.config_ink.resolve_color("fg"));
        let bg_color = Some(state.config_ink.resolve_bg("bg"));
        let accent_color = Some(state.config_ink.resolve_color("accent"));

        let base_config = state
            .config
            .widget
            .get("battery")
            .and_then(|v| v.as_table());
        let instance_config = if let Some(alias) = &self.instance_name {
            base_config
                .and_then(|t| t.get(alias))
                .and_then(|v| v.as_table())
        } else {
            None
        };

        let resolve_override = |key: &str, fallback: Option<Color>| -> Option<Color> {
            instance_config
                .and_then(|t| t.get(key))
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
            Some(state.config_ink.resolve_color("success")),
        );

        let warning_color = resolve_override(
            "color_medium",
            Some(state.config_ink.resolve_color("secondary")),
        );

        let error_color =
            resolve_override("color_low", Some(state.config_ink.resolve_color("error")));

        let bar_color = if self.charging {
            success_color.or(fg_color)
        } else if self.percent > 50 {
            success_color.or(fg_color)
        } else if self.percent > 20 {
            warning_color.or(fg_color)
        } else {
            error_color.or(fg_color)
        };

        let empty_color = accent_color.or(bg_color);

        let icon = if self.charging { "⚡" } else { "" };

        let bar_width = 8;
        let filled = (self.percent as usize * bar_width) / 100;
        let empty = bar_width - filled;

        let bar_filled: String = "█".repeat(filled);
        let bar_empty: String = "░".repeat(empty);
        let percent_str = format!("{:>3}%", self.percent);

        let mut x = area.x;
        let y = area.y;

        // Manual cell-by-cell writing because ratatui's Paragraph widget doesn't support mixed per-char colors.
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

        if x < area.right() && !icon.is_empty() {
            let cell = &mut buf[(x, y)];
            cell.set_char(' ');
            if let Some(c) = fg_color {
                cell.set_fg(c);
            }
            x += 1;
        }

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

        if x < area.right() {
            let cell = &mut buf[(x, y)];
            cell.set_char(' ');
            if let Some(c) = fg_color {
                cell.set_fg(c);
            }
            x += 1;
        }

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

        // Effects are applied post-render so they composite on top of the already-drawn content.
        let mut effect_lock = self.effect.lock().unwrap();
        if let Some(effect) = effect_lock.as_mut() {
            // Effect must only cover the filled bar blocks — animating the icon or percentage looks wrong.
            let icon_offset = if self.charging { 2 } else { 0 };
            let visible_width = if filled > 0 { filled } else { 1 };

            let bar_area = Rect::new(area.x + icon_offset, area.y, visible_width as u16, 1);

            // Bounds check prevents painting outside our allocated area into adjacent widgets.
            if bar_area.right() <= area.right() && filled > 0 {
                effect.process(dt, buf, bar_area);
            }

            // tachyonfx effects are one-shot — restart a fresh instance to keep the animation looping.
            if effect.done() {
                let current_state = self.last_state;
                // Colors may have changed (e.g. theme hot-reload) since the previous loop iteration.
                let accent = if let Some(hex) = &state.config.style.accent {
                    let c = ColorResolver::hex_to_color(hex);
                    Color::Rgb(c.r, c.g, c.b)
                } else {
                    state.config_ink.resolve_color("accent")
                };
                let low_color = state.config_ink.resolve_color("error");

                *effect_lock = match current_state {
                    BatteryState::Charging => Some(
                        fx::fade_to_fg(accent, 1500).with_pattern(SweepPattern::left_to_right(4)),
                    ),
                    BatteryState::Low => Some(fx::ping_pong(fx::fade_from(
                        low_color,
                        Color::Reset,
                        (750, Interpolation::SineInOut),
                    ))),
                    BatteryState::Normal => None,
                };
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_widget() -> Box<dyn Widget> {
    Box::new(BatteryWidget::new())
}
