use hyprbar::prelude::*;
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

impl Widget for Separator {
    fn name(&self) -> &str {
        "separator"
    }

    fn width(&self, state: &BarState) -> u16 {
        let symbol = state
            .config
            .widget
            .get("separator")
            .and_then(|v| v.get("symbol"))
            .and_then(|v| v.as_str())
            .unwrap_or("|");

        symbol.chars().count() as u16
    }

    fn update(&mut self, _dt: std::time::Duration, _state: &BarState) {}

    fn render(&mut self, area: Rect, buf: &mut Buffer, _state: &BarState, _dt: Duration) {
        use ratatui::widgets::Widget as RatatuiWidget;
        ratatui::widgets::Paragraph::new(Cow::from("|")).render(area, buf);
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_widget() -> Box<dyn Widget> {
    Box::new(Separator::new())
}

#[unsafe(no_mangle)]
pub extern "C" fn _plugin_metadata() -> *const std::ffi::c_char {
    static META: &[u8] =
        b"{\"author\":\"\",\"description\":\"\",\"name\":\"Unknown\",\"version\":\"0.0.1\"}\0";
    META.as_ptr() as *const _
}
