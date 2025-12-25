use std::fmt;
use tokio::sync::broadcast;

/// Message types that can be broadcasted on the event bus.
#[derive(Debug, Clone)]
pub enum BusMessage {
    /// A simple signal or trigger (e.g., "music_started", "reload_config")
    Signal(String),
    /// A data payload with a topic key (e.g., "spotify_track", "{...json...}")
    Data(String, String),
    /// Raw bytes payload
    Bytes(String, Vec<u8>),
}

impl fmt::Display for BusMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BusMessage::Signal(s) => write!(f, "Signal({})", s),
            BusMessage::Data(k, v) => write!(f, "Data({}, {} chars)", k, v.len()),
            BusMessage::Bytes(k, v) => write!(f, "Bytes({}, {} bytes)", k, v.len()),
        }
    }
}

/// The nervous system of KitchnSink.
/// Allows dishes to communicate with each other via a pub/sub mechanism.
#[derive(Debug, Clone)]
pub struct EventBus {
    tx: broadcast::Sender<BusMessage>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        // Capacity of 100 messages should be enough for a bar
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    /// broadcast a message to all subscribers.
    /// Returns the number of active subscribers.
    pub fn send(&self, msg: BusMessage) -> usize {
        // We ignore SendError because it only happens if there are no receivers, which is fine.
        self.tx.send(msg).unwrap_or(0)
    }

    /// Subscribe to the event bus.
    pub fn subscribe(&self) -> broadcast::Receiver<BusMessage> {
        self.tx.subscribe()
    }
}
