use crate::config::SinkConfig;
use k_lib::config::Cookbook;

use std::sync::Arc;

#[derive(Debug)]
pub struct BarState {
    pub cookbook: Arc<Cookbook>,
    pub config: SinkConfig,
}

impl BarState {
    pub fn new(cookbook: Arc<Cookbook>, config: SinkConfig) -> Self {
        Self { cookbook, config }
    }
}
