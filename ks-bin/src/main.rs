use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use kitchn_lib::config::Cookbook;
use kitchn_lib::logger;
use ks_core::config::SinkConfig;
use ks_core::renderer::BarRenderer;
use ks_core::state::BarState;
use ks_wayland::init as init_wayland;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;
use tracing::{debug, info};
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

mod tui;

/// Global channel for broadcasting logs to connected debug terminals
static LOG_CHANNEL: OnceLock<broadcast::Sender<String>> = OnceLock::new();

struct SocketSubscriberLayer;

impl<S> Layer<S> for SocketSubscriberLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if let Some(sender) = LOG_CHANNEL.get() {
            // Check if level is <= DEBUG (rough filter, though EnvFilter handles main filtering)
            let metadata = event.metadata();

            // Format similarly to previous logger
            let level_color = match *metadata.level() {
                tracing::Level::ERROR => "ERROR".red(),
                tracing::Level::WARN => "WARN".yellow(),
                tracing::Level::INFO => "INFO".green(),
                tracing::Level::DEBUG => "DEBUG".blue(),
                tracing::Level::TRACE => "TRACE".magenta(),
            };

            let timestamp = chrono::Local::now().format("%H:%M:%S").to_string().dimmed();

            // Visitor to extract message field
            struct MessageVisitor(String);
            impl tracing::field::Visit for MessageVisitor {
                fn record_debug(
                    &mut self,
                    _field: &tracing::field::Field,
                    _value: &dyn std::fmt::Debug,
                ) {
                }
                fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                    if field.name() == "message" {
                        self.0.push_str(value);
                    }
                }
            }

            let mut visitor = MessageVisitor(String::new());
            event.record(&mut visitor);
            let message = visitor.0;

            let msg = format!("{} [{}] {}\n", timestamp, level_color, message);
            let _ = sender.send(msg);
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "kitchnsink", version, about = "Kitchn Sink Wayland Bar")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable debug mode with verbose logging in a separate terminal
    #[arg(long, global = true)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Internal command to watch logs via socket (Hidden)
    #[command(hide = true)]
    InternalWatch { socket_path: PathBuf },

    /// Manage the bar (TUI)
    #[command(alias = "m")]
    Manage,
}

fn load_sink_config() -> SinkConfig {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_path = PathBuf::from(home)
        .join(".config")
        .join("kitchnsink")
        .join("sink.toml"); // User requested "sink.toml"

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(cfg) => return cfg,
                Err(e) => eprintln!("Failed to parse config: {}", e),
            },
            Err(e) => eprintln!("Failed to read config: {}", e),
        }
    }

    // Fallback
    SinkConfig::default()
}

fn get_socket_path() -> PathBuf {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir());
    runtime_dir.join("kitchnsink-debug.sock")
}

fn init_logging(enable_debug: bool) -> Result<()> {
    // 1. Setup Broadcast Channel
    let (tx, _) = broadcast::channel(100);
    LOG_CHANNEL
        .set(tx.clone())
        .map_err(|_| anyhow::anyhow!("Failed to set global log channel"))?;

    // 2. Setup Tracing Subscriber
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(if enable_debug {
            tracing::Level::DEBUG.into()
        } else {
            tracing::Level::INFO.into()
        })
        .parse_lossy(""); // Parse empty string to use default directive

    let socket_layer = SocketSubscriberLayer;

    // We can also add a stdout layer for local debugging if needed,
    // but kitchn philosophy relies on the socket or kitchn_lib logger.
    // However, kitchn_lib logger uses 'log' crate.
    // Tracing captures 'log' events via 'tracing-log' (usually enabled by default or feature).
    // Let's add a basic Fmt layer for stderr as fallback/standard behavior.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .without_time(); // Time is handled by our socket layer or rely on systemd

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(socket_layer)
        .init();

    // 3. Start Socket Server if debug is enabled
    if enable_debug {
        let socket_path = get_socket_path();
        // Remove existing socket
        if socket_path.exists() {
            let _ = fs::remove_file(&socket_path);
        }

        let listener = UnixListener::bind(&socket_path).context("Failed to bind debug socket")?;

        // Spawn server task
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((mut stream, _addr)) => {
                        let mut rx = tx.subscribe();
                        tokio::spawn(async move {
                            while let Ok(msg) = rx.recv().await {
                                if stream.write_all(msg.as_bytes()).await.is_err() {
                                    break;
                                }
                            }
                        });
                    }
                    Err(e) => eprintln!("Accept failed: {}", e),
                }
            }
        });
    }

    Ok(())
}

