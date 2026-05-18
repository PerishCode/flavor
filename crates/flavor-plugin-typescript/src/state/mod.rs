pub mod config;

pub use config::{DecoratorsConfig, JsxConfig, SourceMode, TsPluginConfig};

#[derive(Debug, Clone)]
pub struct TsPluginState {
    config: TsPluginConfig,
}

impl TsPluginState {
    pub fn new(config: TsPluginConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &TsPluginConfig {
        &self.config
    }
}
