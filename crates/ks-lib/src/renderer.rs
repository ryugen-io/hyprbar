use crate::config::SinkConfig;
use crate::dish::Dish;
use crate::state::BarState;
use anyhow::Result;
use k_lib::config::Cookbook;
use log::debug;
use ratatui::prelude::*;

use std::time::Duration;
use tachyonfx::{Effect, Interpolation, fx};

#[derive(Debug, Clone, Copy, PartialEq)]
enum DishSection {
    Left,
    Center,
    Right,
}

struct HitDish {
    area: Rect,
    section: DishSection,
    index: usize,
}

pub struct BarRenderer {
    buffer: Buffer,
    effects: Vec<Effect>,
    pub width: u16,
    pub height: u16,
    left_dishes: Vec<Box<dyn Dish>>,
    center_dishes: Vec<Box<dyn Dish>>,
    right_dishes: Vec<Box<dyn Dish>>,
    hit_map: Vec<HitDish>,
    hovered_dish: Option<(DishSection, usize)>,
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
            hit_map: Vec::new(),
            hovered_dish: None,
        }
    }

    pub fn process_input(&mut self, x: u16, y: u16, event: crate::event::DishEvent) {
        use crate::event::DishEvent;

        // If Surface Leave, clear hover
        if let DishEvent::Leave = event {
            if let Some((section, idx)) = self.hovered_dish {
                match section {
                    DishSection::Left => self.left_dishes[idx].handle_event(DishEvent::Leave),
                    DishSection::Center => self.center_dishes[idx].handle_event(DishEvent::Leave),
                    DishSection::Right => self.right_dishes[idx].handle_event(DishEvent::Leave),
                }
            }
            self.hovered_dish = None;
            return;
        }

        let mut hit_found = None;
        for hit in &self.hit_map {
            if x >= hit.area.x
                && x < hit.area.x + hit.area.width
                && y >= hit.area.y
                && y < hit.area.y + hit.area.height
            {
                hit_found = Some((hit.section, hit.index, hit.area));
                break;
            }
        }

        if let Some((section, idx, area)) = hit_found {
            // Handle Hover Transitions
            if self.hovered_dish != Some((section, idx)) {
                // Leave previous
                if let Some((old_sec, old_idx)) = self.hovered_dish {
                    match old_sec {
                        DishSection::Left => {
                            self.left_dishes[old_idx].handle_event(DishEvent::Leave)
                        }
                        DishSection::Center => {
                            self.center_dishes[old_idx].handle_event(DishEvent::Leave)
                        }
                        DishSection::Right => {
                            self.right_dishes[old_idx].handle_event(DishEvent::Leave)
                        }
                    }
                }
                // Enter new
                match section {
                    DishSection::Left => self.left_dishes[idx].handle_event(DishEvent::Enter),
                    DishSection::Center => self.center_dishes[idx].handle_event(DishEvent::Enter),
                    DishSection::Right => self.right_dishes[idx].handle_event(DishEvent::Enter),
                }
                self.hovered_dish = Some((section, idx));
            }

            // Localize Coordinates
            let mut local_event = event;
            match &mut local_event {
                DishEvent::Motion { x: mx, y: my } | DishEvent::Click { x: mx, y: my, .. } => {
                    *mx = mx.saturating_sub(area.x);
                    *my = my.saturating_sub(area.y);
                }
                _ => {}
            }

            // Dispatch Actual Event (Motion, Click, Scroll)
            match section {
                DishSection::Left => self.left_dishes[idx].handle_event(local_event),
                DishSection::Center => self.center_dishes[idx].handle_event(local_event),
                DishSection::Right => self.right_dishes[idx].handle_event(local_event),
            }
        } else {
            // Hit nothing. If we were hovering something, leave it.
            if let Some((old_sec, old_idx)) = self.hovered_dish {
                match old_sec {
                    DishSection::Left => self.left_dishes[old_idx].handle_event(DishEvent::Leave),
                    DishSection::Center => {
                        self.center_dishes[old_idx].handle_event(DishEvent::Leave)
                    }
                    DishSection::Right => self.right_dishes[old_idx].handle_event(DishEvent::Leave),
                }
            }
            self.hovered_dish = None;
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
            // Support both '#' (legacy) and '.' (new) for sub-dishes
            let (name, alias) = if let Some((n, a)) = raw_name.split_once('.') {
                (n, a)
            } else {
                raw_name
                    .split_once('#')
                    .unwrap_or((raw_name.as_str(), raw_name.as_str()))
            };

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
        self.hit_map.clear();

        // Layout Chunks
        // Check Strategy
        let strategy = state.config.layout.strategy.as_str();
        let padding = state.config.layout.padding;

        let chunks = if strategy == "flex" {
            self.calculate_flex_rects(area, state, padding)
        } else {
            // Legacy Grid
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

        // Render Left
        Self::render_section(
            &mut self.buffer,
            chunks[0],
            &mut self.left_dishes,
            &mut self.hit_map,
            DishSection::Left,
            state,
            Alignment::Left,
            dt,
            padding,
        );
        // Render Center
        Self::render_section(
            &mut self.buffer,
            chunks[1],
            &mut self.center_dishes,
            &mut self.hit_map,
            DishSection::Center,
            state,
            Alignment::Center,
            dt,
            padding,
        );
        // Render Right
        Self::render_section(
            &mut self.buffer,
            chunks[2],
            &mut self.right_dishes,
            &mut self.hit_map,
            DishSection::Right,
            state,
            Alignment::Right,
            dt,
            padding,
        );

        // Apply effects
        self.effects.retain_mut(|effect: &mut Effect| {
            effect.process(dt, &mut self.buffer, area);
            !effect.done()
        });

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn render_section(
        buffer: &mut Buffer,
        area: Rect,
        dishes: &mut [Box<dyn Dish>],
        hit_map: &mut Vec<HitDish>,
        section: DishSection,
        state: &BarState,
        align: Alignment,
        dt: Duration,
        padding: u16,
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
                hit_map.push(HitDish {
                    area: intersection,
                    section,
                    index: i,
                });
            }
            current_x += w + padding;
        }
    }

    fn calculate_flex_rects(&self, area: Rect, state: &BarState, padding: u16) -> Vec<Rect> {
        let left_w = Self::calc_width(&self.left_dishes, state, padding);
        let right_w = Self::calc_width(&self.right_dishes, state, padding);
        let center_w = Self::calc_width(&self.center_dishes, state, padding);

        let total_available = area.width;

        // 1. Left (takes what it needs, up to total)
        let final_left_w = left_w.min(total_available);

        // 2. Right (takes what it needs from remainder)
        let remaining_after_left = total_available.saturating_sub(final_left_w);
        let final_right_w = right_w.min(remaining_after_left);

        // 3. Center (tries to be in the middle of screen)
        let remaining_for_center = remaining_after_left.saturating_sub(final_right_w);
        let final_center_w = center_w.min(remaining_for_center);

        let left_rect = Rect::new(area.x, area.y, final_left_w, area.height);

        let right_x = area.x + area.width - final_right_w;
        let right_rect = Rect::new(right_x, area.y, final_right_w, area.height);

        // Center calculation
        // Ideal X = Center of Screen - Half Center Width
        let ideal_center_x = area.x + (area.width.saturating_sub(final_center_w)) / 2;

        // Constraints
        let min_center_x = left_rect.x + left_rect.width; // Cant go into Left
        let max_center_x = right_rect.x.saturating_sub(final_center_w); // Cant go into Right

        let final_center_x = ideal_center_x.clamp(min_center_x, max_center_x);
        let center_rect = Rect::new(final_center_x, area.y, final_center_w, area.height);

        vec![left_rect, center_rect, right_rect]
    }

    fn calc_width(dishes: &[Box<dyn Dish>], state: &BarState, padding: u16) -> u16 {
        if dishes.is_empty() {
            return 0;
        }
        let sum: u16 = dishes.iter().map(|d| d.width(state)).sum();
        let gaps = (dishes.len() as u16).saturating_sub(1);
        sum + gaps * padding
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}
