use crate::style::ThemeExt;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    widgets::{Paragraph, Widget},
};

pub enum TypographyVariant {
    Header,
    Body,
    Mono,
    Accent,
}

pub struct Label<'a> {
    content: &'a str,
    variant: TypographyVariant,
    alignment: Alignment,
}

impl<'a> Label<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            variant: TypographyVariant::Body,
            alignment: Alignment::Left,
        }
    }

    pub fn variant(mut self, variant: TypographyVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn render(self, area: Rect, buf: &mut Buffer, theme: &impl ThemeExt) {
        let style = match self.variant {
            TypographyVariant::Header => Style::default()
                .fg(theme.resolve_color("header_fg"))
                .add_modifier(Modifier::BOLD),
            TypographyVariant::Body => Style::default().fg(theme.resolve_color("fg")),
            TypographyVariant::Mono => Style::default().fg(theme.resolve_color("mono_fg")), // Could imply a font change if backend supported it
            TypographyVariant::Accent => Style::default()
                .fg(theme.resolve_color("accent"))
                .add_modifier(Modifier::ITALIC),
        };

        Paragraph::new(self.content)
            .style(style)
            .alignment(self.alignment)
            .render(area, buf);
    }
}
