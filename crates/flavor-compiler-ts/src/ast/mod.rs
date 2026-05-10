use flavor_compiler_core::{SourceText, SyntaxNode, Token};

use crate::syntax_kind::TsSyntaxKind;

#[derive(Debug, Clone)]
pub struct TsSourceFile {
    source: SourceText,
    tokens: Vec<Token<TsSyntaxKind>>,
    syntax: SyntaxNode,
}

impl TsSourceFile {
    pub fn new(source: SourceText, tokens: Vec<Token<TsSyntaxKind>>, syntax: SyntaxNode) -> Self {
        Self {
            source,
            tokens,
            syntax,
        }
    }

    pub fn source(&self) -> &SourceText {
        &self.source
    }

    pub fn tokens(&self) -> &[Token<TsSyntaxKind>] {
        &self.tokens
    }

    pub fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
