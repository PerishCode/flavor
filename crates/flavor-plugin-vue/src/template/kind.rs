use flavor_plugin_core::RawSyntaxKind;

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum VueTemplateKind {
    Root = 1000,
    Text = 1001,
    TagName = 1002,
    LessThan = 1003,
    GreaterThan = 1004,
    Slash = 1005,
    InterpolationOpen = 1006,
    InterpolationClose = 1007,
    ExpressionText = 1008,
    StartTag = 1009,
    EndTag = 1010,
    Attribute = 1011,
    Directive = 1012,
    AttributeName = 1013,
    DirectiveName = 1014,
    Equals = 1015,
    AttributeValue = 1016,
    Whitespace = 1017,
    Element = 1018,
    Interpolation = 1019,
    Comment = 1020,
    Error = 1021,
    DirectiveExpression = 1022,
    DirectiveBase = 1023,
    DirectiveArgument = 1024,
    DirectiveModifier = 1025,
}

impl From<VueTemplateKind> for RawSyntaxKind {
    fn from(kind: VueTemplateKind) -> Self {
        RawSyntaxKind(kind as u16)
    }
}
