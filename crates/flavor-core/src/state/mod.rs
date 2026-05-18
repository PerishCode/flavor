pub mod config;

pub use config::FlavorCoreConfig;

#[derive(Debug, Clone)]
pub struct FlavorCoreState {
    config: FlavorCoreConfig,
}

impl FlavorCoreState {
    pub fn new(config: FlavorCoreConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &FlavorCoreConfig {
        &self.config
    }
}
