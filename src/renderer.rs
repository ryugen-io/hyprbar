use crate::config::BarConfig;
use crate::modules::logging::{log_debug, log_info, log_warn};
use crate::state::BarState;
use crate::widget::Widget;
use anyhow::Result;
use hyprink::config::Config;
use ratatui::prelude::*;

use std::time::Duration;
use tachyonfx::{Effect, Interpolation, fx};

#[derive(Debug, Clone, Copy, PartialEq)]
enum WidgetSection {
    Left,
    Center,
    Right,
}

struct HitWidget {
    area: Rect,
    section: WidgetSection,
    index: usize,
}

pub struct BarRenderer {
    buffer: Buffer,
    effects: Vec<Effect>,
    pub width: u16,
    pub height: u16,
    left_widgets: Vec<Box<dyn Widget>>,
    center_widgets: Vec<Box<dyn Widget>>,
    right_widgets: Vec<Box<dyn Widget>>,
    hit_map: Vec<HitWidget>,
    hovered_widget: Option<(WidgetSection, usize)>,
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
                        // TODO: Fix slide_in signature (needs 5 args)
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
            // Default Fallback
            effects.push(fx::fade_from(
                Color::Cyan,
                Color::Cyan,
                (800, Interpolation::SineInOut),
            ));
        }

        let left_widgets =
            Self::init_widgets(&config.layout.modules_left, config, config_ink, provider);
        let center_widgets =
            Self::init_widgets(&config.layout.modules_center, config, config_ink, provider);
        let right_widgets =
            Self::init_widgets(&config.layout.modules_right, config, config_ink, provider);

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
        }
    }

    pub fn process_input(&mut self, x: u16, y: u16, event: crate::event::WidgetEvent) {
        use crate::event::WidgetEvent;

        // Log surface-level events only (widget events logged in hover handling)
        match &event {
            WidgetEvent::Enter => log_debug("INPUT", "Pointer entered bar"),
            WidgetEvent::Leave => log_debug("INPUT", "Pointer left bar"),
            _ => {} // Widget-specific events logged below with widget name
        }

        // If Surface Leave, clear hover
        if let WidgetEvent::Leave = event {
            if let Some((section, idx)) = self.hovered_widget {
                match section {
                    WidgetSection::Left => self.left_widgets[idx].handle_event(WidgetEvent::Leave),
                    WidgetSection::Center => {
                        self.center_widgets[idx].handle_event(WidgetEvent::Leave)
                    }
                    WidgetSection::Right => {
                        self.right_widgets[idx].handle_event(WidgetEvent::Leave)
                    }
                }
            }
            self.hovered_widget = None;
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
            if self.hovered_widget != Some((section, idx)) {
                // Leave previous
                if let Some((old_sec, old_idx)) = self.hovered_widget {
                    let old_name = match old_sec {
                        WidgetSection::Left => self.left_widgets[old_idx].name(),
                        WidgetSection::Center => self.center_widgets[old_idx].name(),
                        WidgetSection::Right => self.right_widgets[old_idx].name(),
                    };
                    log_debug("WIDGET", &format!("Leave: {}", old_name));
                    match old_sec {
                        WidgetSection::Left => {
                            self.left_widgets[old_idx].handle_event(WidgetEvent::Leave)
                        }
                        WidgetSection::Center => {
                            self.center_widgets[old_idx].handle_event(WidgetEvent::Leave)
                        }
                        WidgetSection::Right => {
                            self.right_widgets[old_idx].handle_event(WidgetEvent::Leave)
                        }
                    }
                }
                // Enter new
                let new_name = match section {
                    WidgetSection::Left => self.left_widgets[idx].name(),
                    WidgetSection::Center => self.center_widgets[idx].name(),
                    WidgetSection::Right => self.right_widgets[idx].name(),
                };
                log_debug("WIDGET", &format!("Enter: {}", new_name));
                match section {
                    WidgetSection::Left => self.left_widgets[idx].handle_event(WidgetEvent::Enter),
                    WidgetSection::Center => {
                        self.center_widgets[idx].handle_event(WidgetEvent::Enter)
                    }
                    WidgetSection::Right => {
                        self.right_widgets[idx].handle_event(WidgetEvent::Enter)
                    }
                }
                self.hovered_widget = Some((section, idx));
            }

            // Localize Coordinates
            let mut local_event = event;
            match &mut local_event {
                WidgetEvent::Motion { x: mx, y: my } | WidgetEvent::Click { x: mx, y: my, .. } => {
                    *mx = mx.saturating_sub(area.x);
                    *my = my.saturating_sub(area.y);
                }
                _ => {}
            }

            // Log and dispatch event to widget
            let widget_name = match section {
                WidgetSection::Left => self.left_widgets[idx].name(),
                WidgetSection::Center => self.center_widgets[idx].name(),
                WidgetSection::Right => self.right_widgets[idx].name(),
            };
            match &local_event {
                WidgetEvent::Click { button, .. } => {
                    let btn = match *button {
                        272 => "L",
                        273 => "R",
                        274 => "M",
                        _ => "?",
                    };
                    log_debug("WIDGET", &format!("Click {} on {}", btn, widget_name));
                }
                WidgetEvent::Scroll { dy, .. } => {
                    log_debug("WIDGET", &format!("Scroll {} on {}", dy, widget_name));
                }
                _ => {}
            }
            match section {
                WidgetSection::Left => self.left_widgets[idx].handle_event(local_event),
                WidgetSection::Center => self.center_widgets[idx].handle_event(local_event),
                WidgetSection::Right => self.right_widgets[idx].handle_event(local_event),
            }
        } else {
            // Hit nothing. If we were hovering something, leave it.
            if let Some((old_sec, old_idx)) = self.hovered_widget {
                match old_sec {
                    WidgetSection::Left => {
                        self.left_widgets[old_idx].handle_event(WidgetEvent::Leave)
                    }
                    WidgetSection::Center => {
                        self.center_widgets[old_idx].handle_event(WidgetEvent::Leave)
                    }
                    WidgetSection::Right => {
                        self.right_widgets[old_idx].handle_event(WidgetEvent::Leave)
                    }
                }
            }
            self.hovered_widget = None;
        }
    }

    fn init_widgets(
        names: &[String],
        _config: &BarConfig,
        config_ink: &Config,
        provider: &dyn crate::widget::WidgetProvider,
    ) -> Vec<Box<dyn Widget>> {
        let mut widgets: Vec<Box<dyn Widget>> = Vec::new();
        // Labels are now plain strings
        let log_fmt = config_ink
            .layout
            .labels
            .get("widget_loaded")
            .cloned()
            .unwrap_or_else(|| "Loaded Dish: {0} (Type: {1})".to_string());

        for raw_name in names {
            // Support both '#' (legacy) and '.' (new) for sub-widgets
            let (name, alias) = if let Some((n, a)) = raw_name.split_once('.') {
                (n, a)
            } else {
                raw_name
                    .split_once('#')
                    .unwrap_or((raw_name.as_str(), raw_name.as_str()))
            };

            if let Some(mut plugin_widget) = provider.create_widget(name) {
                // Configure instance alias
                plugin_widget.set_instance_config(alias.to_string());

                let display_name = if name != alias {
                    format!("{} as {}", name, alias)
                } else {
                    name.to_string()
                };

                let msg = log_fmt
                    .replace("{0}", &display_name)
                    .replace("{1}", "Plugin");
                log_info("WIDGET", &msg);
                widgets.push(plugin_widget);
            } else {
                log_warn("WIDGET", &format!("Unknown widget: {}", name));
            }
        }
        widgets
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
        // Update all widgets first
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
            &mut self.left_widgets,
            &mut self.hit_map,
            WidgetSection::Left,
            state,
            Alignment::Left,
            dt,
            padding,
        );
        // Render Center
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
        // Render Right
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
        widgets: &mut [Box<dyn Widget>],
        hit_map: &mut Vec<HitWidget>,
        section: WidgetSection,
        state: &BarState,
        align: Alignment,
        dt: Duration,
        padding: u16,
    ) {
        if widgets.is_empty() {
            return;
        }

        let widget_widths: Vec<u16> = widgets.iter().map(|d| d.width(state)).collect();
        let total_width: u16 = widget_widths.iter().sum();

        // If content is wider than chunk, it will clip.
        // If smaller, alignment matters.

        let mut current_x = match align {
            Alignment::Left => area.x,
            Alignment::Center => area.x + (area.width.saturating_sub(total_width)) / 2,
            Alignment::Right => area.x + area.width.saturating_sub(total_width),
        };

        for (i, widget) in widgets.iter_mut().enumerate() {
            let w = widget_widths[i];
            let render_area = Rect::new(current_x, area.y, w, area.height);
            // Ensure we don't draw outside chunk
            let intersection = render_area.intersection(area);
            if !intersection.is_empty() {
                widget.render(intersection, buffer, state, dt);
                hit_map.push(HitWidget {
                    area: intersection,
                    section,
                    index: i,
                });
            }
            current_x += w + padding;
        }
    }

    fn calculate_flex_rects(&self, area: Rect, state: &BarState, padding: u16) -> Vec<Rect> {
        let left_w = Self::calc_width(&self.left_widgets, state, padding);
        let right_w = Self::calc_width(&self.right_widgets, state, padding);
        let center_w = Self::calc_width(&self.center_widgets, state, padding);

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

    fn calc_width(widgets: &[Box<dyn Widget>], state: &BarState, padding: u16) -> u16 {
        if widgets.is_empty() {
            return 0;
        }
        let sum: u16 = widgets.iter().map(|d| d.width(state)).sum();
        let gaps = (widgets.len() as u16).saturating_sub(1);
        sum + gaps * padding
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}
