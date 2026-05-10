mod parser;

use flavor_compiler_core::SourceText;

pub use parser::{parse_sfc, VueSfcBlock, VueSfcDescriptor, VueSfcError};

use crate::state::VueCompilerConfig;

pub fn parse(source: &SourceText, _config: &VueCompilerConfig) -> VueSfcDescriptor {
    parse_sfc(source.as_str())
}
