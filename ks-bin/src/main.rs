use anyhow::{Context, Result};
use kitchn_lib::config::Cookbook;
use kitchn_lib::logger;
use ks_core::config::SinkConfig;
use ks_core::renderer::BarRenderer;
use ks_core::state::BarState;
use ks_wayland::init as init_wayland;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

fn load_sink_config() -> SinkConfig {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_path = PathBuf::from(home)
        .join(".config")
        .join("kitchnsink")
        .join("sink.toml"); // User requested "sink.toml"

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(cfg) => return cfg,
                Err(e) => eprintln!("Failed to parse config: {}", e),
            },
            Err(e) => eprintln!("Failed to read config: {}", e),
        }
    }

    // Fallback
    SinkConfig::default()
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load Kitchn Config (Global Styles)
    let cookbook = Cookbook::load().context("Failed to load kitchn cookbook")?;

    // 2. Load Sink Config (App Layout)
    let config = load_sink_config();

    // 3. Initialize Logging
    logger::log_to_terminal(&cookbook, "info", "SINK", "kitchnsink starting...");

    // 4. Initialize Bar State
    let mut bar_state = BarState::new(cookbook, config.clone());

    // 5. Initialize Renderer (Offscreen)
    // Initial height from config
    let mut renderer = BarRenderer::new(100, config.window.height as u16);

    // 6. Initialize Wayland
    let (mut wayland_state, mut event_queue, _layer_surface) =
        init_wayland(config.window.height, config.window.anchor == "bottom")
            .context("Failed to initialize Wayland")?;
    let qh = event_queue.handle();

    // 6. Event Loop
    logger::log_to_terminal(
        &bar_state.cookbook,
        "info",
        "SINK",
        "Starting Wayland event loop",
    );

    loop {
        // Exit check
        if wayland_state.exit {
            logger::log_to_terminal(&bar_state.cookbook, "info", "SINK", "Exiting...");
            break;
        }

        // Handle resize/rendering BEFORE blocking for new events
        // This ensures the initial frame (triggered by init) is drawn immediately
        if wayland_state.configured {
            let width = wayland_state.width as u16;
            let height = wayland_state.height as u16;

            // Resize if needed
            if width > 0 && height > 0 && (renderer.width != width || renderer.height != height) {
                renderer.resize(width, height);
            }

            // Render if requested
            if wayland_state.redraw_requested {
                // Mock data update (todo: real stats)
                bar_state.cpu = 12.5;
                bar_state.mem = 45.2;

                // Render to buffer
                renderer.render_frame(&bar_state, Duration::from_millis(16))?;

                // Draw to Wayland
                wayland_state.draw(&qh, renderer.buffer(), &bar_state.cookbook)?;

                // Note: draw() requests the next frame callback and sets redraw_requested = false
            }
        }

        // Wait for next event (VSync or input)
        // blocking_dispatch flushes the output buffer (commits) and blocks on input
        event_queue
            .blocking_dispatch(&mut wayland_state)
            .context("Wayland dispatch failed")?;
    }

    Ok(())
}
