use crate::ui::style::ThemeExt;
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
    pub fn render(self, area: Rect, buf: &mut Buffer, theme: &impl ThemeExt) -> Rect {
        let (bg, border_fg) = match self.variant {
            ContainerVariant::Base => (
                theme.resolve_bg("bg"),
                theme.resolve_color("fg"), // Normal border
            ),
            ContainerVariant::Panel => (
                theme.resolve_bg("panel_bg"),
                theme.resolve_color("panel_border"),
            ),
            ContainerVariant::Glass => (
                // Glass usually implies semi-transparent, handled by specialized renderers or composition
                // For now, we map it to a specific key
                theme.resolve_bg("glass_bg"),
                theme.resolve_color("accent"),
            ),
            ContainerVariant::Alert => (theme.resolve_bg("error_bg"), theme.resolve_color("error")),
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
