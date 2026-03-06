use crate::config::BarConfig;
use crate::theme::ColorResolver;
use ratatui::style::Color;

pub trait ThemeExt {
    fn resolve_color(&self, key: &str) -> Color;
    fn resolve_bg(&self, key: &str) -> Color;
}

impl ThemeExt for BarConfig {
    fn resolve_color(&self, key: &str) -> Color {
        self.color_hex(key)
            .map(ColorResolver::hex_to_ratatui)
            .unwrap_or(Color::Reset)
    }

    fn resolve_bg(&self, key: &str) -> Color {
        // Backgrounds might have transparency or special handling in the future
        self.resolve_color(key)
    }
}
