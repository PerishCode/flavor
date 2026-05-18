use flavor_plugin_core::RawSyntaxKind;

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SvelteMarkupKind {
    Root = 2000,
    Text = 2001,
    Comment = 2002,
    Element = 2003,
    Component = 2004,
    StartTag = 2005,
    EndTag = 2006,
    TagName = 2007,
    LessThan = 2008,
    GreaterThan = 2009,
    Slash = 2010,
    Whitespace = 2011,
    Attribute = 2012,
    AttributeName = 2013,
    AttributeValue = 2014,
    Equals = 2015,
    Directive = 2016,
    DirectiveName = 2017,
    DirectiveBase = 2018,
    DirectiveArgument = 2019,
    DirectiveModifier = 2020,
    DirectiveExpression = 2021,
    Mustache = 2022,
    MustacheOpen = 2023,
    MustacheClose = 2024,
    ExpressionText = 2025,
    Block = 2026,
    BlockOpen = 2027,
    BlockBranch = 2028,
    BlockClose = 2029,
    BlockKeyword = 2030,
    RenderTag = 2031,
    SpreadAttribute = 2032,
    ShorthandAttribute = 2033,
    SpecialTag = 2034,
    Error = 2035,
}

impl From<SvelteMarkupKind> for RawSyntaxKind {
    fn from(kind: SvelteMarkupKind) -> Self {
        RawSyntaxKind(kind as u16)
    }
}

impl SvelteMarkupKind {
    pub fn from_raw(kind: RawSyntaxKind) -> Self {
        match kind.0 {
            2000 => Self::Root,
            2001 => Self::Text,
            2002 => Self::Comment,
            2003 => Self::Element,
            2004 => Self::Component,
            2005 => Self::StartTag,
            2006 => Self::EndTag,
            2007 => Self::TagName,
            2008 => Self::LessThan,
            2009 => Self::GreaterThan,
            2010 => Self::Slash,
            2011 => Self::Whitespace,
            2012 => Self::Attribute,
            2013 => Self::AttributeName,
            2014 => Self::AttributeValue,
            2015 => Self::Equals,
            2016 => Self::Directive,
            2017 => Self::DirectiveName,
            2018 => Self::DirectiveBase,
            2019 => Self::DirectiveArgument,
            2020 => Self::DirectiveModifier,
            2021 => Self::DirectiveExpression,
            2022 => Self::Mustache,
            2023 => Self::MustacheOpen,
            2024 => Self::MustacheClose,
            2025 => Self::ExpressionText,
            2026 => Self::Block,
            2027 => Self::BlockOpen,
            2028 => Self::BlockBranch,
            2029 => Self::BlockClose,
            2030 => Self::BlockKeyword,
            2031 => Self::RenderTag,
            2032 => Self::SpreadAttribute,
            2033 => Self::ShorthandAttribute,
            2034 => Self::SpecialTag,
            _ => Self::Error,
        }
    }
}
