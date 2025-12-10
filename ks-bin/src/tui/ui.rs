use super::app::App;
use kitchn_lib::config::Cookbook;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
// use tachyonfx::{Color, Effect, EffectQueue, EffectTimer, Interpolation, Shader}; // Comment out for now until verified

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // 1. Background
    let bg_color = resolve_color(&app.cookbook, "bg");
    let fg_color = resolve_color(&app.cookbook, "fg");

    let block = Block::default().style(Style::default().bg(bg_color).fg(fg_color));
    f.render_widget(block, size);

    // 2. Layout (Header, Content, Footer)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    draw_header(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let accent = resolve_color(&app.cookbook, "accent");
    let title = Paragraph::new(Span::styled(
        "KITCHN SINK MANAGER",
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, area);
}

fn draw_content(f: &mut Frame, app: &mut App, area: Rect) {
    let accent = resolve_color(&app.cookbook, "accent");
    let fg = resolve_color(&app.cookbook, "fg");

    // Status Indicator
    let status_text = if app.running {
        Span::styled("● ACTIVE", Style::default().fg(Color::Green))
    } else {
        Span::styled("○ INACTIVE", Style::default().fg(Color::Red))
    };

    // Split content area for Status + List
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Status
            Constraint::Min(0),    // List
        ])
        .split(area);

    let status_para = Paragraph::new(Line::from(vec![Span::raw("Status: "), status_text]))
        .alignment(Alignment::Center);
    f.render_widget(status_para, content_chunks[0]);

    // Menu List
    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|i| {
            let lines = vec![Line::from(i.label.clone())];
            ListItem::new(lines).style(Style::default().fg(fg))
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .fg(bg_color(&app.cookbook)) // Invert for highlight
                .bg(accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, content_chunks[1], &mut app.state);

    // Description Box (DMS style side or bottom panel)
    // For now, let's put it at the bottom of the list area or make it a popup?
    // Let's stick to simple list for now as per "base form".
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let dimmed = resolve_color(&app.cookbook, "dimmed"); // Fallback needed if not in theme
    let keys = Paragraph::new("ESC/q: Quit  ↑/↓: Navigate  Enter: Select")
        .style(Style::default().fg(dimmed))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(keys, area);
}

// Helper to resolve colors from cookbook or fallback
fn resolve_color(cookbook: &Cookbook, key: &str) -> Color {
    // This assumes specific keys exist in stylesheet, mapping them is tricky without direct access
    // to the hashmap values in a way that maps to Ratatui Colors cleanly without parsing.
    // For this prototype, we'll try to parse hex strings or fallback.

    if let Some(val) = cookbook.theme.colors.get(key) {
        // Try hex parsing manually if palette crate isn't used here directly,
        // but kitchn_lib might expose color parsing?
        // Let's implement a simple hex parser for standard kitchn theme strings "#RRGGBB"
        if val.starts_with('#') && val.len() == 7 {
            let r = u8::from_str_radix(&val[1..3], 16).unwrap_or(255);
            let g = u8::from_str_radix(&val[3..5], 16).unwrap_or(255);
            let b = u8::from_str_radix(&val[5..7], 16).unwrap_or(255);
            return Color::Rgb(r, g, b);
        }
    }

    match key {
        "bg" => Color::Reset, // Or generic dark
        "fg" => Color::White,
        "accent" => Color::Cyan,
        _ => Color::Gray,
    }
}

// Fallbacks for specific purposes
fn bg_color(cookbook: &Cookbook) -> Color {
    resolve_color(cookbook, "bg")
}
