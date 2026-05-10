use flavor_compiler_core::{Diagnostic, SyntaxNode};

#[derive(Debug, Clone)]
pub struct SvelteMarkupAst {
    syntax: SyntaxNode,
    diagnostics: Vec<Diagnostic>,
}

impl SvelteMarkupAst {
    pub fn new(syntax: SyntaxNode, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            syntax,
            diagnostics,
        }
    }

    pub fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
