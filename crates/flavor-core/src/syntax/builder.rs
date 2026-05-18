use rowan::{GreenNodeBuilder, SyntaxKind};

use crate::{RawSyntaxKind, SyntaxNode};

pub trait SyntaxKindSchema: Copy + Into<RawSyntaxKind> {
    fn raw_is_node(kind: RawSyntaxKind) -> bool;
    fn raw_is_token(kind: RawSyntaxKind) -> bool;

    fn is_node(self) -> bool {
        Self::raw_is_node(self.into())
    }

    fn is_token(self) -> bool {
        Self::raw_is_token(self.into())
    }
}

#[derive(Debug, Default)]
pub struct SyntaxBuilder {
    inner: GreenNodeBuilder<'static>,
}

impl SyntaxBuilder {
    pub fn new() -> Self {
        Self {
            inner: GreenNodeBuilder::new(),
        }
    }

    pub fn start_node(&mut self, kind: impl Into<RawSyntaxKind>) {
        self.inner.start_node(raw(kind));
    }

    pub fn start_schema_node(&mut self, kind: impl SyntaxKindSchema) {
        assert!(kind.is_node(), "schema kind must be a node");
        self.start_node(kind);
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn token(&mut self, kind: impl Into<RawSyntaxKind>, text: &str) {
        self.inner.token(raw(kind), text);
    }

    pub fn schema_token(&mut self, kind: impl SyntaxKindSchema, text: &str) {
        assert!(kind.is_token(), "schema kind must be a token");
        self.token(kind, text);
    }

    pub fn finish(self) -> SyntaxNode {
        SyntaxNode::new_root(self.inner.finish())
    }
}

fn raw(kind: impl Into<RawSyntaxKind>) -> SyntaxKind {
    SyntaxKind(kind.into().0)
}
