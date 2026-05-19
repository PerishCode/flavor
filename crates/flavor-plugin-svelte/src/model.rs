use flavor_core::{Diagnostic, SourceText};

pub use crate::{
    descriptor::{SvelteBlock, SvelteDescriptor, SvelteDescriptorError, SvelteMarkup},
    facts::{SvelteFacts, SvelteMarkupNameFact},
    markup::SvelteMarkupAst,
};

#[derive(Debug, Clone)]
pub struct SvelteAnalysisOutput {
    pub source: SourceText,
    pub descriptor: SvelteDescriptor,
    pub markup: Option<SvelteMarkupAst>,
    pub facts: SvelteFacts,
    pub diagnostics: Vec<Diagnostic>,
}
