use crate::state::BarState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// A Dish is a renderable component (widget) for the kitchnsink bar.
pub trait Dish: Send + Sync {
    /// The unique name/type of the dish.
    fn name(&self) -> &str;

    /// Render the dish into the specified area.
    /// The dish should only draw within the bounds of `area`.
    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, dt: std::time::Duration);

    /// Update the dish state. called every frame.
    fn update(&mut self, _dt: std::time::Duration) {}

    /// Return the required width of the dish.
    fn width(&self, state: &BarState) -> u16;

    /// Set the instance configuration name (alias).
    /// Used when a dish is instantiated multiple times with different configs (e.g. "TextArea#2").
    /// The dish should use this name to look up its configuration in `state.config.dish`.
    #[allow(unused_variables)]
    fn set_instance_config(&mut self, name: String) {}
}

/// A provider that creates dishes by name.
pub trait DishProvider {
    fn create_dish(&self, name: &str) -> Option<Box<dyn Dish>>;
}
