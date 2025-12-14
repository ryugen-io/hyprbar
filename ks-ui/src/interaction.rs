use ratatui::layout::Rect;

pub trait InteractionExt {
    /// Checks if a Click event occurred within the given area.
    /// Returns true if it did.
    fn is_click_in(&self, area: Rect) -> bool;

    /// Checks if a Motion event occurred within the given area.
    /// Returns true if it did.
    fn is_hover_in(&self, area: Rect) -> bool;

    /// Returns the relative coordinates (x, y) if the event is a motion or click inside the area.
    /// Returns None regarding the specific event type or if outside.
    fn relative_pos(&self, area: Rect) -> Option<(u16, u16)>;
}
