use crate::config::SinkConfig;
use k_lib::config::Cookbook;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct BarState {
    pub cookbook: Arc<Cookbook>,
    pub config: SinkConfig,
    pub bus: crate::bus::EventBus,
}

impl BarState {
    pub fn new(cookbook: Arc<Cookbook>, config: SinkConfig) -> Self {
        let bus = crate::bus::EventBus::new();
        Self {
            cookbook,
            config,
            bus,
        }
    }
}
