use crate::{RawSyntaxKind, Span};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TriviaKind {
    Whitespace,
    LineComment,
    BlockComment,
    Shebang,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub span: Span,
}

impl Trivia {
    pub fn new(kind: TriviaKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl From<TriviaKind> for RawSyntaxKind {
    fn from(kind: TriviaKind) -> Self {
        match kind {
            TriviaKind::Whitespace => RawSyntaxKind(1),
            TriviaKind::LineComment => RawSyntaxKind(2),
            TriviaKind::BlockComment => RawSyntaxKind(3),
            TriviaKind::Shebang => RawSyntaxKind(4),
        }
    }
}
