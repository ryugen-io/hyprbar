use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SinkConfig {
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub style: StyleConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowConfig {
    pub height: u32,
    pub anchor: String, // "top", "bottom"
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            height: 30,
            anchor: "top".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StyleConfig {
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
