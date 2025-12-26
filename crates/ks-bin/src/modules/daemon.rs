use crate::modules::config::{get_pid_file_path, get_socket_path};
use anyhow::{Context, Result};
use k_lib::config::Cookbook;
use k_lib::logger;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use procfs::process::Process;
use std::env;
use std::fs;
use std::io::Write;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;

pub async fn spawn_bar_daemon(cookbook: &Arc<Cookbook>, debug: bool) -> Result<()> {
    let self_exe = env::current_exe().context("Failed to get current executable path")?;
    let pid_file_path = get_pid_file_path();

    // Check if daemon is already running
    if pid_file_path.exists()
        && let Ok(pid_str) = fs::read_to_string(&pid_file_path)
        && let Ok(pid) = pid_str.trim().parse::<u32>()
    {
        if let Ok(_proc) = Process::new(pid as i32) {
            // Use procfs here
            let msg = cookbook
                .dictionary
                .presets
                .get("sink_running")
                .map(|p| p.msg.replace("{pid}", &pid.to_string()))
                .unwrap_or_else(|| format!("daemon running (pid: {})", pid));
            logger::log_to_terminal(cookbook, "info", "DAEMON", &msg);
            return Ok(());
        } else {
            let msg = cookbook
                .dictionary
                .presets
                .get("sink_stale")
                .map(|p| p.msg.clone())
                .unwrap_or_else(|| "stale pid file cleaned".to_string());
            logger::log_to_terminal(cookbook, "warn", "DAEMON", &msg);
            fs::remove_file(&pid_file_path).ok(); // Ignore error if cannot remove
        }
    }
    // Spawn self with internal-run and debug flag
    // Spawn self with internal-run
    let mut command = Command::new(self_exe);
    command.arg("internal-run");

    if debug {
        command.arg("--debug");
    }

    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn background bar process")?;

    // Write PID to file
    let pid = child.id().context("Failed to get child PID")?;
    let mut file = fs::File::create(&pid_file_path)
        .context(format!("Failed to create PID file at {:?}", pid_file_path))?;
    writeln!(file, "{}", pid).context("Failed to write PID to file")?;

    // drop(child); // Explicit drop not strictly needed but safe

    let msg = cookbook
        .dictionary
        .presets
        .get("sink_start")
        .map(|p| p.msg.replace("{pid}", &pid.to_string()))
        .unwrap_or_else(|| format!("daemon started (pid: {})", pid));
    logger::log_to_terminal(cookbook, "info", "DAEMON", &msg);
    Ok(())
}

pub async fn terminate_bar_daemon(cookbook: &Arc<Cookbook>) -> Result<()> {
    let pid_file_path = get_pid_file_path();

    if pid_file_path.exists() {
        let pid_str = fs::read_to_string(&pid_file_path)
            .context(format!("Failed to read PID from {:?}", pid_file_path))?;
        let pid = pid_str
            .trim()
            .parse::<i32>()
            .context(format!("Failed to parse PID from '{}'", pid_str))?;

        // Send SIGTERM using nix
        signal::kill(Pid::from_raw(pid), Signal::SIGTERM)
            .context(format!("Failed to send SIGTERM to PID {}", pid))?;

        // Use tokio::time::sleep for async context
        sleep(Duration::from_millis(500)).await;

        fs::remove_file(&pid_file_path)
            .context(format!("Failed to remove PID file at {:?}", pid_file_path))?;

        let msg = cookbook
            .dictionary
            .presets
            .get("sink_stop")
            .map(|p| p.msg.replace("{pid}", &pid.to_string()))
            .unwrap_or_else(|| format!("daemon terminated (pid: {})", pid));
        logger::log_to_terminal(cookbook, "info", "DAEMON", &msg);
    } else {
        let msg = cookbook
            .dictionary
            .presets
            .get("sink_not_found")
            .map(|p| p.msg.clone())
            .unwrap_or_else(|| "no daemon found".to_string());
        logger::log_to_terminal(cookbook, "info", "DAEMON", &msg);
    }
    Ok(())
}

pub async fn restart_bar_daemon(cookbook: &Arc<Cookbook>, debug: bool) -> Result<()> {
    let msg = cookbook
        .dictionary
        .presets
        .get("sink_restart")
        .map(|p| p.msg.clone())
        .unwrap_or_else(|| "restarting daemon...".to_string());
    logger::log_to_terminal(cookbook, "info", "DAEMON", &msg);

    terminate_bar_daemon(cookbook)
        .await
        .context("Failed to terminate daemon for restart")?;
    sleep(Duration::from_millis(100)).await;
    spawn_bar_daemon(cookbook, debug)
        .await
        .context("Failed to spawn daemon for restart")?;
    Ok(())
}

pub fn spawn_debug_viewer(cookbook: &Arc<Cookbook>) -> Result<()> {
    let socket_path = get_socket_path();

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
        let msg = cookbook
            .dictionary
            .presets
            .get("sink_term_error")
            .map(|p| p.msg.clone())
            .unwrap_or_else(|| "no compatible terminal found for debug".to_string());
        logger::log_to_terminal(cookbook, "warn", "DAEMON", &msg);
    }

    Ok(())
}
