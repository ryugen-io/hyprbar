use anyhow::{Context, Result};

use crate::modules::bootstrap;
use crate::modules::logging::*;
use crate::modules::wayland_integration;

use crate::config::BarConfig;
use hyprink::config::Config;
use std::sync::Arc;

pub async fn run_server(initial_config_ink: Arc<Config>, initial_config: BarConfig) -> Result<()> {
    log_debug("BAR", "Starting server initialization");

    let (config_ink, config, bar_state, _plugin_manager, mut renderer) =
        bootstrap::init_application(initial_config_ink, initial_config)
            .await
            .context("Failed to bootstrap application")?;

    log_debug("BAR", "Bootstrap complete");

    // Labels fetched now — config_ink is shared via Arc and these strings
    // are used after the event loop where borrowing would be inconvenient.
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

    log_debug("WAYLAND", "Initializing Wayland integration");
    let (mut wayland_state, mut event_queue, _layer_surface) =
        wayland_integration::init_wayland_integration(&config)
            .await
            .context("Failed to initialize Wayland integration")?;

    log_info("WAYLAND", "Wayland integration initialized");

    log_info("BAR", &msg_loop);
    log_debug(
        "WAYLAND",
        &format!(
            "Configured: {}x{}",
            wayland_state.width, wayland_state.height
        ),
    );

    loop {
        if let Err(e) = wayland_integration::handle_wayland_events(
            &mut wayland_state,
            &mut event_queue,
            &mut renderer,
            &bar_state,
            &config,
        ) {
            log_error("WAYLAND", &format!("Event handling error: {}", e));
            log_warn("BAR", "Attempting to continue after error");
            return Err(e);
        }

        if wayland_state.exit {
            log_info("BAR", &msg_exit);
            break;
        }
    }

    log_debug("BAR", "Server shutdown complete");
    Ok(())
}
