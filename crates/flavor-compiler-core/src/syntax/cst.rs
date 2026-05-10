use rowan::{Language, SyntaxKind};

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
