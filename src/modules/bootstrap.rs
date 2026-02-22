use anyhow::Result;
use hyprink::config::Config;
use std::sync::Arc;
use tokio::signal::unix::{SignalKind, signal};
use tokio::time::Duration;

use crate::modules::logging::*;
use crate::{config::BarConfig, renderer::BarRenderer, state::BarState};

use crate::plugin_loader::PluginManager;

pub async fn init_application(
    config_ink: Arc<Config>,
    config: BarConfig,
) -> Result<(Arc<Config>, BarConfig, BarState, PluginManager, BarRenderer)> {
    log_debug("BOOTSTRAP", "Starting application initialization");

    // Config labels are fetched now because the Arc is shared and the signal task
    // needs owned Strings — can't borrow from config_ink across the spawn boundary.
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

    log_debug("BOOTSTRAP", "Signal handlers configured");

    // Brief delay before exit gives in-flight Wayland frames time to finish rendering.
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

    log_debug("BOOTSTRAP", "Initializing bar state");
    let bar_state = BarState::new(config_ink.clone(), config.clone());

    log_debug("PLUGINS", "Initializing plugin manager");
    let mut plugin_manager = PluginManager::new();

    // XDG data dir is the canonical location for user-installed widget .so files.
    if let Some(data_dir) = dirs::data_local_dir() {
        let widgets_dir = data_dir.join("hyprbar/widgets");
        log_debug(
            "PLUGINS",
            &format!("Scanning for plugins in {:?}", widgets_dir),
        );

        if widgets_dir.exists() {
            match std::fs::read_dir(&widgets_dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|ext| ext == "so") {
                            log_debug("PLUGINS", &format!("Loading plugin: {:?}", path));
                            if let Err(e) = plugin_manager.load_plugin(&path, true, true) {
                                log_error("PLUGINS", &format!("Failed to load {:?}: {}", path, e));
                            }
                        }
                    }
                }
                Err(e) => {
                    log_warn("PLUGINS", &format!("Cannot read widgets dir: {}", e));
                }
            }
        } else {
            log_warn(
                "PLUGINS",
                &format!("Widgets directory does not exist: {:?}", widgets_dir),
            );
        }
    } else {
        log_error("PLUGINS", "Cannot determine local data directory");
    }

    log_debug("RENDER", "Initializing renderer");
    let renderer = BarRenderer::new(
        100, // Placeholder — real width arrives in the first Wayland configure event.
        config.window.height as u16,
        &config,
        &bar_state.config_ink,
        &plugin_manager,
    );

    log_info("BOOTSTRAP", "Application initialization complete");
    Ok((config_ink, config, bar_state, plugin_manager, renderer))
}
