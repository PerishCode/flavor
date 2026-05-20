use flavor_core::{RawSyntaxKind, SourceText, SyntaxBuilder, SyntaxNode, Token};

use crate::{GrammarKindName, RawAstSchema};

#[derive(Debug)]
pub struct RawAstBuilder<'schema> {
    inner: SyntaxBuilder,
    schema: &'schema RawAstSchema,
}

impl<'schema> RawAstBuilder<'schema> {
    pub fn new(schema: &'schema RawAstSchema) -> Self {
        Self {
            inner: SyntaxBuilder::new(),
            schema,
        }
    }

    pub fn start_node(&mut self, kind: impl GrammarKindName) {
        self.inner.start_node(self.schema.node_kind(kind));
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn token(&mut self, kind: impl GrammarKindName, text: &str) {
        self.inner.token(self.schema.token_kind(kind), text);
    }

    pub fn raw_token(&mut self, kind: impl Into<RawSyntaxKind>, text: &str) {
        self.inner.token(kind, text);
    }

    pub fn source_token<K>(&mut self, source: &SourceText, token: &Token<K>)
    where
        K: Copy + GrammarKindName,
    {
        for trivia in &token.leading {
            self.raw_token(trivia.kind, source.slice(trivia.span));
        }
        self.token(token.kind, source.slice(token.span));
    }

    pub fn finish(self) -> SyntaxNode {
        self.inner.finish()
    }
}
