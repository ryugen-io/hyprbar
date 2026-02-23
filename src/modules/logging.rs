use anyhow::Context;
use colored::Colorize;
use std::collections::VecDeque;
use std::fs;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;
use tokio::sync::broadcast;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

use hyprink::config::Config;
use hyprlog::Logger;

use crate::modules::config::get_socket_path;

/// Debug viewers connect over Unix sockets and need a shared channel to receive live logs.
pub static LOG_CHANNEL: OnceLock<broadcast::Sender<String>> = OnceLock::new();
/// Viewers that connect after boot would miss initialization messages without a replay buffer.
pub static STARTUP_BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
/// Tracing layers are stateless callbacks — they need a shared logger to persist output to disk.
pub static HYPRLOG: OnceLock<Logger> = OnceLock::new();

/// Dual output ensures log messages reach both persistent storage and live debug viewers.
pub fn log_info(scope: &str, msg: &str) {
    log_with_hyprlog(Level::Info, scope, msg);
    broadcast_log("INFO", scope, msg);
}

pub fn log_debug(scope: &str, msg: &str) {
    log_with_hyprlog(Level::Debug, scope, msg);
    broadcast_log("DEBUG", scope, msg);
}

pub fn log_warn(scope: &str, msg: &str) {
    log_with_hyprlog(Level::Warn, scope, msg);
    broadcast_log("WARN", scope, msg);
}

pub fn log_error(scope: &str, msg: &str) {
    log_with_hyprlog(Level::Error, scope, msg);
    broadcast_log("ERROR", scope, msg);
}

#[derive(Clone, Copy)]
enum Level {
    Info,
    Debug,
    Warn,
    Error,
}

// Use the configured hyprlog logger as the primary sink.
// Fallback keeps early-start logs visible before init_logging() runs.
fn log_with_hyprlog(level: Level, scope: &str, msg: &str) {
    if let Some(logger) = HYPRLOG.get() {
        match level {
            Level::Info => logger.info(scope, msg),
            Level::Debug => logger.debug(scope, msg),
            Level::Warn => logger.warn(scope, msg),
            Level::Error => logger.error(scope, msg),
        }
    } else {
        match level {
            Level::Info => hyprlog::internal::info(scope, msg),
            Level::Debug => hyprlog::internal::debug(scope, msg),
            Level::Warn => hyprlog::internal::warn(scope, msg),
            Level::Error => hyprlog::internal::error(scope, msg),
        }
    }
}

fn broadcast_log(level: &str, scope: &str, msg: &str) {
    let level_color = match level {
        "ERROR" => level.red(),
        "WARN" => level.yellow(),
        "INFO" => level.green(),
        "DEBUG" => level.blue(),
        _ => level.normal(),
    };

    let timestamp = chrono::Local::now().format("%H:%M:%S").to_string().dimmed();
    let formatted = format!(
        "{} [{}] [{}] {}\n",
        timestamp,
        level_color,
        scope.cyan(),
        msg
    );

    // Late-connecting viewers need to see logs from before they attached.
    if let Some(buffer) = STARTUP_BUFFER.get()
        && let Ok(mut lock) = buffer.lock()
    {
        if lock.len() >= 50 {
            lock.pop_front();
        }
        lock.push_back(formatted.clone());
    }

    if let Some(sender) = LOG_CHANNEL.get() {
        let _ = sender.send(formatted);
    }
}

struct SocketSubscriberLayer;

/// Tracing's subscriber model requires a Layer to bridge events into hyprlog's file-based API.
struct HyprlogLayer;

impl<S> Layer<S> for HyprlogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let Some(logger) = HYPRLOG.get() else {
            return;
        };

        let metadata = event.metadata();

        // tracing doesn't give direct access to the message — visitor pattern is required.
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

        let target = metadata.target();
        let scope = if target.starts_with("hyprbar") {
            "HYPRBAR"
        } else {
            target
        };

        match *metadata.level() {
            tracing::Level::ERROR => logger.error(scope, &message),
            tracing::Level::WARN => logger.warn(scope, &message),
            tracing::Level::INFO => logger.info(scope, &message),
            tracing::Level::DEBUG => logger.debug(scope, &message),
            tracing::Level::TRACE => logger.trace(scope, &message),
        }
    }
}

impl<S> Layer<S> for SocketSubscriberLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();

        let level_color = match *metadata.level() {
            tracing::Level::ERROR => "ERROR".red(),
            tracing::Level::WARN => "WARN".yellow(),
            tracing::Level::INFO => "INFO".green(),
            tracing::Level::DEBUG => "DEBUG".blue(),
            tracing::Level::TRACE => "TRACE".magenta(),
        };

        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string().dimmed();

        struct MessageVisitor(String);
        impl tracing::field::Visit for MessageVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    // fmt::Arguments implements Debug but not Display, so {:?} is correct here.
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

        // Late-connecting viewers need to see logs from before they attached.
        #[allow(clippy::collapsible_if)]
        if let Some(buffer) = STARTUP_BUFFER.get() {
            if let Ok(mut lock) = buffer.lock() {
                if lock.len() >= 50 {
                    lock.pop_front();
                }
                lock.push_back(msg.clone());
            }
        }

        if let Some(sender) = LOG_CHANNEL.get() {
            let _ = sender.send(msg);
        }
    }
}

