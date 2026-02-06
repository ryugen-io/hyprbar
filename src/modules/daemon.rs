use crate::modules::config::{get_pid_file_path, get_socket_path};
use anyhow::{Context, Result};
use hyprink::config::Config;
use hyprlog;
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

pub async fn spawn_bar_daemon(config_ink: &Arc<Config>, debug: bool) -> Result<()> {
    let self_exe = env::current_exe().context("Failed to get current executable path")?;
    let pid_file_path = get_pid_file_path();

    // Check if daemon is already running
    if pid_file_path.exists()
        && let Ok(pid_str) = fs::read_to_string(&pid_file_path)
        && let Ok(pid) = pid_str.trim().parse::<u32>()
    {
        if let Ok(_proc) = Process::new(pid as i32) {
            // Use procfs here
            let msg = config_ink
                .layout
                .labels
                .get("bar_running")
                .map(|p| p.replace("{pid}", &pid.to_string()))
                .unwrap_or_else(|| format!("daemon running (pid: {})", pid));
            hyprlog::internal::info("DAEMON", &msg);
            return Ok(());
        } else {
            let msg = config_ink
                .layout
                .labels
                .get("bar_stale")
                .cloned()
                .unwrap_or_else(|| "stale pid file cleaned".to_string());
            hyprlog::internal::warn("DAEMON", &msg);
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

    let msg = config_ink
        .layout
        .labels
        .get("bar_start")
        .map(|p| p.replace("{pid}", &pid.to_string()))
        .unwrap_or_else(|| format!("daemon started (pid: {})", pid));
    hyprlog::internal::info("DAEMON", &msg);
    Ok(())
}

pub async fn terminate_bar_daemon(config_ink: &Arc<Config>) -> Result<()> {
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

        let msg = config_ink
            .layout
            .labels
            .get("bar_stop")
            .map(|p| p.replace("{pid}", &pid.to_string()))
            .unwrap_or_else(|| format!("daemon terminated (pid: {})", pid));
        hyprlog::internal::info("DAEMON", &msg);
    } else {
        let msg = config_ink
            .layout
            .labels
            .get("bar_not_found")
            .cloned()
            .unwrap_or_else(|| "no daemon found".to_string());
        hyprlog::internal::info("DAEMON", &msg);
    }
    Ok(())
}

pub async fn restart_bar_daemon(config_ink: &Arc<Config>, debug: bool) -> Result<()> {
    let msg = config_ink
        .layout
        .labels
        .get("bar_restart")
        .cloned()
        .unwrap_or_else(|| "restarting daemon...".to_string());
    hyprlog::internal::info("DAEMON", &msg);

    terminate_bar_daemon(config_ink)
        .await
        .context("Failed to terminate daemon for restart")?;
    sleep(Duration::from_millis(100)).await;
    spawn_bar_daemon(config_ink, debug)
        .await
        .context("Failed to spawn daemon for restart")?;
    Ok(())
}

pub fn spawn_debug_viewer(config_ink: &Arc<Config>) -> Result<()> {
    let socket_path = get_socket_path();

    // Check if a debug viewer is already running for this socket
    if is_debug_viewer_running(&socket_path) {
        hyprlog::internal::debug("DAEMON", "Debug viewer already running, skipping spawn");
        return Ok(());
    }

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

        hyprlog::internal::info("DAEMON", "Debug viewer spawned");
    } else {
        let msg = config_ink
            .layout
            .labels
            .get("bar_term_error")
            .cloned()
            .unwrap_or_else(|| "no compatible terminal found for debug".to_string());
        hyprlog::internal::warn("DAEMON", &msg);
    }

    Ok(())
}

fn is_debug_viewer_running(socket_path: &std::path::Path) -> bool {
    use std::fs;

    let socket_str = socket_path.to_string_lossy();

    // Check /proc for processes with internal-watch in cmdline
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(pid) = path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|s| s.parse::<u32>().ok())
            {
                let cmdline_path = path.join("cmdline");
                if let Ok(cmdline) = fs::read_to_string(&cmdline_path)
                    && cmdline.contains("internal-watch")
                    && cmdline.contains(&*socket_str)
                {
                    hyprlog::internal::debug(
                        "DAEMON",
                        &format!("Found existing viewer PID {}", pid),
                    );
                    return true;
                }
            }
        }
    }

    false
}
