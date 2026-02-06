pub mod bus;
pub mod config;
pub mod event;
pub mod plugin_loader;
pub mod renderer;
pub mod state;
pub mod widget;

pub mod cli;
pub mod modules;
pub mod tui;
pub mod ui;
pub mod wayland;

pub mod prelude {
    pub use crate::config::BarConfig;
    pub use crate::event::WidgetEvent;
    pub use crate::state::BarState;
    pub use crate::ui::container::{Container, ContainerVariant};
    pub use crate::ui::interaction::InteractionExt;
    pub use crate::ui::label::{Label, TypographyVariant};
    pub use crate::ui::style::ThemeExt;
    pub use crate::widget::{PopupRequest, Widget, WidgetProvider};
    pub use hyprink::config::Config;
    pub use hyprink::factory::ColorResolver;
    pub use ratatui::prelude::*;
    pub use std::time::Duration;
}
