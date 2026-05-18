pub mod config;

pub use config::{TemplateConfig, VuePluginConfig};

#[derive(Debug, Clone)]
pub struct VuePluginState {
    config: VuePluginConfig,
}

impl VuePluginState {
    pub fn new(config: VuePluginConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &VuePluginConfig {
        &self.config
    }
}
