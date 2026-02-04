use anyhow::{Context, Result};

use crate::modules::bootstrap;
use crate::modules::logging::{log_debug, log_info};
use crate::modules::wayland_integration;

use crate::config::BarConfig;
use hyprink::config::Config;
use std::sync::Arc;

pub async fn run_server(initial_config_ink: Arc<Config>, initial_config: BarConfig) -> Result<()> {
    // Initialize application components using the bootstrap module
    let (config_ink, config, bar_state, _plugin_manager, mut renderer) =
        bootstrap::init_application(initial_config_ink, initial_config)
            .await
            .context("Failed to bootstrap application")?;

    // Pre-fetch log strings (Config consumed later)
    let get_msg = |key: &str, default: &str| -> String {
        config_ink
            .layout
            .labels
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    };

    let msg_loop = get_msg("bar_start_loop", "Starting Wayland event loop");
    let msg_exit = get_msg("bar_exit", "Exiting...");

    // 6. Initialize Wayland & Smart Scaling
    let (mut wayland_state, mut event_queue, _layer_surface) =
        wayland_integration::init_wayland_integration(&config)
            .await
            .context("Failed to initialize Wayland integration")?;

    // 7. Event Loop
    log_info("BAR", &msg_loop);
    log_debug(
        "WAYLAND",
        &format!(
            "Configured: {}x{}",
            wayland_state.width, wayland_state.height
        ),
    );

    loop {
        wayland_integration::handle_wayland_events(
            &mut wayland_state,
            &mut event_queue,
            &mut renderer,
            &bar_state,
            &config,
        )?;

        if wayland_state.exit {
            log_info("BAR", &msg_exit);
            break;
        }
    }

    Ok(())
}
