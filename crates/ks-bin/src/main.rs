use anyhow::{Context, Result};
use clap::Parser;
use k_lib::config::Cookbook;
use k_lib::logger;
use tracing::debug;

mod modules;
mod plugin_loader;
mod tui;

use modules::build;
use modules::cli::{Cli, Commands};
use modules::config;
use modules::daemon;
use modules::install;
use modules::logging;
use modules::runner;
use modules::watcher;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Load Kitchn Config (Global Styles) - Essential for logging everywhere
    let cookbook = std::sync::Arc::new(Cookbook::load().context("Failed to load kitchn cookbook")?);

    // 0. Handle Detached Debug Mode
    // Only if debug is on, no subcommand, AND no action flags (start/stop/restart/autostart)
    let has_action_flag = cli.start || cli.stop || cli.restart || cli.autostart;
    if cli.debug && cli.command.is_none() && !has_action_flag {
        // If debug is on and no subcommand, we spawn the daemon and the viewer, then exit
        daemon::spawn_bar_daemon(&cookbook)
            .await
            .context("Failed to spawn bar daemon")?;
        daemon::spawn_debug_viewer(&cookbook).context("Failed to spawn debug viewer")?; // Not async

        let msg = cookbook
            .dictionary
            .presets
            .get("sink_debug_started")
            .map(|p| p.msg.clone())
            .unwrap_or_else(|| "debug mode started".to_string());
        logger::log_to_terminal(
            &cookbook,
            "info",
            "SINK",
            &format!("{} Socket: {:?}", msg, config::get_socket_path()),
        );
        return Ok(());
    }

    // 2. Load Sink Config (App Layout & Logging)
    let sink_config = config::load_sink_config(&cookbook);

    // 3. Initialize Logging (after config load)
    logging::init_logging(
        cookbook.clone(),
        cli.debug,
        &sink_config.logging.level,
        &sink_config.logging.debug_filter,
        matches!(cli.command, Some(Commands::InternalRun)), // Only bind socket if we are the daemon
    )?;

    // 4. Handle Top-Level Flags (start, stop, restart, autostart)
    // These take precedence over subcommands for daemon control
    if cli.start {
        debug!("Handling --start flag -> spawning daemon");
        daemon::spawn_bar_daemon(&cookbook)
            .await
            .context("Failed to spawn bar daemon")?;
        return Ok(());
    }
    if cli.stop {
        daemon::terminate_bar_daemon(&cookbook)
            .await
            .context("Failed to terminate bar daemon")?;
        return Ok(());
    }
    if cli.restart {
        debug!("Handling --restart flag -> restarting daemon");
        daemon::restart_bar_daemon(&cookbook)
            .await
            .context("Failed to restart bar daemon")?;
        return Ok(());
    }
    if cli.autostart {
        debug!("Handling --autostart flag");
        modules::autostart::handle_autostart(&cookbook)?;
        return Ok(());
    }

    // 5. Handle Subcommands
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::InternalWatch { socket_path } => {
                return watcher::run_watcher(socket_path).await;
            }
            Commands::InternalRun => {
                // This is the actual bar process, run in foreground
                let start_msg = cookbook
                    .dictionary
                    .presets
                    .get("sink_startup")
                    .map(|p| p.msg.clone())
                    .unwrap_or_else(|| "kitchnsink starting...".to_string());
                logger::log_to_terminal(&cookbook, "info", "SINK", &start_msg);
                return runner::run_server(cookbook, sink_config).await;
            }
            Commands::Wash { path } => return build::wash_dish(path, &cookbook).await,
            Commands::Load { path } => return install::load_dish(path, &cookbook).await,
            Commands::List => {
                use crate::modules::registry::Registry;
                let registry = Registry::load()?;

                let header_msg = cookbook
                    .dictionary
                    .presets
                    .get("sink_plugins_header")
                    .map(|p| p.msg.clone())
                    .unwrap_or_else(|| "Installed Plugins:".to_string());
                logger::log_to_terminal(&cookbook, "info", "PLUGINS", &header_msg);

                for (name, entry) in registry.plugins {
                    let status_msg_key = if entry.enabled {
                        "sink_plugin_status_enabled"
                    } else {
                        "sink_plugin_status_disabled"
                    };
                    let status = cookbook
                        .dictionary
                        .presets
                        .get(status_msg_key)
                        .map(|p| p.msg.clone())
                        .unwrap_or_else(|| {
                            if entry.enabled {
                                "enabled".to_string()
                            } else {
                                "disabled".to_string()
                            }
                        });

                    let entry_msg = cookbook
                        .dictionary
                        .presets
                        .get("sink_plugin_entry")
                        .map(|p| {
                            p.msg
                                .replace("{name}", &name)
                                .replace("{status}", &status)
                                .replace("{path}", &entry.path.display().to_string())
                        })
                        .unwrap_or_else(|| {
                            format!("- {}: {} ({})", name, status, entry.path.display())
                        });
                    logger::log_to_terminal(&cookbook, "info", "PLUGINS", &entry_msg);

                    if !entry.metadata.version.is_empty() {
                        let meta_msg = cookbook
                            .dictionary
                            .presets
                            .get("sink_plugin_meta")
                            .map(|p| {
                                p.msg
                                    .replace("{version}", &entry.metadata.version)
                                    .replace("{author}", &entry.metadata.author)
                            })
                            .unwrap_or_else(|| {
                                format!(
                                    "  v{} by {}",
                                    entry.metadata.version, entry.metadata.author
                                )
                            });
                        logger::log_to_terminal(&cookbook, "info", "PLUGINS", &meta_msg);

                        let desc_msg = cookbook
                            .dictionary
                            .presets
                            .get("sink_plugin_desc")
                            .map(|p| p.msg.replace("{description}", &entry.metadata.description))
                            .unwrap_or_else(|| format!("  {}", entry.metadata.description));
                        logger::log_to_terminal(&cookbook, "info", "PLUGINS", &desc_msg);
                    }
                }
                return Ok(());
            }
            Commands::Enable { name } => {
                use crate::modules::registry::Registry;
                let mut registry = Registry::load()?;
                registry.enable(name)?;
                logger::log_to_terminal(
                    &cookbook,
                    "info",
                    "PLUGINS",
                    &format!("plugin '{}' enabled", name),
                );
                return Ok(());
            }
            Commands::Disable { name } => {
                use crate::modules::registry::Registry;
                let mut registry = Registry::load()?;
                registry.disable(name)?;
                logger::log_to_terminal(
                    &cookbook,
                    "info",
                    "PLUGINS",
                    &format!("plugin '{}' disabled", name),
                );
                return Ok(());
            }
        }
    }

    // 6. Default Action: No flags or subcommands means launch TUI
    // TUI manager
    // Pass sink_config which contains the theme styles
    tui::run_tui(sink_config).map_err(|e| anyhow::anyhow!("TUI error: {}", e))
}
