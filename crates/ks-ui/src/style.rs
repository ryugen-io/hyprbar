use k_lib::config::Cookbook;
use k_lib::factory::ColorResolver;
use ratatui::style::Color;

pub trait ThemeExt {
    fn resolve_color(&self, key: &str) -> Color;
    fn resolve_bg(&self, key: &str) -> Color;
}

impl ThemeExt for Cookbook {
    fn resolve_color(&self, key: &str) -> Color {
        self.theme
            .colors
            .get(key)
            .map(|s| {
                let c = ColorResolver::hex_to_color(s);
                Color::Rgb(c.r, c.g, c.b)
            })
            .unwrap_or(Color::Reset)
    }

    fn resolve_bg(&self, key: &str) -> Color {
        // Backgrounds might have transparency or special handling in the future
        self.resolve_color(key)
    }
}
