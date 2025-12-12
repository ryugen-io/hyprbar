use anyhow::{Context, Result};
use k_lib::config::Cookbook;
use ks_core::config::SinkConfig;
use ks_core::event::DishEvent;
use ks_core::renderer::BarRenderer;
use ks_core::state::BarState;
use ks_wayland::init as init_wayland;
use log::{debug, error};
use std::time::Duration;

use crate::plugin_loader::PluginManager;

use std::sync::Arc;

pub fn run_server(cookbook: Arc<Cookbook>, config: SinkConfig) -> Result<()> {
    // let cookbook = std::sync::Arc::new(cookbook); // Removed, passed as Arc directly
    // Pre-fetch log strings (Cookbook consumed later)
    let get_msg = |key: &str, default: &str| -> String {
        cookbook
            .dictionary
            .presets
            .get(key)
            .map(|p| p.msg.clone())
            .unwrap_or_else(|| default.to_string())
    };

    let msg_loop = get_msg("sink_start_loop", "Starting Wayland event loop");
    let msg_exit = get_msg("sink_exit", "Exiting...");
    let msg_sigterm = get_msg(
        "sink_sigterm",
        "Received SIGTERM (Toggle), shutting down...",
    );
    let msg_sigint = get_msg("sink_sigint", "Received SIGINT, shutting down...");

    // Spawn Signal Handler
    let signal_cookbook = cookbook.clone();
    tokio::spawn(async move {
        use k_lib::logger;
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).unwrap();
        let mut int = signal(SignalKind::interrupt()).unwrap();

        tokio::select! {
            _ = term.recv() => logger::log_to_terminal(&signal_cookbook, "info", "SINK", &msg_sigterm),
            _ = int.recv() => logger::log_to_terminal(&signal_cookbook, "info", "SINK", &msg_sigint),
        }

        // Give a tiny bit of time for logs to flush if needed, though they are broadcast immediately
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(0);
    });

    // 4. Initialize Bar State
    let mut bar_state = BarState::new(cookbook, config.clone());

    // 5. Initialize Renderer (Offscreen)
    let mut plugin_manager = PluginManager::new();

    // Load plugins from ~/.local/share/kitchnsink/dishes
    if let Some(data_dir) = dirs::data_local_dir() {
        let dishes_dir = data_dir.join("kitchnsink/dishes");
        if dishes_dir.exists()
            && let Ok(entries) = std::fs::read_dir(dishes_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "dish")
                    && let Err(e) = plugin_manager.load_plugin(&path, true, true)
                {
                    error!("Failed to load plugin {:?}: {}", path, e);
                }
            }
        }
    }

    let mut renderer = BarRenderer::new(
        100,
        config.window.height as u16,
        &config,
        &bar_state.cookbook,
        &plugin_manager,
    );

    // 6. Initialize Wayland
    // 6. Initialize Wayland & Smart Scaling
    let (font_size, window_height) = config.window.calculate_dimensions();
    debug!(
        "Layout Calculated: Height {}px, Font {}px (Smart Scaling: {}, Pixel Font: {})",
        window_height, font_size, config.window.scale_font, config.window.pixel_font
    );

    let (mut wayland_state, mut event_queue, _layer_surface) = init_wayland(
        window_height,
        config.window.anchor == "bottom",
        Some(config.window.monitor.clone()),
        config.style.font.clone(),
        font_size,
    )
    .context("Failed to initialize Wayland")?;
    let qh = event_queue.handle();

    // 7. Event Loop
    // Use logger to match system theme
    k_lib::logger::log_to_terminal(&bar_state.cookbook, "info", "SINK", &msg_loop);

    loop {
        if wayland_state.exit {
            k_lib::logger::log_to_terminal(&bar_state.cookbook, "info", "SINK", &msg_exit);
            break;
        }

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
                    debug!(
                        "Resized to {}x{} cells (Window: {}x{}, Char: {}x{})",
                        cols, rows, width, height, char_w, char_h
                    );
                }
            }

            if wayland_state.redraw_requested {
                bar_state.cpu = 12.5;
                bar_state.mem = 45.2;

                renderer.render_frame(&bar_state, Duration::from_millis(16))?;
                wayland_state.draw(
                    &qh,
                    renderer.buffer(),
                    &bar_state.cookbook,
                    config
                        .style
                        .window_bg
                        .as_deref()
                        .unwrap_or(&config.style.bg),
                )?;
                // trace!("Frame rendered"); // Too noisy
            }
        }

        event_queue
            .blocking_dispatch(&mut wayland_state)
            .context("Wayland dispatch failed")?;

        // Process Input Events
        // We do this after dispatch to handle events received this turn
        for event in wayland_state.input_events.drain(..) {
            let char_w = wayland_state.text_renderer.char_width as f64;
            let char_h = wayland_state.text_renderer.char_height as f64;

            // Extract pixel coordinates if present, else use last known cursor pos
            let (px, py) = match event {
                DishEvent::Motion { x, y } | DishEvent::Click { x, y, .. } => (x as f64, y as f64),
                _ => (wayland_state.cursor_x, wayland_state.cursor_y),
            };

            // Convert to Cell Coordinates
            if char_w > 0.0 && char_h > 0.0 {
                let cx = (px / char_w) as u16;
                let cy = (py / char_h) as u16;

                // Create a Cell-based event
                let mut cell_event = event;
                match &mut cell_event {
                    DishEvent::Motion { x, y } | DishEvent::Click { x, y, .. } => {
                        *x = cx;
                        *y = cy;
                    }
                    _ => {} // Leave/Enter/Scroll don't carry x/y in the enum variant usually (except scroll maybe?)
                }

                renderer.process_input(cx, cy, cell_event);
            }
        }
    }

    Ok(())
}
