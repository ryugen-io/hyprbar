use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "kitchnsink", version, about = "Kitchn Sink Wayland Bar")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable debug mode with verbose logging in a separate terminal
    #[arg(long, global = true)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Internal command to watch logs via socket (Hidden)
    #[command(hide = true)]
    InternalWatch { socket_path: PathBuf },

    /// Internal command to run the bar process (Hidden)
    #[command(hide = true)]
    InternalRun,

    /// Manage the bar (TUI)
    #[command(alias = "m")]
    Manage,

    /// Compile a .rs dish file into a .dish plugin
    #[command(alias = "w")]
    Wash {
        /// Path to the .rs file
        path: PathBuf,
    },

    /// Load/Install a .dish plugin
    #[command(alias = "l")]
    Load {
        /// Path to the .dish file
        path: PathBuf,
    },

    /// List installed plugins
    #[command(alias = "ls")]
    List,

    /// Enable a plugin
    #[command(alias = "on")]
    Enable {
        /// Name of the plugin (e.g. "battery.dish")
        name: String,
    },

    /// Disable a plugin
    #[command(alias = "off")]
    Disable {
        /// Name of the plugin
        name: String,
    },
}
