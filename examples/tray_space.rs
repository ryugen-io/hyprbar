//! Name: Tray Space
//! Version: 1.0.0
//! Author: hyprbar contributors
//! Description: A widget that provides space for tray icons

use hyprbar::prelude::*;

/// TraySpace provides a reserved area for system tray icons
pub struct TraySpaceWidget {
    instance_name: Option<String>,
    icons: Vec<String>,
}

impl TraySpaceWidget {
    pub fn new() -> Self {
        Self {
            instance_name: None,
            icons: vec![],
        }
    }

    fn icon_size(&self, state: &BarState) -> u16 {
        let base_config = state
            .config
            .widget
            .get("tray_space")
            .and_then(|v| v.as_table());

        if let Some(alias) = &self.instance_name {
            if let Some(size) = base_config
                .and_then(|t| t.get(alias))
                .and_then(|v| v.get("icon_size"))
                .and_then(|v| v.as_integer())
            {
                return size as u16;
            }
        }

        base_config
            .and_then(|t| t.get("icon_size"))
            .and_then(|v| v.as_integer())
            .unwrap_or(2) as u16
    }

    fn max_icons(&self, state: &BarState) -> usize {
        let base_config = state
            .config
            .widget
            .get("tray_space")
            .and_then(|v| v.as_table());

        if let Some(alias) = &self.instance_name {
            if let Some(count) = base_config
                .and_then(|t| t.get(alias))
                .and_then(|v| v.get("max_icons"))
                .and_then(|v| v.as_integer())
            {
                return count as usize;
            }
        }

        base_config
            .and_then(|t| t.get("max_icons"))
            .and_then(|v| v.as_integer())
            .unwrap_or(5) as usize
    }

    fn separator(&self, state: &BarState) -> String {
        let base_config = state
            .config
            .widget
            .get("tray_space")
            .and_then(|v| v.as_table());

        if let Some(alias) = &self.instance_name {
            if let Some(sep) = base_config
                .and_then(|t| t.get(alias))
                .and_then(|v| v.get("separator"))
                .and_then(|v| v.as_str())
            {
                return sep.to_string();
            }
        }

        base_config
            .and_then(|t| t.get("separator"))
            .and_then(|v| v.as_str())
            .unwrap_or(" ")
            .to_string()
    }

    fn show_placeholder(&self, state: &BarState) -> bool {
        let base_config = state
            .config
            .widget
            .get("tray_space")
            .and_then(|v| v.as_table());

        if let Some(alias) = &self.instance_name {
            if let Some(show) = base_config
                .and_then(|t| t.get(alias))
                .and_then(|v| v.get("show_placeholder"))
                .and_then(|v| v.as_bool())
            {
                return show;
            }
        }

        base_config
            .and_then(|t| t.get("show_placeholder"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    }
}

impl Widget for TraySpaceWidget {
    fn name(&self) -> &str {
        "tray_space"
    }

    fn set_instance_config(&mut self, name: String) {
        self.instance_name = Some(name);
    }

    fn width(&self, state: &BarState) -> u16 {
        let icon_size = self.icon_size(state);
        let max_icons = self.max_icons(state);
        let separator_len = self.separator(state).chars().count() as u16;

        let base_width = icon_size * (max_icons as u16);
        let separator_width = if max_icons > 1 {
            separator_len * (max_icons as u16 - 1)
        } else {
            0
        };

        base_width + separator_width
    }

    fn update(&mut self, _dt: Duration, _state: &BarState) {}

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, _dt: Duration) {
        let icon_size = self.icon_size(state) as usize;
        let max_icons = self.max_icons(state);
        let separator = self.separator(state);
        let show_placeholder = self.show_placeholder(state);

        if self.icons.is_empty() && !show_placeholder {
            return;
        }

        let mut display = String::new();

        if self.icons.is_empty() {
            if show_placeholder {
                let placeholder_icon = "○";
                for i in 0..max_icons {
                    if i > 0 {
                        display.push_str(&separator);
                    }
                    let padded = format!("{:^width$}", placeholder_icon, width = icon_size);
                    display.push_str(&padded);
                }
            }
        } else {
            for (i, icon) in self.icons.iter().take(max_icons).enumerate() {
                if i > 0 {
                    display.push_str(&separator);
                }
                let padded = format!("{:^width$}", icon, width = icon_size);
                display.push_str(&padded);
            }
        }

        Label::new(&display)
            .variant(TypographyVariant::Body)
            .render(area, buf, state.config_ink.as_ref());
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_widget() -> Box<dyn Widget> {
    Box::new(TraySpaceWidget::new())
}
