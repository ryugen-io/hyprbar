use anyhow::Context;
use colored::Colorize;
use std::collections::VecDeque;
use std::fs;
use std::sync::{Mutex, OnceLock};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;
use tokio::sync::broadcast;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::modules::config::get_socket_path;

/// Global channel for broadcasting logs to connected debug terminals
pub static LOG_CHANNEL: OnceLock<broadcast::Sender<String>> = OnceLock::new();
/// Global buffer for capturing startup logs to send to new clients
pub static STARTUP_BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

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
        // Format similarly to previous logger
        let metadata = event.metadata();

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
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    // Tracing messages use fmt::Arguments which print correctly via Debug
                    self.0.push_str(&format!("{:?}", value));
                }
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

        // Save to Startup Buffer
        #[allow(clippy::collapsible_if)]
        if let Some(buffer) = STARTUP_BUFFER.get() {
            if let Ok(mut lock) = buffer.lock() {
                if lock.len() >= 50 {
                    lock.pop_front();
                }
                lock.push_back(msg.clone());
            }
        }

        // Broadcast
        if let Some(sender) = LOG_CHANNEL.get() {
            let _ = sender.send(msg);
        }
    }
}

use k_lib::config::Cookbook;
use k_lib::logger;
use std::sync::Arc;

/// Layer that forwards logs to kitchn_lib's file logger
struct KitchnFileLayer {
    cookbook: Arc<Cookbook>,
}

impl<S> Layer<S> for KitchnFileLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Skip if not writing by default, unless overriden?
        // For now, we respect the global config directly in log_to_file call or here.
        // k_lib::logger::log_to_file doesn't check 'write_by_default', it just writes.
        // So we should check it here.
        if !self.cookbook.layout.logging.write_by_default {
            return;
        }

        let metadata = event.metadata();
        let level_str = metadata.level().as_str().to_lowercase();

        // Visitor to extract message field
        struct MessageVisitor(String);
        impl tracing::field::Visit for MessageVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0.push_str(&format!("{:?}", value));
                }
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

        // Determine scope - currently hardcoded or extracted?
        // We can use target as scope, or a fixed "SINK" scope.
        // Let's use target for now, but clean it up maybe?
        let target = metadata.target();
        // If target is module path, it might be too long.
        // For now, let's stick to "SINK" for consistency with main.rs,
        // OR use "SINK" if target starts with "ks_", otherwise use target.
        let scope = if target.starts_with("ks_") || target == "ks_bin" {
            "SINK"
        } else {
            target
        };

        // We use the synchronous log_to_file.
        // NOTE: This might block the async runtime if disk is slow.
        // Ideally this should be done in a separate blocking task or thread,
        // but for now strictly following the request to use k_lib directly.
        let _ = logger::log_to_file(
            &self.cookbook,
            &level_str,
            scope,
            &message,
            Some("kitchnsink"), // Enforce app name
        );
    }
}

pub fn init_logging(
    cookbook: Arc<Cookbook>,
    enable_debug: bool,
    config_level: &str,
    config_filter: &str,
) -> anyhow::Result<()> {
    // 0. Setup LogTracer (Handled automatically by registry().init() if tracing-log feature is enabled)
    // tracing_log::LogTracer::init().map_err(|_| anyhow::anyhow!("Failed to init LogTracer"))?;

    // Force colored output even if no TTY (daemon mode)
    colored::control::set_override(true);

    // 1. Setup Broadcast Channel
    let (tx, _) = broadcast::channel(100);
    LOG_CHANNEL
        .set(tx.clone())
        .map_err(|_| anyhow::anyhow!("Failed to set global log channel"))?;

    // 2. Setup Startup Buffer
    STARTUP_BUFFER
        .set(Mutex::new(VecDeque::with_capacity(50)))
        .map_err(|_| anyhow::anyhow!("Failed to set global startup buffer"))?;

    // 3. Setup Tracing Subscriber
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .parse_lossy(if enable_debug {
            config_filter
        } else {
            config_level
        });

    let socket_layer = SocketSubscriberLayer;

    // We can also add a stdout layer for local debugging if needed,
    // but kitchn philosophy relies on the socket or k_lib logger.
    // However, k_lib logger uses 'log' crate.
    // Tracing captures 'log' events via 'tracing-log' (usually enabled by default or feature).
    // Let's add a basic Fmt layer for stderr as fallback/standard behavior.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .without_time(); // Time is handled by our socket layer or rely on systemd

    let file_layer = KitchnFileLayer {
        cookbook, // passed as Arc already
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(socket_layer)
        .with(file_layer)
        .init();

    // 4. Start Socket Server if debug is enabled
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
                        // Send startup logs first (Snapshot to avoid holding lock across await)
                        // This handles the race condition where logs are emitted before the client connects
                        let startup_logs: Vec<String> = if let Some(buffer) = STARTUP_BUFFER.get() {
                            if let Ok(lock) = buffer.lock() {
                                lock.iter().cloned().collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        };

                        for line in startup_logs {
                            if stream.write_all(line.as_bytes()).await.is_err() {
                                break;
                            }
                        }

                        // Subscribe to live events
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
