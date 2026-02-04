use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BarConfig {
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub style: StyleConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub widget: std::collections::HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_debug_filter")]
    pub debug_filter: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            debug_filter: default_debug_filter(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_debug_filter() -> String {
    "info,hyprbar=debug".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayoutConfig {
    #[serde(default)]
    pub modules_left: Vec<String>,
    #[serde(default)]
    pub modules_center: Vec<String>,
    #[serde(default)]
    pub modules_right: Vec<String>,

    #[serde(default = "default_layout_percent")]
    pub left: u8,
    #[serde(default = "default_layout_percent")]
    pub center: u8,
    #[serde(default = "default_layout_percent")]
    pub right: u8,

    #[serde(default = "default_layout_strategy")]
    pub strategy: String,
    #[serde(default = "default_layout_padding")]
    pub padding: u16,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            left: 33,
            center: 33,
            right: 33,
            strategy: default_layout_strategy(),
            padding: default_layout_padding(),
            modules_left: vec![],
            modules_center: vec![],
            modules_right: vec![],
        }
    }
}

fn default_layout_percent() -> u8 {
    33
}

fn default_layout_strategy() -> String {
    "grid".to_string()
}

fn default_layout_padding() -> u16 {
    1
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowConfig {
    pub height: u32,
    pub anchor: String, // "top", "bottom"
    #[serde(default = "default_monitor")]
    pub monitor: String,

    // Smart Scaling Options
    #[serde(default = "default_scale_font")]
    pub scale_font: bool,
    #[serde(default)]
    pub pixel_font: bool,
    #[serde(default = "default_font_base_size")]
    pub font_base_size: u32,
    #[serde(default = "default_min_padding")]
    pub min_padding: u32,
    #[serde(default)]
    pub height_rows: Option<u32>,
}

impl WindowConfig {
    /// Calculates (font_size, window_height) based on configuration.
    /// Returns:
    /// - font_size: The calculated font size in pixels
    /// - window_height: The final window height in pixels
    pub fn calculate_dimensions(&self) -> (f32, u32) {
        if let Some(rows) = self.height_rows {
            // Row-based sizing: Height follows Font
            let fs = 16.0; // Default base size for row-mode
            let lh = fs * 1.2;
            let h = (lh * rows as f32).ceil() as u32 + self.min_padding;
            (fs, h)
        } else {
            // Pixel-based sizing: Font follows Height (Smart Scaling)
            let target_h = self.height;
            if self.scale_font {
                let available_h = target_h.saturating_sub(self.min_padding) as f32;
                let raw_fs = available_h / 1.2; // Derived from line_height = fs * 1.2
                let fs = if self.pixel_font {
                    let base = self.font_base_size as f32;
                    let scale = (raw_fs / base).floor().max(1.0);
                    base * scale
                } else {
                    raw_fs
                };
                (fs, target_h)
            } else {
                (16.0, target_h)
            }
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            height: 30,
            anchor: "top".to_string(),
            monitor: String::new(), // Empty = compositor decides
            scale_font: true,
            pixel_font: false,
            font_base_size: 10,
            min_padding: 2,
            height_rows: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_based_sizing() {
        let mut config = WindowConfig::default();
        config.height_rows = Some(1);
        config.min_padding = 2;

        let (fs, h) = config.calculate_dimensions();
        assert_eq!(fs, 16.0);
        // 16.0 * 1.2 = 19.2 -> ceil 20 + 2 = 22
        assert_eq!(h, 22);

        config.height_rows = Some(2);
        // (19.2 * 2) = 38.4 -> ceil 39 + 2 = 41
        let (_, h) = config.calculate_dimensions();
        assert_eq!(h, 41);
    }

    #[test]
    fn test_smart_scaling_standard() {
        let mut config = WindowConfig::default();
        config.height_rows = None;
        config.height = 30;
        config.min_padding = 6; // 24px available
        config.scale_font = true;
        config.pixel_font = false;

        let (fs, h) = config.calculate_dimensions();
        assert_eq!(h, 30);
        // 24 / 1.2 = 20.0
        assert!((fs - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_smart_scaling_pixel_font() {
        let mut config = WindowConfig::default();
        config.height = 30;
        config.min_padding = 2; // 28px available -> 28/1.2 = 23.33
        config.scale_font = true;
        config.pixel_font = true;
        config.font_base_size = 10;

        let (fs, _) = config.calculate_dimensions();
        // 23.33 / 10 = 2.33 -> floor 2 -> 2 * 10 = 20.0
        assert_eq!(fs, 20.0);
    }

    #[test]
    fn test_no_scaling() {
        let mut config = WindowConfig::default();
        config.height = 50;
        config.scale_font = false;

        let (fs, h) = config.calculate_dimensions();
        assert_eq!(h, 50);
        assert_eq!(fs, 16.0);
    }
}

fn default_monitor() -> String {
    String::new() // Empty = compositor decides
}

fn default_scale_font() -> bool {
    true
}

fn default_font_base_size() -> u32 {
    10
}

fn default_min_padding() -> u32 {
    2
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StyleConfig {
    #[serde(default)]
    pub window_bg: Option<String>,
    #[serde(default = "default_bg")]
    pub bg: String,
    #[serde(default = "default_fg")]
    pub fg: String,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub primary: Option<String>,
    #[serde(default)]
    pub secondary: Option<String>,
    #[serde(default)]
    pub success: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    pub font: Option<String>,
    #[serde(default)]
    pub animation: Option<AnimationConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AnimationConfig {
    pub entry: Option<String>, // e.g. "slide_up", "fade"
    pub exit: Option<String>,
    pub duration: Option<u64>, // ms
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            window_bg: None,
            bg: default_bg(),
            fg: default_fg(),
            accent: None,
            primary: None,
            secondary: None,
            success: None,
            error: None,
            font: None,
            animation: None,
        }
    }
}

fn default_bg() -> String {
    "#000000".to_string()
}

fn default_fg() -> String {
    "#FFFFFF".to_string()
}
