use crate::state::BarState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// A Dish is a renderable component (widget) for the kitchnsink bar.
pub trait Dish: Send + Sync {
    /// The unique name/type of the dish.
    fn name(&self) -> &str;

    /// Render the dish into the specified area.
    /// The dish should only draw within the bounds of `area`.
    fn render(&self, area: Rect, buf: &mut Buffer, state: &BarState);

    /// Update the dish state. called every frame.
    fn update(&mut self, _dt: std::time::Duration) {}

    /// Return the required width of the dish.
    fn width(&self, state: &BarState) -> u16;
}

/// A provider that creates dishes by name.
pub trait DishProvider {
    fn create_dish(&self, name: &str) -> Option<Box<dyn Dish>>;
}
