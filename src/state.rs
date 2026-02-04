use crate::config::BarConfig;
use hyprink::config::Config;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct BarState {
    pub config_ink: Arc<Config>,
    pub config: BarConfig,
    pub bus: crate::bus::EventBus,
}

impl BarState {
    pub fn new(config_ink: Arc<Config>, config: BarConfig) -> Self {
        let bus = crate::bus::EventBus::new();
        Self {
            config_ink,
            config,
            bus,
        }
    }
}
