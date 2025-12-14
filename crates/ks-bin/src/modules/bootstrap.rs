use anyhow::Result;
use k_lib::config::Cookbook;
use log::error;
use std::sync::Arc;
use tokio::signal::unix::{SignalKind, signal};
use tokio::time::Duration;

use ks_lib::{config::SinkConfig, renderer::BarRenderer, state::BarState};

use crate::plugin_loader::PluginManager;

pub async fn init_application(
    cookbook: Arc<Cookbook>,
    config: SinkConfig,
) -> Result<(
    Arc<Cookbook>,
    SinkConfig,
    BarState,
    PluginManager,
    BarRenderer,
)> {
    // Pre-fetch log strings (Cookbook consumed later)
    let get_msg = |key: &str, default: &str| -> String {
        cookbook
            .dictionary
            .presets
            .get(key)
            .map(|p| p.msg.clone())
            .unwrap_or_else(|| default.to_string())
    };

    let msg_sigterm = get_msg(
        "sink_sigterm",
        "Received SIGTERM (Toggle), shutting down...",
    );
    let msg_sigint = get_msg("sink_sigint", "Received SIGINT, shutting down...");

    // Spawn Signal Handler
    let signal_cookbook = cookbook.clone();
    tokio::spawn(async move {
        use k_lib::logger;
        let mut term = signal(SignalKind::terminate()).unwrap();
        let mut int = signal(SignalKind::interrupt()).unwrap();

        tokio::select! {
            _ = term.recv() => logger::log_to_terminal(&signal_cookbook, "info", "SINK", &msg_sigterm),
            _ = int.recv() => logger::log_to_terminal(&signal_cookbook, "info", "SINK", &msg_sigint),
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(0);
    });

    // Initialize Bar State (now simpler)
    let bar_state = BarState::new(cookbook.clone(), config.clone());

    // Initialize Plugin Manager
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

    // Initialize Renderer
    let renderer = BarRenderer::new(
        100, // TODO: This needs to be dynamic based on screen width
        config.window.height as u16,
        &config,
        &bar_state.cookbook,
        &plugin_manager,
    );

    Ok((cookbook, config, bar_state, plugin_manager, renderer))
}
