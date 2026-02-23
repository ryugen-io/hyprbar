use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "hyprbar", version, about = "Wayland status bar for Hyprland")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable debug mode with verbose logging in a separate terminal
    #[arg(long, global = true)]
    pub debug: bool,

    /// Start the bar daemon process
    #[arg(long)]
    pub start: bool,

    /// Stop the running bar daemon process
    #[arg(long)]
    pub stop: bool,

    /// Restart the bar daemon process
    #[arg(long)]
    pub restart: bool,

    /// Toggle native Hyprland autostart integration
    #[arg(long)]
    pub autostart: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Internal command to watch logs via socket (Hidden)
    #[command(hide = true)]
    InternalWatch { socket_path: PathBuf },

    /// Internal command to run the bar process (Hidden)
    #[command(hide = true)]
    InternalRun,

    /// Compile a .rs widget file into a .so plugin
    #[command(alias = "c")]
    Compile {
        /// Path to the .rs file
        path: PathBuf,
    },

    /// Install a .so widget plugin
    #[command(alias = "i")]
    Install {
        /// Path to the .so file
        path: PathBuf,
    },

    /// List installed plugins
    #[command(alias = "ls")]
    List,

    /// Enable a plugin
    #[command(alias = "on")]
    Enable {
        /// Name of the plugin
        name: String,
    },

    /// Disable a plugin
    #[command(alias = "off")]
    Disable {
        /// Name of the plugin
        name: String,
    },
}
