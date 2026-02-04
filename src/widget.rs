use crate::state::BarState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy)]
pub struct PopupRequest {
    pub width: u16,
    pub height: u16,
    pub offset_x: i16,
    pub offset_y: i16,
}

impl PopupRequest {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            offset_x: 0,
            offset_y: 4,
        }
    }
}

/// A Widget is a renderable component for the hyprbar bar.
pub trait Widget: Send + Sync {
    fn name(&self) -> &str;
    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, dt: std::time::Duration);
    fn update(&mut self, _dt: std::time::Duration, _state: &BarState) {}
    fn width(&self, state: &BarState) -> u16;
    fn set_instance_config(&mut self, _name: String) {}
    fn handle_event(&mut self, _event: crate::event::WidgetEvent) {}
    fn popup_request(&self) -> Option<PopupRequest> {
        None
    }
    fn render_popup(&mut self, _area: Rect, _buf: &mut Buffer, _state: &BarState) {}
}

/// A provider that creates widgets by name.
pub trait WidgetProvider {
    fn create_widget(&self, name: &str) -> Option<Box<dyn Widget>>;
}
