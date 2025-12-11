use ks_core::prelude::*;
use std::borrow::Cow;

pub struct TextArea {
    content: String,
}

impl TextArea {
    pub fn new() -> Self {
        // Since we can't pass args to new(), we set a default.
        // In render(), we check the config for override.
        Self {
            content: "Kitchn Sink".to_string(),
        }
    }
}

impl Dish for TextArea {
    fn name(&self) -> &str {
        "text_area"
    }

    fn width(&self, state: &BarState) -> u16 {
        // Re-read content from state config to ensure dynamic updates
        let content = state
            .config
            .dish
            .get("text_area")
            .and_then(|v| v.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("Kitchn Sink");

        content.chars().count() as u16
    }

    fn update(&mut self, _dt: std::time::Duration) {}

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, _dt: Duration) {
        let content = state
            .config
            .dish
            .get("text_area")
            .and_then(|v| v.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("Kitchn Sink");

        // Assuming simple default style or config style
        ratatui::widgets::Paragraph::new(Cow::from(content)).render(area, buf);
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_dish() -> Box<dyn Dish> {
    Box::new(TextArea::new())
}
