pub mod config;

pub use config::PluginCoreConfig;

#[derive(Debug, Clone)]
pub struct PluginCoreState {
    config: PluginCoreConfig,
}

impl PluginCoreState {
    pub fn new(config: PluginCoreConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &PluginCoreConfig {
        &self.config
    }
}
