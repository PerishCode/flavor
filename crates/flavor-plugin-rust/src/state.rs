#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct RustPluginConfig {}

#[derive(Debug, Clone)]
pub struct RustPluginState {
    config: RustPluginConfig,
}

impl RustPluginState {
    pub fn new(config: RustPluginConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &RustPluginConfig {
        &self.config
    }
}
