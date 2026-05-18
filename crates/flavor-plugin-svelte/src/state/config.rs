use flavor_shared::PluginState;

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

pub type SveltePluginState = PluginState<SveltePluginConfig>;
