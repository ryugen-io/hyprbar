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
