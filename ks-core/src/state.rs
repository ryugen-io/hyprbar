use crate::config::SinkConfig;
use kitchn_lib::config::Cookbook;

#[derive(Debug)]
pub struct BarState {
    pub cpu: f32,
    pub mem: f32,
    pub time: String,
    pub cookbook: Cookbook,
    pub config: SinkConfig,
}

impl BarState {
    pub fn new(cookbook: Cookbook, config: SinkConfig) -> Self {
        Self {
            cpu: 0.0,
            mem: 0.0,
            time: String::new(),
            cookbook,
            config,
        }
    }
}
