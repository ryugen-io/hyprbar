use anyhow::Result;
use k_lib::config::Cookbook;
use ks_lib::prelude::*;
use ks_ui::{Container, ContainerVariant, Label, TypographyVariant};
use ratatui::{
    backend::TestBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::sync::Arc;

#[allow(dead_code)]
fn main() -> Result<()> {
    // 1. Mock State
    let cookbook = Arc::new(Cookbook::load().expect("Failed to load kitchn cookbook"));
    let config = SinkConfig::default();
    let state = BarState::new(cookbook, config);

    // 2. Setup Terminal (Test Backend)
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;

    // 3. Render Loop (Single Frame)
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(0)])
            .split(f.area());

        // Outer Container (Panel)
        let inner_area = Container::new()
            .variant(ContainerVariant::Panel)
            .title(" UI Kit Demo ")
            .render(chunks[0], f.buffer_mut(), state.cookbook.as_ref());

        // Inner Content
        let text_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(inner_area);

        Label::new("This is a Header")
            .variant(TypographyVariant::Header)
            .render(text_chunks[0], f.buffer_mut(), state.cookbook.as_ref());

        Label::new("This is body text inside a container.")
            .variant(TypographyVariant::Body)
            .render(text_chunks[1], f.buffer_mut(), state.cookbook.as_ref());
    })?;

    println!("Render successful!");
    Ok(())
}
