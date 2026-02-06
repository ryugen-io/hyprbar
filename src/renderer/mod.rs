mod input;
mod layout;
mod popup;
mod types;
mod widgets;

pub use types::{ActivePopup, WidgetSection};

use crate::config::BarConfig;
use crate::modules::logging::*;
use crate::state::BarState;
use crate::widget::Widget;
use anyhow::Result;
use hyprink::config::Config;
use ratatui::prelude::*;
use std::time::Duration;
use tachyonfx::{Effect, Interpolation, fx};
use types::HitWidget;

pub struct BarRenderer {
    buffer: Buffer,
    effects: Vec<Effect>,
    pub width: u16,
    pub height: u16,
    pub(crate) left_widgets: Vec<Box<dyn Widget>>,
    pub(crate) center_widgets: Vec<Box<dyn Widget>>,
    pub(crate) right_widgets: Vec<Box<dyn Widget>>,
    pub(crate) hit_map: Vec<HitWidget>,
    pub(crate) hovered_widget: Option<(WidgetSection, usize)>,
    pub(crate) popup_buffer: Option<Buffer>,
    pub(crate) active_popup: Option<ActivePopup>,
}

impl BarRenderer {
    pub fn new(
        width: u16,
        height: u16,
        config: &BarConfig,
        config_ink: &Config,
        provider: &dyn crate::widget::WidgetProvider,
    ) -> Self {
        log_debug(
            "RENDER",
            &format!("Initializing BarRenderer {}x{}", width, height),
        );
        let area = Rect::new(0, 0, width, height);

        let mut effects = Vec::new();

        if let Some(anim_config) = &config.style.animation {
            let duration = anim_config.duration.unwrap_or(800) as u32;
            if let Some(entry) = &anim_config.entry {
                match entry.as_str() {
                    "slide_up" | "slide_down" | "slide_left" | "slide_right" => {
                        effects.push(fx::fade_from(
                            Color::Black,
                            Color::Reset,
                            (duration, Interpolation::SineInOut),
                        ));
                    }
                    "fade" => effects.push(fx::fade_from(
                        Color::Black,
                        Color::Reset,
                        (duration, Interpolation::SineInOut),
                    )),
                    _ => log_warn("RENDER", &format!("Unknown animation: {}", entry)),
                }
            }
        } else {
            effects.push(fx::fade_from(
                Color::Cyan,
                Color::Cyan,
                (800, Interpolation::SineInOut),
            ));
        }

        log_debug("RENDER", "Initializing left widgets");
        let left_widgets =
            Self::init_widgets(&config.layout.modules_left, config, config_ink, provider);
        log_debug("RENDER", "Initializing center widgets");
        let center_widgets =
            Self::init_widgets(&config.layout.modules_center, config, config_ink, provider);
        log_debug("RENDER", "Initializing right widgets");
        let right_widgets =
            Self::init_widgets(&config.layout.modules_right, config, config_ink, provider);

        let total_widgets = left_widgets.len() + center_widgets.len() + right_widgets.len();
        log_info(
            "RENDER",
            &format!(
                "BarRenderer initialized: {} widgets (L:{}, C:{}, R:{})",
                total_widgets,
                left_widgets.len(),
                center_widgets.len(),
                right_widgets.len()
            ),
        );

        Self {
            buffer: Buffer::empty(area),
            effects,
            width,
            height,
            left_widgets,
            center_widgets,
            right_widgets,
            hit_map: Vec::new(),
            hovered_widget: None,
            popup_buffer: None,
            active_popup: None,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            log_debug(
                "RENDER",
                &format!("Resizing buffer to {}x{}", width, height),
            );
            self.width = width;
            self.height = height;
            let area = Rect::new(0, 0, width, height);
            self.buffer = Buffer::empty(area);
        }
    }

    pub fn render_frame(&mut self, state: &BarState, dt: Duration) -> Result<()> {
        for widget in self.left_widgets.iter_mut() {
            widget.update(dt, state);
        }
        for widget in self.center_widgets.iter_mut() {
            widget.update(dt, state);
        }
        for widget in self.right_widgets.iter_mut() {
            widget.update(dt, state);
        }

        let area = Rect::new(0, 0, self.width, self.height);
        self.buffer.reset();
        self.hit_map.clear();

        let strategy = state.config.layout.strategy.as_str();
        let padding = state.config.layout.padding;

        let chunks = if strategy == "flex" {
            self.calculate_flex_rects(area, state, padding)
        } else {
            let layout_constraints = [
                Constraint::Percentage(state.config.layout.left as u16),
                Constraint::Percentage(state.config.layout.center as u16),
                Constraint::Percentage(state.config.layout.right as u16),
            ];

            let rects = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(layout_constraints)
                .split(area);

            vec![rects[0], rects[1], rects[2]]
        };

        Self::render_section(
            &mut self.buffer,
            chunks[0],
            &mut self.left_widgets,
            &mut self.hit_map,
            WidgetSection::Left,
            state,
            Alignment::Left,
            dt,
            padding,
        );
        Self::render_section(
            &mut self.buffer,
            chunks[1],
            &mut self.center_widgets,
            &mut self.hit_map,
            WidgetSection::Center,
            state,
            Alignment::Center,
            dt,
            padding,
        );
        Self::render_section(
            &mut self.buffer,
            chunks[2],
            &mut self.right_widgets,
            &mut self.hit_map,
            WidgetSection::Right,
            state,
            Alignment::Right,
            dt,
            padding,
        );

        self.effects.retain_mut(|effect: &mut Effect| {
            effect.process(dt, &mut self.buffer, area);
            !effect.done()
        });

        Ok(())
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}
