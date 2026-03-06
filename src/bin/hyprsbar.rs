use anyhow::{Context, Result};
use clap::Parser;

use hyprsbar::modules::autostart;
use hyprsbar::modules::build;
use hyprsbar::modules::cli::{Cli, Commands};
use hyprsbar::modules::config;
use hyprsbar::modules::daemon;
use hyprsbar::modules::install;
use hyprsbar::modules::logging::{self, *};
use hyprsbar::modules::runner;
use hyprsbar::modules::watcher;
use hyprsbar::tui;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Config must load first — all logging, labels, and theme resolution depend on it.
    let bar_config = config::load_bar_config();

    // Debug without subcommand = "launch daemon + viewer", not "run in foreground".
    // Action flags (start/stop/restart/autostart) take precedence over this shortcut.
    let has_action_flag = cli.start || cli.stop || cli.restart || cli.autostart;
    if cli.debug && cli.command.is_none() && !has_action_flag {
        daemon::spawn_bar_daemon(&bar_config, true)
            .await
            .context("Failed to spawn bar daemon")?;
        daemon::spawn_debug_viewer(&bar_config).context("Failed to spawn debug viewer")?;

        let msg = bar_config.label("bar_debug_started", "debug mode started");
        log_info(
            "BAR",
            &format!("{} Socket: {:?}", msg, config::get_socket_path()),
        );
        return Ok(());
    }

    // Logging depends on bar_config for level/filter settings.
    logging::init_logging(
        cli.debug,
        &bar_config.logging.level,
        &bar_config.logging.debug_filter,
        matches!(cli.command, Some(Commands::InternalRun)), // only the daemon binds the debug socket
    )?;

    // Flags take precedence over subcommands — "hyprsbar --start compile foo"
    // should start the daemon, not compile.
    if cli.start {
        log_debug("CLI", "Handling --start flag -> spawning daemon");
        daemon::spawn_bar_daemon(&bar_config, cli.debug)
            .await
            .context("Failed to spawn bar daemon")?;
        if cli.debug {
            daemon::spawn_debug_viewer(&bar_config)?;
        }
        return Ok(());
    }
    if cli.stop {
        daemon::terminate_bar_daemon(&bar_config)
            .await
            .context("Failed to terminate bar daemon")?;
        return Ok(());
    }
    if cli.restart {
        log_debug("CLI", "Handling --restart flag -> restarting daemon");
        daemon::restart_bar_daemon(&bar_config, cli.debug)
            .await
            .context("Failed to restart bar daemon")?;
        if cli.debug {
            daemon::spawn_debug_viewer(&bar_config)?;
        }
        return Ok(());
    }
    if cli.autostart {
        log_debug("CLI", "Handling --autostart flag");
        autostart::handle_autostart(&bar_config)?;
        return Ok(());
    }

    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::InternalWatch { socket_path } => {
                return watcher::run_watcher(socket_path).await;
            }
            Commands::InternalRun => {
                let start_msg = bar_config.label("bar_startup", "hyprsbar starting...");
                log_info("BAR", start_msg);
                return runner::run_server(bar_config).await;
            }
            Commands::Compile { path } => return build::compile_widget(path).await,
            Commands::Install { path } => return install::install_widget(path).await,
            Commands::List => {
                use hyprsbar::modules::registry::Registry;
                let registry = Registry::load()?;

                let header_msg = bar_config.label("bar_plugins_header", "Installed Plugins:");
                log_info("PLUGINS", header_msg);

                for (name, entry) in registry.plugins {
                    let status_msg_key = if entry.enabled {
                        "bar_plugin_status_enabled"
                    } else {
                        "bar_plugin_status_disabled"
                    };
                    let status_default = if entry.enabled { "enabled" } else { "disabled" };
                    let status = bar_config.label(status_msg_key, status_default);

                    let entry_msg = bar_config
                        .label("bar_plugin_entry", "- {name}: {status} ({path})")
                        .replace("{name}", &name)
                        .replace("{status}", status)
                        .replace("{path}", &entry.path.display().to_string());
                    log_info("PLUGINS", &entry_msg);

                    if !entry.metadata.version.is_empty() {
                        let meta_msg = bar_config
                            .label("bar_plugin_meta", "  v{version} by {author}")
                            .replace("{version}", &entry.metadata.version)
                            .replace("{author}", &entry.metadata.author);
                        log_info("PLUGINS", &meta_msg);

                        let desc_msg = bar_config
                            .label("bar_plugin_desc", "  {description}")
                            .replace("{description}", &entry.metadata.description);
                        log_info("PLUGINS", &desc_msg);
                    }
                }
                return Ok(());
            }
            Commands::Enable { name } => {
                use hyprsbar::modules::registry::Registry;
                let mut registry = Registry::load()?;
                registry.enable(name)?;
                log_info("PLUGINS", &format!("plugin '{}' enabled", name));
                return Ok(());
            }
            Commands::Disable { name } => {
                use hyprsbar::modules::registry::Registry;
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
