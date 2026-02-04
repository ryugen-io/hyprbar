use anyhow::{Context, Result};
use hyprink::config::Config;
use hyprlog;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

pub fn handle_autostart(config_ink: &Arc<Config>) -> Result<()> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_dir = PathBuf::from(home).join(".config").join("hyprbar");

    // Ensure config dir exists
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    }

    let script_path = config_dir.join("autostart.sh");
    debug!("Checking autostart script at {:?}", script_path);

    if script_path.exists() {
        debug!("Autostart script exists. Removing...");
        fs::remove_file(&script_path).context("Failed to remove autostart script")?;

        let msg = config_ink
            .layout
            .labels
            .get("bar_autostart_disabled")
            .cloned()
            .unwrap_or_else(|| "autostart script removed".to_string());
        hyprlog::internal::info("AUTOSTART", &msg);
    } else {
        debug!("Autostart script does not exist. Creating...");
        // Simple shell script to start the daemon
        let content = "#!/bin/sh\n# Hyprbar Autostart\n# Add this script to your window manager's startup\n\nhyprbar --start\n";
        fs::write(&script_path, content).context("Failed to write autostart script")?;

        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;

        let msg = config_ink
            .layout
            .labels
            .get("bar_autostart_enabled")
            .cloned()
            .unwrap_or_else(|| "autostart script created".to_string());
        hyprlog::internal::info("AUTOSTART", &msg);

        // Help message
        hyprlog::internal::info("AUTOSTART", &format!("Script location: {:?}", script_path));
        hyprlog::internal::info(
            "AUTOSTART",
            "Example for Hyprland: exec-once = ~/.config/hyprbar/autostart.sh",
        );
    }

    Ok(())
}
