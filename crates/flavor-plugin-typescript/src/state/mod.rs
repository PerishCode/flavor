pub mod config;

use flavor_shared::PluginState;

pub use config::{DecoratorsConfig, JsxConfig, SourceMode, TsPluginConfig};

pub type TsPluginState = PluginState<TsPluginConfig>;
