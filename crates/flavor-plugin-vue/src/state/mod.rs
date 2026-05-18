pub mod config;

use flavor_shared::PluginState;

pub use config::{TemplateConfig, VuePluginConfig};

pub type VuePluginState = PluginState<VuePluginConfig>;
