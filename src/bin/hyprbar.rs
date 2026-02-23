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

    // Config must load first — all logging, labels, and theme resolution depend on it.
    let config_ink = std::sync::Arc::new(Config::load().context("Failed to load config")?);

    // Debug without subcommand = "launch daemon + viewer", not "run in foreground".
    // Action flags (start/stop/restart/autostart) take precedence over this shortcut.
    let has_action_flag = cli.start || cli.stop || cli.restart || cli.autostart;
    if cli.debug && cli.command.is_none() && !has_action_flag {
        daemon::spawn_bar_daemon(&config_ink, true)
            .await
            .context("Failed to spawn bar daemon")?;
        daemon::spawn_debug_viewer(&config_ink).context("Failed to spawn debug viewer")?;

        let msg = config_ink
            .layout
            .labels
            .get("bar_debug_started")
            .cloned()
            .unwrap_or_else(|| "debug mode started".to_string());
        log_info(
            "BAR",
            &format!("{} Socket: {:?}", msg, config::get_socket_path()),
        );
        return Ok(());
    }

    let bar_config = config::load_bar_config(&config_ink);

    // Logging depends on bar_config for level/filter settings.
    logging::init_logging(
        config_ink.clone(),
        cli.debug,
        &bar_config.logging.level,
        &bar_config.logging.debug_filter,
        matches!(cli.command, Some(Commands::InternalRun)), // only the daemon binds the debug socket
    )?;

    // Flags take precedence over subcommands — "hyprbar --start compile foo"
    // should start the daemon, not compile.
    if cli.start {
        log_debug("CLI", "Handling --start flag -> spawning daemon");
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
        log_debug("CLI", "Handling --restart flag -> restarting daemon");
        daemon::restart_bar_daemon(&config_ink, cli.debug)
            .await
            .context("Failed to restart bar daemon")?;
        if cli.debug {
            daemon::spawn_debug_viewer(&config_ink)?;
        }
        return Ok(());
    }
    if cli.autostart {
        log_debug("CLI", "Handling --autostart flag");
        autostart::handle_autostart(&config_ink)?;
        return Ok(());
    }

    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::InternalWatch { socket_path } => {
                return watcher::run_watcher(socket_path).await;
            }
            Commands::InternalRun => {
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
                log_info("PLUGINS", &header_msg);

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
                    log_info("PLUGINS", &entry_msg);

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
                        log_info("PLUGINS", &meta_msg);

                        let desc_msg = config_ink
                            .layout
                            .labels
                            .get("bar_plugin_desc")
                            .map(|p| p.replace("{description}", &entry.metadata.description))
                            .unwrap_or_else(|| format!("  {}", entry.metadata.description));
                        log_info("PLUGINS", &desc_msg);
                    }
                }
                return Ok(());
            }
            Commands::Enable { name } => {
                use hyprbar::modules::registry::Registry;
                let mut registry = Registry::load()?;
                registry.enable(name)?;
                log_info("PLUGINS", &format!("plugin '{}' enabled", name));
                return Ok(());
            }
            Commands::Disable { name } => {
                use hyprbar::modules::registry::Registry;
                let mut registry = Registry::load()?;
                registry.disable(name)?;
                log_info("PLUGINS", &format!("plugin '{}' disabled", name));
                return Ok(());
            }
        }
    }

    // No flags, no subcommands — interactive TUI is the default UX.
    tui::run_tui(bar_config).map_err(|e| anyhow::anyhow!("TUI error: {}", e))
}
