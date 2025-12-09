use crate::state::BarState;
use anyhow::Result;
use kitchn_lib::factory::ColorResolver;
use log::debug;
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget};
use std::time::Duration;
use tachyonfx::{Effect, Interpolation, Shader, fx};

pub struct BarRenderer {
    buffer: Buffer,
    effects: Vec<Effect>,
    pub width: u16,
    pub height: u16,
}

impl BarRenderer {
    pub fn new(width: u16, height: u16) -> Self {
        debug!("Initializing BarRenderer with size {}x{}", width, height);
        let area = Rect::new(0, 0, width, height);
        // Example startup effect
        // Example startup effect: Fade from Cyan (FG & BG) to default
        let effects = vec![Effect::new(fx::fade_from(
            Color::Cyan,
            Color::Cyan,
            (800, Interpolation::SineInOut),
        ))];

        Self {
            buffer: Buffer::empty(area),
            effects,
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            debug!("Resizing buffer to {}x{}", width, height);
            self.width = width;
            self.height = height;
            let area = Rect::new(0, 0, width, height);
            self.buffer = Buffer::empty(area);
        }
    }

    pub fn render_frame(&mut self, state: &BarState, dt: Duration) -> Result<()> {
        let area = Rect::new(0, 0, self.width, self.height);

        // Reset buffer
        self.buffer.reset();

        // Build widgets based on state
        // Build widgets based on state
        // Decoupled from Cookbook (use Config)
        let bg_color_hex = &state.config.style.bg;
        let fg_color_hex = &state.config.style.fg;

        let bg_custom = ColorResolver::hex_to_color(bg_color_hex);
        let fg_custom = ColorResolver::hex_to_color(fg_color_hex);

        let bg = Color::Rgb(bg_custom.r, bg_custom.g, bg_custom.b);
        let fg = Color::Rgb(fg_custom.r, fg_custom.g, fg_custom.b);

        let text = format!(
            " kitchnsink | CPU: {:.1}% | MEM: {:.1}% | {} ",
            state.cpu, state.mem, state.time
        );

        let bar = Paragraph::new(text)
            .style(Style::default().fg(fg).bg(bg))
            .alignment(Alignment::Center);

        bar.render(area, &mut self.buffer);

        // Apply effects
        self.effects.retain_mut(|effect| {
            effect.process(dt, &mut self.buffer, area);
            !effect.done()
        });

        Ok(())
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}
