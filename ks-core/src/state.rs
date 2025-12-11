use crate::config::SinkConfig;
use k_lib::config::Cookbook;

use std::sync::Arc;

#[derive(Debug)]
pub struct BarState {
    pub cpu: f32,
    pub mem: f32,
    pub time: String,
    pub cookbook: Arc<Cookbook>,
    pub config: SinkConfig,
}

impl BarState {
    pub fn new(cookbook: Arc<Cookbook>, config: SinkConfig) -> Self {
        Self {
            cpu: 0.0,
            mem: 0.0,
            time: String::new(),
            cookbook,
            config,
        }
    }
}
