use crate::modules::config::get_socket_path;
use anyhow::{Context, Result};
use std::env;
use std::process::Stdio;
use tokio::process::Command;

pub fn spawn_bar_daemon() -> Result<()> {
    let self_exe = env::current_exe().context("Failed to get current executable path")?;

    // Spawn self with internal-run and debug flag
    // tokio::process::Command::spawn is synchronous but returns a Future-aware Child.
    Command::new(self_exe)
        .arg("internal-run")
        .arg("--debug")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn background bar process")?;

    Ok(())
}

pub fn spawn_debug_viewer() -> Result<()> {
    let socket_path = get_socket_path();

    // Detect terminal
    let terminal = env::var("TERMINAL").ok().or_else(|| {
        let terminals = ["rio", "alacritty", "kitty", "gnome-terminal", "xterm"];
        for term in terminals {
            if which::which(term).is_ok() {
                return Some(term.to_string());
            }
        }
        None
    });

    if let Some(term) = terminal {
        let self_exe = env::current_exe().context("Failed to get current executable path")?;

        // Spawn terminal running our internal watch command
        Command::new(&term)
            .arg("-e")
            .arg(&self_exe)
            .arg("internal-watch")
            .arg(&socket_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn debug terminal")?;
    } else {
        println!("No compatible terminal found for debug mode.");
    }

    Ok(())
}
