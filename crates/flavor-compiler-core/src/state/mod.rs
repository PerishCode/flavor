pub mod config;

pub use config::CompilerCoreConfig;

#[derive(Debug, Clone)]
pub struct CompilerCoreState {
    config: CompilerCoreConfig,
}

impl CompilerCoreState {
    pub fn new(config: CompilerCoreConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &CompilerCoreConfig {
        &self.config
    }
}
