use flavor_core::{Diagnostic, SourceText};

pub use crate::{
    facts::{
        VueFacts, VueTemplateDirectiveClass, VueTemplateDirectiveFact, VueTemplateElementFact,
        VueTemplateExpressionFact,
    },
    sfc::{VueSfcBlock, VueSfcDescriptor, VueSfcError},
    template::TemplateAst,
};

#[derive(Debug, Clone)]
pub struct VueAnalysisOutput {
    pub source: SourceText,
    pub descriptor: VueSfcDescriptor,
    pub template: Option<TemplateAst>,
    pub facts: VueFacts,
    pub diagnostics: Vec<Diagnostic>,
}