fn spawn_debug_viewer() -> Result<()> {
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
        let _ = Command::new(&term)
            .arg("-e")
            .arg(&self_exe)
            .arg("internal-watch")
            .arg(&socket_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn debug terminal")?;

        println!(
            "Debug Mode Started. Tailing logs via socket: {:?}",
            socket_path
        );
    } else {
        println!("No compatible terminal found for debug mode.");
    }

    Ok(())
}

async fn run_watcher(socket_path: &Path) -> Result<()> {
    println!("{}", "KitchnSink Debug Watcher".bold().underline());
    println!("Connecting to: {:?}\n", socket_path);

    // Retry connection loop
    let stream = loop {
        match UnixStream::connect(socket_path).await {
            Ok(s) => break s,
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }
        }
    };

    let mut buf = [0u8; 1024];
    loop {
        // Simple read loop - output is line based
        match stream.try_read(&mut buf) {
            Ok(0) => break, // EOF
            Ok(n) => {
                let s = String::from_utf8_lossy(&buf[..n]);
                print!("{}", s);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(_) => break,
        }
    }

    println!("{}", "\nConnection closed.".yellow());
    // Keep window open for a bit
    tokio::time::sleep(Duration::from_secs(5)).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 0. Handle Internal Watcher (Client Mode)
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::InternalWatch { socket_path } => return run_watcher(socket_path).await,
            Commands::Manage => {
                let cookbook =
                    Cookbook::load().context("Failed to load kitchn cookbook for TUI")?;
                return tui::run_tui(cookbook).map_err(|e| anyhow::anyhow!("TUI error: {}", e));
            }
        }
    }

    // 1. Load Kitchn Config (Global Styles)
    let cookbook = Cookbook::load().context("Failed to load kitchn cookbook")?;

    // 2. Load Sink Config (App Layout)
    let config = load_sink_config();

    // 3. Initialize Logging
    init_logging(cli.debug)?;

    if cli.debug {
        spawn_debug_viewer()?;
        // Short pause to let viewer connect
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    logger::log_to_terminal(&cookbook, "info", "SINK", "kitchnsink starting...");

    // 4. Initialize Bar State
    let mut bar_state = BarState::new(cookbook, config.clone());

    // 5. Initialize Renderer (Offscreen)
    let mut renderer = BarRenderer::new(100, config.window.height as u16);

    // 6. Initialize Wayland
    let (mut wayland_state, mut event_queue, _layer_surface) =
        init_wayland(config.window.height, config.window.anchor == "bottom")
            .context("Failed to initialize Wayland")?;
    let qh = event_queue.handle();

    // 7. Event Loop
    // Use standard tracing::info! instead of logger::log_to_terminal to verify our stream
    info!("Starting Wayland event loop");

    loop {
        if wayland_state.exit {
            info!("Exiting...");
            break;
        }

        if wayland_state.configured {
            let width = u16::try_from(wayland_state.width).unwrap_or(u16::MAX);
            let height = u16::try_from(wayland_state.height).unwrap_or(u16::MAX);

            if width > 0 && height > 0 && (renderer.width != width || renderer.height != height) {
                renderer.resize(width, height);
                debug!("Resized to {}x{}", width, height);
            }

            if wayland_state.redraw_requested {
                bar_state.cpu = 12.5;
                bar_state.mem = 45.2;

                renderer.render_frame(&bar_state, Duration::from_millis(16))?;
                wayland_state.draw(&qh, renderer.buffer(), &bar_state.cookbook)?;
                // trace!("Frame rendered"); // Too noisy
            }
        }

        event_queue
            .blocking_dispatch(&mut wayland_state)
            .context("Wayland dispatch failed")?;
    }

    Ok(())
}
