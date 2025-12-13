use crate::style::ThemeExt;
use ks_core::state::BarState;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Widget},
};

pub enum ContainerVariant {
    Base,
    Panel,
    Glass,
    Alert,
}

pub struct Container {
    variant: ContainerVariant,
    title: Option<String>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            variant: ContainerVariant::Base,
            title: None,
        }
    }

    pub fn variant(mut self, variant: ContainerVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Renders the container and returns the inner area for content placement
    pub fn render(self, area: Rect, buf: &mut Buffer, state: &BarState) -> Rect {
        let (bg, border_fg) = match self.variant {
            ContainerVariant::Base => (
                state.cookbook.resolve_bg("bg"),
                state.cookbook.resolve_color("fg"), // Normal border
            ),
            ContainerVariant::Panel => (
                state.cookbook.resolve_bg("panel_bg"),
                state.cookbook.resolve_color("panel_border"),
            ),
            ContainerVariant::Glass => (
                // Glass usually implies semi-transparent, handled by specialized renderers or composition
                // For now, we map it to a specific key
                state.cookbook.resolve_bg("glass_bg"),
                state.cookbook.resolve_color("accent"),
            ),
            ContainerVariant::Alert => (
                state.cookbook.resolve_bg("error_bg"),
                state.cookbook.resolve_color("error"),
            ),
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_fg));

        // Only set BG if it's not Reset, to allow transparency
        if bg != ratatui::style::Color::Reset {
            block = block.style(Style::default().bg(bg));
        }

        if let Some(t) = self.title {
            block = block.title(t);
        }

        let inner = block.inner(area);
        block.render(area, buf);

        inner
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
