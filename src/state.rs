use std::sync::Arc;

use crate::config::BarConfig;

#[derive(Debug, Clone)]
pub struct BarState {
    pub config: Arc<BarConfig>,
    pub bus: crate::bus::EventBus,
}

impl BarState {
    pub fn new(config: Arc<BarConfig>) -> Self {
        let bus = crate::bus::EventBus::new();
        Self { config, bus }
    }
}
