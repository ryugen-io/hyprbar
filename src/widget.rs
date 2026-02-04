use crate::state::BarState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// A Widget is a renderable component for the hyprbar bar.
pub trait Widget: Send + Sync {
    /// The unique name/type of the widget.
    fn name(&self) -> &str;

    /// Render the widget into the specified area.
    /// The widget should only draw within the bounds of `area`.
    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, dt: std::time::Duration);

    /// Update the widget state. Called every frame.
    fn update(&mut self, _dt: std::time::Duration, _state: &BarState) {}

    /// Return the required width of the widget.
    fn width(&self, state: &BarState) -> u16;

    /// Set the instance configuration name (alias).
    /// Used when a widget is instantiated multiple times with different configs.
    /// The widget should use this name to look up its configuration in `state.config.widget`.
    #[allow(unused_variables)]
    fn set_instance_config(&mut self, name: String) {}

    /// Handle an input event.
    /// The renderer calls this when an event occurs within the widget's area.
    #[allow(unused_variables)]
    fn handle_event(&mut self, event: crate::event::WidgetEvent) {}
}

/// A provider that creates widgets by name.
pub trait WidgetProvider {
    fn create_widget(&self, name: &str) -> Option<Box<dyn Widget>>;
}
