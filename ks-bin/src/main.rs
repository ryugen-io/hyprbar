use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use k_lib::config::Cookbook;
use k_lib::logger;

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

    // 0. Handle Detached Debug Mode
    if cli.debug && cli.command.is_none() {
        // If debug is on and no subcommand, we spawn the daemon and the viewer, then exit
        daemon::spawn_bar_daemon().context("Failed to spawn bar daemon")?;
        daemon::spawn_debug_viewer().context("Failed to spawn debug viewer")?;

        println!("Debug Mode Started.");
        println!("Socket: {:?}", config::get_socket_path());
        return Ok(());
    }

    // 1. Load Kitchn Config (Global Styles)
    let cookbook = std::sync::Arc::new(Cookbook::load().context("Failed to load kitchn cookbook")?);

    // 2. Load Sink Config (App Layout & Logging)
    let sink_config = config::load_sink_config(&cookbook);

    // 3. Initialize Logging (after config load)
    logging::init_logging(
        cookbook.clone(),
        cli.debug,
        &sink_config.logging.level,
        &sink_config.logging.debug_filter,
    )?;

    // 4. Handle Subcommands
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::InternalWatch { socket_path } => {
                return watcher::run_watcher(socket_path).await;
            }
            Commands::Manage => {
                // TUI manager
                // Pass sink_config which contains the theme styles
                return tui::run_tui(sink_config).map_err(|e| anyhow::anyhow!("TUI error: {}", e));
            }
            Commands::InternalRun => {
                // Determine we should run the bar (fallthrough)
            }
            Commands::Wash { path } => return build::wash_dish(path, &cookbook).await,
            Commands::Load { path } => return install::load_dish(path, &cookbook).await,
            Commands::List => {
                use crate::modules::registry::Registry;
                let registry = Registry::load()?;
                println!("Installed Plugins:");
                for (name, entry) in registry.plugins {
                    let status = if entry.enabled {
                        "Enabled".green()
                    } else {
                        "Disabled".red()
                    };
                    println!("  - {}: {} ({})", name, status, entry.path.display());
                    if !entry.metadata.version.is_empty() {
                        println!(
                            "    v{} by {}",
                            entry.metadata.version, entry.metadata.author
                        );
                        println!("    {}", entry.metadata.description);
                    }
                }
                return Ok(());
            }
            Commands::Enable { name } => {
                use crate::modules::registry::Registry;
                let mut registry = Registry::load()?;
                registry.enable(name)?;
                println!("Plugin '{}' enabled.", name);
                return Ok(());
            }
            Commands::Disable { name } => {
                use crate::modules::registry::Registry;
                let mut registry = Registry::load()?;
                registry.disable(name)?;
                println!("Plugin '{}' disabled.", name);
                return Ok(());
            }
        }
    }

    let start_msg = cookbook
        .dictionary
        .presets
        .get("sink_startup")
        .map(|p| p.msg.clone())
        .unwrap_or_else(|| "kitchnsink starting...".to_string());

    logger::log_to_terminal(&cookbook, "info", "SINK", &start_msg);

    // 4. Run Server
    runner::run_server(cookbook, sink_config).await
}
