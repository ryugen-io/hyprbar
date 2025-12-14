use anyhow::{Context, Result};

use crate::modules::bootstrap;
use crate::modules::wayland_integration; // Import the new wayland integration module

use k_lib::config::Cookbook;
use ks_core::config::SinkConfig;
use std::sync::Arc;

pub async fn run_server(initial_cookbook: Arc<Cookbook>, initial_config: SinkConfig) -> Result<()> {
    // Initialize application components using the bootstrap module
    let (cookbook, config, bar_state, _plugin_manager, mut renderer) =
        bootstrap::init_application(initial_cookbook, initial_config)
            .await
            .context("Failed to bootstrap application")?;

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

    // 6. Initialize Wayland & Smart Scaling
    let (mut wayland_state, mut event_queue, _layer_surface) =
        wayland_integration::init_wayland_integration(&config)
            .await
            .context("Failed to initialize Wayland integration")?;

    // 7. Event Loop
    // Use logger to match system theme
    k_lib::logger::log_to_terminal(&bar_state.cookbook, "info", "SINK", &msg_loop);

    loop {
        wayland_integration::handle_wayland_events(
            &mut wayland_state,
            &mut event_queue,
            &mut renderer,
            &bar_state,
            &config,
        )?;

        if wayland_state.exit {
            k_lib::logger::log_to_terminal(&bar_state.cookbook, "info", "SINK", &msg_exit);
            break;
        }
    }

    Ok(())
}
