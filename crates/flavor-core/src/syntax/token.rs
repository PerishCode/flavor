use crate::{Span, Trivia};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token<K> {
    pub kind: K,
    pub span: Span,
    pub leading: Vec<Trivia>,
    pub trailing: Vec<Trivia>,
}

impl<K> Token<K> {
    pub fn new(kind: K, span: Span) -> Self {
        Self {
            kind,
            span,
            leading: Vec::new(),
            trailing: Vec::new(),
        }
    }
}
