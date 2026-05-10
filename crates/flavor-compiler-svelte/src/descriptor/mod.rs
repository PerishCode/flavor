mod parser;

use flavor_compiler_core::SourceText;

pub use parser::{
    parse_descriptor, SvelteBlock, SvelteDescriptor, SvelteDescriptorError, SvelteMarkup,
};

use crate::state::SvelteCompilerConfig;

pub fn parse(source: &SourceText, config: &SvelteCompilerConfig) -> SvelteDescriptor {
    if config.descriptor {
        parse_descriptor(source.as_str())
    } else {
        SvelteDescriptor::default()
    }
}
