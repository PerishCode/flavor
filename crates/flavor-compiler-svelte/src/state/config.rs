#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SvelteCompilerConfig {
    pub descriptor: bool,
    pub markup: bool,
    pub expressions: bool,
}

impl Default for SvelteCompilerConfig {
    fn default() -> Self {
        Self {
            descriptor: true,
            markup: true,
            expressions: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SvelteCompilerState {
    config: SvelteCompilerConfig,
}

impl SvelteCompilerState {
    pub fn new(config: SvelteCompilerConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &SvelteCompilerConfig {
        &self.config
    }
}
