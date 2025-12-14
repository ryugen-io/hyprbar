use ks_lib::prelude::*;
use std::borrow::Cow;

pub struct Separator {
    _symbol: String,
}

impl Separator {
    pub fn new() -> Self {
        Self {
            _symbol: "|".to_string(),
        }
    }
}

impl Dish for Separator {
    fn name(&self) -> &str {
        "separator"
    }

    fn width(&self, state: &BarState) -> u16 {
        let symbol = state
            .config
            .dish
            .get("separator")
            .and_then(|v| v.get("symbol"))
            .and_then(|v| v.as_str())
            .unwrap_or("|");

        symbol.chars().count() as u16
    }

    fn update(&mut self, _dt: std::time::Duration) {}

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, _dt: Duration) {
        let symbol = state
            .config
            .dish
            .get("separator")
            .and_then(|v| v.get("symbol"))
            .and_then(|v| v.as_str())
            .unwrap_or("|");

        ratatui::widgets::Paragraph::new(Cow::from(symbol)).render(area, buf);
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_dish() -> Box<dyn Dish> {
    Box::new(Separator::new())
}
