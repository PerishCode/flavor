use flavor_core::{SourceText, SyntaxNode, Token};

use crate::internal::grammar::Kind;

#[derive(Debug, Clone)]
pub struct TsSourceFile {
    source: SourceText,
    tokens: Vec<Token<Kind>>,
    syntax: SyntaxNode,
}

impl TsSourceFile {
    pub fn new(source: SourceText, tokens: Vec<Token<Kind>>, syntax: SyntaxNode) -> Self {
        Self {
            source,
            tokens,
            syntax,
        }
    }

    pub fn source(&self) -> &SourceText {
        &self.source
    }

    pub fn tokens(&self) -> &[Token<Kind>] {
        &self.tokens
    }

    pub fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }

    pub fn into_parts(self) -> (SourceText, Vec<Token<Kind>>, SyntaxNode) {
        (self.source, self.tokens, self.syntax)
    }
}