pub fn init_logging(
    _config_ink: Arc<Config>,
    enable_debug: bool,
    config_level: &str,
    config_filter: &str,
    bind_socket: bool,
) -> anyhow::Result<()> {
    // Daemon runs without a TTY, but debug viewers expect ANSI colors.
    colored::control::set_override(true);

    let (tx, _) = broadcast::channel(100);
    LOG_CHANNEL
        .set(tx.clone())
        .map_err(|_| anyhow::anyhow!("Failed to set global log channel"))?;

    STARTUP_BUFFER
        .set(Mutex::new(VecDeque::with_capacity(50)))
        .map_err(|_| anyhow::anyhow!("Failed to set global startup buffer"))?;

    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .parse_lossy(if enable_debug {
            config_filter
        } else {
            config_level
        });

    let socket_layer = SocketSubscriberLayer;

    // stderr fallback ensures logs appear even without a debug viewer.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .without_time(); // timestamps come from the socket layer or systemd journal

    // CLI commands (non-daemon) forward their logs to the daemon's viewer socket.
    let publisher_layer = if !bind_socket {
        let socket_path = get_socket_path();
        if socket_path.exists() {
            if let Ok(stream) = std::os::unix::net::UnixStream::connect(socket_path) {
                // Read side unused — shutting it prevents the daemon's echo from filling the buffer.
                let _ = stream.shutdown(std::net::Shutdown::Read);
                Some(SocketPublisherLayer {
                    stream: Mutex::new(stream),
                })
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    HYPRLOG
        .set(Logger::from_config("hyprbar"))
        .map_err(|_| anyhow::anyhow!("Failed to set hyprlog logger"))?;

    let hyprlog_layer = HyprlogLayer;

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(socket_layer)
        .with(publisher_layer)
        .with(hyprlog_layer)
        .init();

    // Only the daemon owns the socket — CLI processes connect as clients above.
    if enable_debug && bind_socket {
        let socket_path = get_socket_path();
        if socket_path.exists() {
            let _ = fs::remove_file(&socket_path);
        }

        let listener = UnixListener::bind(&socket_path).context("Failed to bind debug socket")?;
        let tx = tx.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        let (reader, mut writer) = stream.into_split();
                        let tx_for_read = tx.clone();
                        let mut rx_for_write = tx.subscribe();

                        // CLI processes publish logs here — without ingesting them,
                        // only daemon-internal logs would reach debug viewers.
                        tokio::spawn(async move {
                            use tokio::io::{AsyncBufReadExt, BufReader};
                            let mut buf_reader = BufReader::new(reader);
                            let mut line = String::new();
                            while let Ok(n) = buf_reader.read_line(&mut line).await {
                                if n == 0 {
                                    break;
                                }
                                let _ = tx_for_read.send(line.clone());
                                line.clear();
                            }
                        });

                        // Viewers that connect after boot would miss initialization
                        // messages without a replay step.
                        tokio::spawn(async move {
                            let startup_logs: Vec<String> =
                                if let Some(buffer) = STARTUP_BUFFER.get() {
                                    if let Ok(lock) = buffer.lock() {
                                        lock.iter().cloned().collect()
                                    } else {
                                        Vec::new()
                                    }
                                } else {
                                    Vec::new()
                                };

                            for line in startup_logs {
                                if writer.write_all(line.as_bytes()).await.is_err() {
                                    return;
                                }
                            }

                            while let Ok(msg) = rx_for_write.recv().await {
                                if writer.write_all(msg.as_bytes()).await.is_err() {
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

/// CLI processes aren't the daemon — they forward logs over a socket so
/// the daemon's debug viewer can show them.
struct SocketPublisherLayer {
    stream: Mutex<std::os::unix::net::UnixStream>,
}

impl<S> Layer<S> for SocketPublisherLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let level_color = match *metadata.level() {
            tracing::Level::ERROR => "ERROR".red(),
            tracing::Level::WARN => "WARN".yellow(),
            tracing::Level::INFO => "INFO".green(),
            tracing::Level::DEBUG => "DEBUG".blue(),
            tracing::Level::TRACE => "TRACE".magenta(),
        };

        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string().dimmed();

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

        let msg = format!("{} [{}] {}\n", timestamp, level_color, message);

        if let Ok(mut stream) = self.stream.lock() {
            use std::io::Write;
            let _ = stream.write_all(msg.as_bytes());
        }
    }
}
