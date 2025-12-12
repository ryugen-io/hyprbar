use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SinkConfig {
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub style: StyleConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub dish: std::collections::HashMap<String, toml::Value>,
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
    "info,ks_bin=debug,ks_core=debug,ks_wayland=debug".to_string()
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
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            left: 33,
            center: 33,
            right: 33,
            modules_left: vec![],
            modules_center: vec![],
            modules_right: vec![],
        }
    }
}

fn default_layout_percent() -> u8 {
    33
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

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            height: 30,
            anchor: "top".to_string(),
            monitor: "primary".to_string(),
            scale_font: true,
            pixel_font: false,
            font_base_size: 10,
            min_padding: 2,
            height_rows: None,
        }
    }
}

fn default_monitor() -> String {
    "primary".to_string()
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
        }
    }
}

fn default_bg() -> String {
    "#000000".to_string()
}

fn default_fg() -> String {
    "#FFFFFF".to_string()
}
