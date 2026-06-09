mod sfc;
mod template;

pub use sfc::{parse_vue_sfc, VueSfcBlock, VueSfcDescriptor, VueSfcError};
pub use template::parse_vue_template;
