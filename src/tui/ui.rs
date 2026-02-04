use super::app::App;
use crate::config::BarConfig;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // 1. Background
    let bg_color = resolve_color(&app.config, "bg");
    let fg_color = resolve_color(&app.config, "fg");

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
    let accent = resolve_color(&app.config, "accent");
    let title = Paragraph::new(Span::styled(
        "HYPRBAR MANAGER",
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, area);
}

fn draw_content(f: &mut Frame, app: &mut App, area: Rect) {
    let accent = resolve_color(&app.config, "accent");
    let fg = resolve_color(&app.config, "fg");

    // Status Indicator
    let status_text = if app.running {
        Span::styled(
            "● ACTIVE",
            Style::default().fg(resolve_color(&app.config, "success")),
        )
    } else {
        Span::styled(
            "○ INACTIVE",
            Style::default().fg(resolve_color(&app.config, "error")),
        )
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
                .fg(bg_color(&app.config)) // Invert for highlight
                .bg(accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, content_chunks[1], &mut app.state);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let dimmed = resolve_color(&app.config, "dimmed");
    let keys = Paragraph::new("ESC/q: Quit  ↑/↓: Navigate  Enter: Select")
        .style(Style::default().fg(dimmed))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(keys, area);
}

// Helper to resolve colors from config
fn resolve_color(config: &BarConfig, key: &str) -> Color {
    match key {
        "bg" => parse_hex(&config.style.bg),
        "fg" => parse_hex(&config.style.fg),
        "accent" | "primary" => {
            if let Some(ref val) = config.style.primary {
                return parse_hex(val);
            }
            if let Some(ref val) = config.style.accent {
                return parse_hex(val);
            }
            Color::Cyan
        }
        "secondary" => {
            if let Some(ref val) = config.style.secondary {
                return parse_hex(val);
            }
            Color::Magenta
        }
        "success" => {
            if let Some(ref val) = config.style.success {
                return parse_hex(val);
            }
            Color::Green
        }
        "error" => {
            if let Some(ref val) = config.style.error {
                return parse_hex(val);
            }
            Color::Red
        }
        "dimmed" => Color::DarkGray,
        _ => Color::Gray,
    }
}

fn parse_hex(val: &str) -> Color {
    if val.starts_with('#') && val.len() == 7 {
        let r = u8::from_str_radix(&val[1..3], 16).unwrap_or(255);
        let g = u8::from_str_radix(&val[3..5], 16).unwrap_or(255);
        let b = u8::from_str_radix(&val[5..7], 16).unwrap_or(255);
        return Color::Rgb(r, g, b);
    }
    Color::Reset
}

// Fallbacks for specific purposes
// Fallbacks for specific purposes
fn bg_color(config: &BarConfig) -> Color {
    resolve_color(config, "bg")
}
