use ks_ui::interaction::InteractionExt;
use ratatui::layout::Rect;

/// Events that can be sent to a Dish.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DishEvent {
    /// Pointer entered the dish area
    Enter,
    /// Pointer left the dish area
    Leave,
    /// Pointer moved within the dish area (relative coordinates)
    Motion { x: u16, y: u16 },
    /// Pointer button clicked (1=Left, 2=Middle, 3=Right, etc.)
    Click { button: u32, x: u16, y: u16 },
    /// Scroll event (dx, dy)
    Scroll { dx: f64, dy: f64 },
}

impl InteractionExt for DishEvent {
    fn is_click_in(&self, area: Rect) -> bool {
        match self {
            DishEvent::Click { x, y, .. } => area.contains((*x, *y).into()),
            _ => false,
        }
    }

    fn is_hover_in(&self, area: Rect) -> bool {
        match self {
            DishEvent::Motion { x, y } => area.contains((*x, *y).into()),
            _ => false,
        }
    }

    fn relative_pos(&self, area: Rect) -> Option<(u16, u16)> {
        match self {
            DishEvent::Click { x, y, .. } | DishEvent::Motion { x, y } => {
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
