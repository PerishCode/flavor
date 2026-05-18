use rowan::{Language, SyntaxKind};

use crate::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RawSyntaxKind(pub u16);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FlavorLanguage {}

impl Language for FlavorLanguage {
    type Kind = RawSyntaxKind;

    fn kind_from_raw(raw: SyntaxKind) -> Self::Kind {
        RawSyntaxKind(raw.0)
    }

    fn kind_to_raw(kind: Self::Kind) -> SyntaxKind {
        SyntaxKind(kind.0)
    }
}

pub type SyntaxNode = rowan::SyntaxNode<FlavorLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<FlavorLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<FlavorLanguage>;

pub trait SyntaxSpanExt {
    fn source_span(&self) -> Span;
}

impl SyntaxSpanExt for SyntaxNode {
    fn source_span(&self) -> Span {
        let range = self.text_range();
        Span::new(range.start().into(), range.end().into())
    }
}

impl SyntaxSpanExt for SyntaxToken {
    fn source_span(&self) -> Span {
        let range = self.text_range();
        Span::new(range.start().into(), range.end().into())
    }
}
