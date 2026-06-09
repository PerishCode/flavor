use flavor_core::SourceText;
use flavor_grammar::parse_vue_sfc;

pub use flavor_grammar::{VueSfcBlock, VueSfcDescriptor, VueSfcError};

use crate::state::VuePluginConfig;

pub fn parse(source: &SourceText, _config: &VuePluginConfig) -> VueSfcDescriptor {
    parse_vue_sfc(source.as_str())
}
