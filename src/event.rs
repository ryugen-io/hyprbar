use crate::ui::interaction::InteractionExt;
use ratatui::layout::Rect;

/// Events that can be sent to a Widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WidgetEvent {
    /// Pointer entered the widget area
    Enter,
    /// Pointer left the widget area
    Leave,
    /// Pointer moved within the widget area (relative coordinates)
    Motion { x: u16, y: u16 },
    /// Pointer button clicked (1=Left, 2=Middle, 3=Right, etc.)
    Click { button: u32, x: u16, y: u16 },
    /// Scroll event (dx, dy)
    Scroll { dx: f64, dy: f64 },
}

impl InteractionExt for WidgetEvent {
    fn is_click_in(&self, area: Rect) -> bool {
        match self {
            WidgetEvent::Click { x, y, .. } => area.contains((*x, *y).into()),
            _ => false,
        }
    }

    fn is_hover_in(&self, area: Rect) -> bool {
        match self {
            WidgetEvent::Motion { x, y } => area.contains((*x, *y).into()),
            _ => false,
        }
    }

    fn relative_pos(&self, area: Rect) -> Option<(u16, u16)> {
        match self {
            WidgetEvent::Click { x, y, .. } | WidgetEvent::Motion { x, y } => {
                if area.contains((*x, *y).into()) {
                    Some((x - area.x, y - area.y))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
