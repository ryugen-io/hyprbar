use ratatui::prelude::Rect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WidgetSection {
    Left,
    Center,
    Right,
}

pub(crate) struct HitWidget {
    pub area: Rect,
    pub section: WidgetSection,
    pub index: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct ActivePopup {
    pub section: WidgetSection,
    pub index: usize,
    pub widget_area: Rect,
}
