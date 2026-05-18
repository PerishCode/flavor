mod parser;

use flavor_core::SourceText;

pub use parser::{parse_sfc, VueSfcBlock, VueSfcDescriptor, VueSfcError};

use crate::state::VuePluginConfig;

pub fn parse(source: &SourceText, _config: &VuePluginConfig) -> VueSfcDescriptor {
    parse_sfc(source.as_str())
}
