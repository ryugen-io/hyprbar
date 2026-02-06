use anyhow::{Context, Result};
use clap::Parser;
use hyprink::config::Config;

use hyprbar::modules::autostart;
use hyprbar::modules::build;
use hyprbar::modules::cli::{Cli, Commands};
use hyprbar::modules::config;
use hyprbar::modules::daemon;
use hyprbar::modules::install;
use hyprbar::modules::logging::{self, *};
use hyprbar::modules::runner;
use hyprbar::modules::watcher;
use hyprbar::tui;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Load Config (Global Styles) - Essential for logging everywhere
    let config_ink = std::sync::Arc::new(Config::load().context("Failed to load config")?);

    // 0. Handle Detached Debug Mode
    // Only if debug is on, no subcommand, AND no action flags (start/stop/restart/autostart)
    let has_action_flag = cli.start || cli.stop || cli.restart || cli.autostart;
    if cli.debug && cli.command.is_none() && !has_action_flag {
        // If debug is on and no subcommand, we spawn the daemon and the viewer, then exit
        daemon::spawn_bar_daemon(&config_ink, true)
            .await
            .context("Failed to spawn bar daemon")?;
        daemon::spawn_debug_viewer(&config_ink).context("Failed to spawn debug viewer")?; // Not async

        let msg = config_ink
            .layout
            .labels
            .get("bar_debug_started")
            .cloned()
            .unwrap_or_else(|| "debug mode started".to_string());
        hyprlog::internal::info(
            "BAR",
            &format!("{} Socket: {:?}", msg, config::get_socket_path()),
        );
        return Ok(());
    }

    // 2. Load Bar Config (App Layout & Logging)
    let bar_config = config::load_bar_config(&config_ink);

    // 3. Initialize Logging (after config load)
    logging::init_logging(
        config_ink.clone(),
        cli.debug,
        &bar_config.logging.level,
        &bar_config.logging.debug_filter,
        matches!(cli.command, Some(Commands::InternalRun)), // Only bind socket if we are the daemon
    )?;

    // 4. Handle Top-Level Flags (start, stop, restart, autostart)
    // These take precedence over subcommands for daemon control
    if cli.start {
        hyprlog::internal::debug("CLI", "Handling --start flag -> spawning daemon");
        daemon::spawn_bar_daemon(&config_ink, cli.debug)
            .await
            .context("Failed to spawn bar daemon")?;
        if cli.debug {
            daemon::spawn_debug_viewer(&config_ink)?;
        }
        return Ok(());
    }
    if cli.stop {
        daemon::terminate_bar_daemon(&config_ink)
            .await
            .context("Failed to terminate bar daemon")?;
        return Ok(());
    }
    if cli.restart {
        hyprlog::internal::debug("CLI", "Handling --restart flag -> restarting daemon");
        daemon::restart_bar_daemon(&config_ink, cli.debug)
            .await
            .context("Failed to restart bar daemon")?;
        if cli.debug {
            daemon::spawn_debug_viewer(&config_ink)?;
        }
        return Ok(());
    }
    if cli.autostart {
        hyprlog::internal::debug("CLI", "Handling --autostart flag");
        autostart::handle_autostart(&config_ink)?;
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
                let start_msg = config_ink
                    .layout
                    .labels
                    .get("bar_startup")
                    .cloned()
                    .unwrap_or_else(|| "hyprbar starting...".to_string());
                log_info("BAR", &start_msg);
                return runner::run_server(config_ink, bar_config).await;
            }
            Commands::Compile { path } => return build::compile_widget(path, &config_ink).await,
            Commands::Install { path } => return install::install_widget(path, &config_ink).await,
            Commands::List => {
                use hyprbar::modules::registry::Registry;
                let registry = Registry::load()?;

                let header_msg = config_ink
                    .layout
                    .labels
                    .get("bar_plugins_header")
                    .cloned()
                    .unwrap_or_else(|| "Installed Plugins:".to_string());
                hyprlog::internal::info("PLUGINS", &header_msg);

                for (name, entry) in registry.plugins {
                    let status_msg_key = if entry.enabled {
                        "bar_plugin_status_enabled"
                    } else {
                        "bar_plugin_status_disabled"
                    };
                    let status = config_ink
                        .layout
                        .labels
                        .get(status_msg_key)
                        .cloned()
                        .unwrap_or_else(|| {
                            if entry.enabled {
                                "enabled".to_string()
                            } else {
                                "disabled".to_string()
                            }
                        });

                    let entry_msg = config_ink
                        .layout
                        .labels
                        .get("bar_plugin_entry")
                        .map(|p| {
                            p.replace("{name}", &name)
                                .replace("{status}", &status)
                                .replace("{path}", &entry.path.display().to_string())
                        })
                        .unwrap_or_else(|| {
                            format!("- {}: {} ({})", name, status, entry.path.display())
                        });
                    hyprlog::internal::info("PLUGINS", &entry_msg);

                    if !entry.metadata.version.is_empty() {
                        let meta_msg = config_ink
                            .layout
                            .labels
                            .get("bar_plugin_meta")
                            .map(|p| {
                                p.replace("{version}", &entry.metadata.version)
                                    .replace("{author}", &entry.metadata.author)
                            })
                            .unwrap_or_else(|| {
                                format!(
                                    "  v{} by {}",
                                    entry.metadata.version, entry.metadata.author
                                )
                            });
                        hyprlog::internal::info("PLUGINS", &meta_msg);

                        let desc_msg = config_ink
                            .layout
                            .labels
                            .get("bar_plugin_desc")
                            .map(|p| p.replace("{description}", &entry.metadata.description))
                            .unwrap_or_else(|| format!("  {}", entry.metadata.description));
                        hyprlog::internal::info("PLUGINS", &desc_msg);
                    }
                }
                return Ok(());
            }
            Commands::Enable { name } => {
                use hyprbar::modules::registry::Registry;
                let mut registry = Registry::load()?;
                registry.enable(name)?;
                hyprlog::internal::info("PLUGINS", &format!("plugin '{}' enabled", name));
                return Ok(());
            }
            Commands::Disable { name } => {
                use hyprbar::modules::registry::Registry;
                let mut registry = Registry::load()?;
                registry.disable(name)?;
                hyprlog::internal::info("PLUGINS", &format!("plugin '{}' disabled", name));
                return Ok(());
            }
        }
    }

    // 6. Default Action: No flags or subcommands means launch TUI
    // TUI manager
    // Pass bar_config which contains the theme styles
    tui::run_tui(bar_config).map_err(|e| anyhow::anyhow!("TUI error: {}", e))
}
