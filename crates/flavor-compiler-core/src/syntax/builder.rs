use rowan::{GreenNodeBuilder, SyntaxKind};

use crate::{RawSyntaxKind, SyntaxNode};

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

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn token(&mut self, kind: impl Into<RawSyntaxKind>, text: &str) {
        self.inner.token(raw(kind), text);
    }

    pub fn finish(self) -> SyntaxNode {
        SyntaxNode::new_root(self.inner.finish())
    }
}

fn raw(kind: impl Into<RawSyntaxKind>) -> SyntaxKind {
    SyntaxKind(kind.into().0)
}
