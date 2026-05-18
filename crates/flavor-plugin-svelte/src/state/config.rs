#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SveltePluginConfig {
    pub descriptor: bool,
    pub markup: bool,
    pub expressions: bool,
}

impl Default for SveltePluginConfig {
    fn default() -> Self {
        Self {
            descriptor: true,
            markup: true,
            expressions: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SveltePluginState {
    config: SveltePluginConfig,
}

impl SveltePluginState {
    pub fn new(config: SveltePluginConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &SveltePluginConfig {
        &self.config
    }
}
