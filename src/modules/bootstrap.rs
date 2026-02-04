use anyhow::Result;
use hyprink::config::Config;
use std::sync::Arc;
use tokio::signal::unix::{SignalKind, signal};
use tokio::time::Duration;

use crate::modules::logging::{log_error, log_info};
use crate::{config::BarConfig, renderer::BarRenderer, state::BarState};

use crate::plugin_loader::PluginManager;

pub async fn init_application(
    config_ink: Arc<Config>,
    config: BarConfig,
) -> Result<(Arc<Config>, BarConfig, BarState, PluginManager, BarRenderer)> {
    // Pre-fetch log strings (Config consumed later)
    let get_msg = |key: &str, default: &str| -> String {
        config_ink
            .layout
            .labels
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    };

    let msg_sigterm = get_msg("bar_sigterm", "Received SIGTERM (Toggle), shutting down...");
    let msg_sigint = get_msg("bar_sigint", "Received SIGINT, shutting down...");

    // Spawn Signal Handler
    let _signal_config_ink = config_ink.clone();
    tokio::spawn(async move {
        let mut term = signal(SignalKind::terminate()).unwrap();
        let mut int = signal(SignalKind::interrupt()).unwrap();

        tokio::select! {
            _ = term.recv() => log_info("BAR", &msg_sigterm),
            _ = int.recv() => log_info("BAR", &msg_sigint),
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(0);
    });

    // Initialize Bar State (now simpler)
    let bar_state = BarState::new(config_ink.clone(), config.clone());

    // Initialize Plugin Manager
    let mut plugin_manager = PluginManager::new();

    // Load plugins from ~/.local/share/hyprbar/widgets
    if let Some(data_dir) = dirs::data_local_dir() {
        let widgets_dir = data_dir.join("hyprbar/widgets");
        if widgets_dir.exists()
            && let Ok(entries) = std::fs::read_dir(widgets_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "so")
                    && let Err(e) = plugin_manager.load_plugin(&path, true, true)
                {
                    log_error("PLUGINS", &format!("Failed to load {:?}: {}", path, e));
                }
            }
        }
    }

    // Initialize Renderer
    let renderer = BarRenderer::new(
        100, // TODO: This needs to be dynamic based on screen width
        config.window.height as u16,
        &config,
        &bar_state.config_ink,
        &plugin_manager,
    );

    Ok((config_ink, config, bar_state, plugin_manager, renderer))
}
