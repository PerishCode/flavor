pub mod config;

pub use config::{TemplateConfig, VueCompilerConfig};

#[derive(Debug, Clone)]
pub struct VueCompilerState {
    config: VueCompilerConfig,
}

impl VueCompilerState {
    pub fn new(config: VueCompilerConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &VueCompilerConfig {
        &self.config
    }
}
