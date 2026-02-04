use crate::config::BarConfig;
use crate::event::WidgetEvent;
use crate::modules::logging::log_debug;
use crate::renderer::BarRenderer;
use crate::state::BarState;
use crate::wayland::init as init_wayland;
use crate::wayland::state::WaylandState;
use anyhow::{Context, Result};
use smithay_client_toolkit::reexports::client::EventQueue;
use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use std::time::Duration;

pub async fn init_wayland_integration(
    config: &BarConfig,
) -> Result<(WaylandState, EventQueue<WaylandState>, LayerSurface)> {
    // 6. Initialize Wayland & Smart Scaling
    let (font_size, window_height) = config.window.calculate_dimensions();
    log_debug(
        "WAYLAND",
        &format!(
            "Layout: Height {}px, Font {}px (Scale: {}, Pixel: {})",
            window_height, font_size, config.window.scale_font, config.window.pixel_font
        ),
    );

    let monitor = if config.window.monitor.is_empty() {
        None
    } else {
        Some(config.window.monitor.clone())
    };

    init_wayland(
        window_height,
        config.window.anchor == "bottom",
        monitor,
        config.style.font.clone(),
        font_size,
    )
    .context("Failed to initialize Wayland")
}

pub fn handle_wayland_events(
    wayland_state: &mut WaylandState,
    event_queue: &mut EventQueue<WaylandState>,
    renderer: &mut BarRenderer,
    bar_state: &BarState,
    config: &BarConfig,
) -> Result<()> {
    let qh = event_queue.handle();

    if wayland_state.configured {
        let width = u16::try_from(wayland_state.width).unwrap_or(u16::MAX);
        let height = u16::try_from(wayland_state.height).unwrap_or(u16::MAX);

        if width > 0 && height > 0 {
            // Calculate grid size based on font metrics
            let char_w = wayland_state.text_renderer.char_width as u16;
            let char_h = wayland_state.text_renderer.char_height as u16;

            let cols = width / char_w;
            let rows = height / char_h;

            if renderer.width != cols || renderer.height != rows {
                renderer.resize(cols, rows);
                log_debug(
                    "WAYLAND",
                    &format!(
                        "Resized to {}x{} cells (Win: {}x{}, Char: {}x{})",
                        cols, rows, width, height, char_w, char_h
                    ),
                );
            }
        }

        if wayland_state.redraw_requested {
            renderer.render_frame(bar_state, Duration::from_millis(16))?;
            wayland_state.draw(
                &qh,
                renderer.buffer(),
                &bar_state.config_ink,
                config
                    .style
                    .window_bg
                    .as_deref()
                    .unwrap_or(&config.style.bg),
            )?;
        }
    }

    event_queue
        .blocking_dispatch(wayland_state)
        .context("Wayland dispatch failed")?;

    // Process Input Events
    // We do this after dispatch to handle events received this turn
    for event in wayland_state.input_events.drain(..) {
        let char_w = wayland_state.text_renderer.char_width as f64;
        let char_h = wayland_state.text_renderer.char_height as f64;

        // Extract pixel coordinates if present, else use last known cursor pos
        let (px, py) = match event {
            WidgetEvent::Motion { x, y } | WidgetEvent::Click { x, y, .. } => (x as f64, y as f64),
            _ => (wayland_state.cursor_x, wayland_state.cursor_y),
        };

        // Convert to Cell Coordinates
        if char_w > 0.0 && char_h > 0.0 {
            let cx = (px / char_w) as u16;
            let cy = (py / char_h) as u16;

            // Create a Cell-based event
            let mut cell_event = event;
            match &mut cell_event {
                WidgetEvent::Motion { x, y } | WidgetEvent::Click { x, y, .. } => {
                    *x = cx;
                    *y = cy;
                }
                _ => {} // Leave/Enter/Scroll don't carry x/y in the enum variant usually (except scroll maybe?)
            }

            renderer.process_input(cx, cy, cell_event);
        }
    }

    // Render again if input events triggered a redraw
    if wayland_state.configured && wayland_state.redraw_requested {
        renderer.render_frame(bar_state, Duration::from_millis(16))?;
        wayland_state.draw(
            &qh,
            renderer.buffer(),
            &bar_state.config_ink,
            config
                .style
                .window_bg
                .as_deref()
                .unwrap_or(&config.style.bg),
        )?;
    }

    Ok(())
}
