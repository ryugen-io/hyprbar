use ratatui::style::Color;

/// Simple RGB container used by rendering paths that expect explicit channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Shared color conversion helpers for hyprsbar.
pub struct ColorResolver;

impl ColorResolver {
    #[must_use]
    pub fn hex_to_color(value: &str) -> RgbColor {
        parse_hex(value).unwrap_or(RgbColor {
            r: 255,
            g: 255,
            b: 255,
        })
    }

    #[must_use]
    pub fn hex_to_ratatui(value: &str) -> Color {
        parse_hex(value)
            .map(|rgb| Color::Rgb(rgb.r, rgb.g, rgb.b))
            .unwrap_or(Color::Reset)
    }
}

#[must_use]
pub fn parse_hex(value: &str) -> Option<RgbColor> {
    let stripped = value.trim();
    let hex = stripped.strip_prefix('#').unwrap_or(stripped);
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(RgbColor { r, g, b })
}
