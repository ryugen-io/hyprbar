use crate::config::BarConfig;
use crate::event::WidgetEvent;
use crate::modules::logging::*;
use crate::renderer::BarRenderer;
use crate::state::BarState;
use crate::wayland::init as init_wayland;
use crate::wayland::state::WaylandState;
use crate::wayland::{create_popup_surface, destroy_popup_surface};
use anyhow::{Context, Result};
use smithay_client_toolkit::reexports::client::EventQueue;
use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use std::time::Duration;

pub async fn init_wayland_integration(
    config: &BarConfig,
) -> Result<(WaylandState, EventQueue<WaylandState>, LayerSurface)> {
    log_info("WAYLAND", "Initializing Wayland integration");

    let (font_size, window_height) = config.window.calculate_dimensions();
    log_debug(
        "WAYLAND",
        &format!(
            "Layout: Height {}px, Font {}px (Scale: {}, Pixel: {})",
            window_height, font_size, config.window.scale_font, config.window.pixel_font
        ),
    );

    let monitor = if config.window.monitor.is_empty() {
        log_debug("WAYLAND", "No monitor specified, using default");
        None
    } else {
        log_info(
            "WAYLAND",
            &format!("Target monitor: {}", config.window.monitor),
        );
        Some(config.window.monitor.clone())
    };

    let anchor = if config.window.anchor == "bottom" {
        "bottom"
    } else {
        "top"
    };
    log_debug("WAYLAND", &format!("Bar anchor: {}", anchor));

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

    if let Err(e) = event_queue.blocking_dispatch(wayland_state) {
        log_error("WAYLAND", &format!("Dispatch failed: {}", e));
        return Err(e).context("Wayland dispatch failed");
    }

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

    // Popup handling: check if widget wants a popup
    handle_popup_lifecycle(wayland_state, &qh, renderer, bar_state, config)?;

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

fn handle_popup_lifecycle(
    wayland_state: &mut WaylandState,
    qh: &smithay_client_toolkit::reexports::client::QueueHandle<WaylandState>,
    renderer: &mut BarRenderer,
    bar_state: &BarState,
    config: &BarConfig,
) -> Result<()> {
    let char_w = wayland_state.text_renderer.char_width;
    let char_h = wayland_state.text_renderer.char_height;
    let anchor_bottom = config.window.anchor == "bottom";

    // Check if hovered widget wants a popup
    if let Some((request, popup_info)) = renderer.check_popup_request() {
        // Check if we already have this popup active
        let needs_create = match renderer.active_popup() {
            Some(active) => {
                active.section != popup_info.section || active.index != popup_info.index
            }
            None => true,
        };

        if needs_create {
            // Calculate popup position in pixels
            // Widget left edge in pixels
            let widget_left_px = popup_info.widget_area.x as i32 * char_w as i32;

            let popup_width_px = request.width as u32 * char_w as u32;
            let popup_height_px = request.height as u32 * char_h as u32;

            // Base position: widget left edge + offsets
            // offset 0,0 = popup starts at widget's left edge
            let popup_x = widget_left_px + request.offset_x as i32 + config.popup.offset_x as i32;
            let popup_y = request.offset_y as i32 + config.popup.offset_y as i32;

            log_debug(
                "POPUP",
                &format!(
                    "Creating at ({}, {}) - config offset: ({}, {})",
                    popup_x, popup_y, config.popup.offset_x, config.popup.offset_y
                ),
            );

            // Create the popup surface
            create_popup_surface(
                wayland_state,
                qh,
                popup_width_px,
                popup_height_px,
                popup_x,
                popup_y,
                anchor_bottom,
            )?;

            // Update renderer state
            renderer.set_active_popup(popup_info, request.width, request.height);
        }
    } else {
        // No popup wanted, destroy if active
        if renderer.active_popup().is_some() {
            log_debug("POPUP", "Widget no longer requests popup, destroying");
            destroy_popup_surface(wayland_state);
            renderer.clear_active_popup();
        }
    }

    // Render popup if configured
    if wayland_state.popup_configured
        && wayland_state.popup_redraw_requested
        && let Some(buf) = renderer.render_popup(bar_state)
    {
        let bg = config.style.popup_bg.as_deref().unwrap_or(&config.style.bg);
        wayland_state.draw_popup(qh, buf, &bar_state.config_ink, bg)?;
    }

    Ok(())
}
