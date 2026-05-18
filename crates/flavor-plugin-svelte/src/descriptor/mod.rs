mod parser;

use flavor_plugin_core::SourceText;

pub use parser::{
    parse_descriptor, SvelteBlock, SvelteDescriptor, SvelteDescriptorError, SvelteMarkup,
};

use crate::state::SveltePluginConfig;

pub fn parse(source: &SourceText, config: &SveltePluginConfig) -> SvelteDescriptor {
    if config.descriptor {
        parse_descriptor(source.as_str())
    } else {
        SvelteDescriptor::default()
    }
}
