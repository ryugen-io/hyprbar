use crate::config::SinkConfig;
use crate::dish::Dish;
use crate::state::BarState;
use anyhow::Result;
use k_lib::config::Cookbook;
use log::debug;
use ratatui::prelude::*;

use std::time::Duration;
use tachyonfx::{Effect, Interpolation, fx};

pub struct BarRenderer {
    buffer: Buffer,
    effects: Vec<Effect>,
    pub width: u16,
    pub height: u16,
    left_dishes: Vec<Box<dyn Dish>>,
    center_dishes: Vec<Box<dyn Dish>>,
    right_dishes: Vec<Box<dyn Dish>>,
}

impl BarRenderer {
    pub fn new(
        width: u16,
        height: u16,
        config: &SinkConfig,
        cookbook: &Cookbook,
        provider: &dyn crate::dish::DishProvider,
    ) -> Self {
        debug!("Initializing BarRenderer with size {}x{}", width, height);
        let area = Rect::new(0, 0, width, height);

        let effects = vec![fx::fade_from(
            Color::Cyan,
            Color::Cyan,
            (800, Interpolation::SineInOut),
        )];

        let left_dishes =
            Self::init_dishes(&config.layout.modules_left, config, cookbook, provider);
        let center_dishes =
            Self::init_dishes(&config.layout.modules_center, config, cookbook, provider);
        let right_dishes =
            Self::init_dishes(&config.layout.modules_right, config, cookbook, provider);

        Self {
            buffer: Buffer::empty(area),
            effects,
            width,
            height,
            left_dishes,
            center_dishes,
            right_dishes,
        }
    }

    fn init_dishes(
        names: &[String],
        _config: &SinkConfig,
        cookbook: &Cookbook,
        provider: &dyn crate::dish::DishProvider,
    ) -> Vec<Box<dyn Dish>> {
        let mut dishes: Vec<Box<dyn Dish>> = Vec::new();
        // Preset is a struct, use .msg for the format string
        let log_fmt = cookbook
            .dictionary
            .presets
            .get("dish_loaded")
            .map(|p| p.msg.clone())
            .unwrap_or_else(|| "Loaded Dish: {0} (Type: {1})".to_string());

        for raw_name in names {
            let (name, alias) = raw_name
                .split_once('#')
                .unwrap_or((raw_name.as_str(), raw_name.as_str()));

            if let Some(mut plugin_dish) = provider.create_dish(name) {
                // Configure instance alias
                plugin_dish.set_instance_config(alias.to_string());

                let display_name = if name != alias {
                    format!("{} as {}", name, alias)
                } else {
                    name.to_string()
                };

                let msg = log_fmt
                    .replace("{0}", &display_name)
                    .replace("{1}", "Plugin");
                log::info!("{}", msg);
                dishes.push(plugin_dish);
            } else {
                debug!("Unknown dish: {}", name);
            }
        }
        dishes
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
        // Update all dishes first
        for dish in self.left_dishes.iter_mut() {
            dish.update(dt);
        }
        for dish in self.center_dishes.iter_mut() {
            dish.update(dt);
        }
        for dish in self.right_dishes.iter_mut() {
            dish.update(dt);
        }

        let area = Rect::new(0, 0, self.width, self.height);
        self.buffer.reset();

        // Layout Chunks
        let layout_constraints = [
            Constraint::Percentage(state.config.layout.left as u16),
            Constraint::Percentage(state.config.layout.center as u16),
            Constraint::Percentage(state.config.layout.right as u16),
        ];

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(layout_constraints)
            .split(area);

        // Render Left
        Self::render_section(
            &mut self.buffer,
            chunks[0],
            &mut self.left_dishes,
            state,
            Alignment::Left,
            dt,
        );
        // Render Center
        Self::render_section(
            &mut self.buffer,
            chunks[1],
            &mut self.center_dishes,
            state,
            Alignment::Center,
            dt,
        );
        // Render Right
        Self::render_section(
            &mut self.buffer,
            chunks[2],
            &mut self.right_dishes,
            state,
            Alignment::Right,
            dt,
        );

        // Apply effects
        self.effects.retain_mut(|effect| {
            effect.process(dt, &mut self.buffer, area);
            !effect.done()
        });

        Ok(())
    }

    fn render_section(
        buffer: &mut Buffer,
        area: Rect,
        dishes: &mut [Box<dyn Dish>],
        state: &BarState,
        align: Alignment,
        dt: Duration,
    ) {
        if dishes.is_empty() {
            return;
        }

        let dish_widths: Vec<u16> = dishes.iter().map(|d| d.width(state)).collect();
        let total_width: u16 = dish_widths.iter().sum();

        // If content is wider than chunk, it will clip.
        // If smaller, alignment matters.

        let mut current_x = match align {
            Alignment::Left => area.x,
            Alignment::Center => area.x + (area.width.saturating_sub(total_width)) / 2,
            Alignment::Right => area.x + area.width.saturating_sub(total_width),
        };

        for (i, dish) in dishes.iter_mut().enumerate() {
            let w = dish_widths[i];
            let render_area = Rect::new(current_x, area.y, w, area.height);
            // Ensure we don't draw outside chunk
            let intersection = render_area.intersection(area);
            if !intersection.is_empty() {
                dish.render(intersection, buffer, state, dt);
            }
            current_x += w;
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}
