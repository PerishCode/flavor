pub mod config;

pub use config::{DecoratorsConfig, JsxConfig, SourceMode, TsCompilerConfig};

#[derive(Debug, Clone)]
pub struct TsCompilerState {
    config: TsCompilerConfig,
}

impl TsCompilerState {
    pub fn new(config: TsCompilerConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &TsCompilerConfig {
        &self.config
    }
}
