use flavor_core::{Diagnostic, SourceText, SyntaxNode};

#[derive(Debug, Clone)]
pub struct GrammarParseOutput {
    pub source: SourceText,
    pub syntax: SyntaxNode,
    pub diagnostics: Vec<Diagnostic>,
}

impl GrammarParseOutput {
    pub fn new(source: SourceText, syntax: SyntaxNode, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            source,
            syntax,
            diagnostics,
        }
    }
}
