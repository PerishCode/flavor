use flavor_core::{Diagnostic, SourceText, Span, SyntaxNode};

#[derive(Debug, Clone)]
pub struct RustAnalysisOutput {
    pub source: SourceText,
    pub syntax: SyntaxNode,
    pub facts: RustFacts,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct RustFacts {
    pub names: Vec<RustNameFact>,
    pub match_arms: Vec<RustMatchArmFact>,
    pub test_attributes: Vec<RustTestAttributeFact>,
    pub repeated_token_patterns: Vec<RustRepeatedTokenPatternFact>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RustNameKind {
    Function,
    Method,
    Binding,
    Parameter,
}

impl RustNameKind {
    pub fn label(self) -> &'static str {
        match self {
            RustNameKind::Function => "function",
            RustNameKind::Method => "method",
            RustNameKind::Binding => "binding",
            RustNameKind::Parameter => "parameter",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RustNameFact {
    pub kind: RustNameKind,
    pub name: String,
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RustMatchArmFact {
    pub span: Span,
    pub line: usize,
    pub lines: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RustTestAttributeFact {
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RustRepeatedTokenPatternFact {
    pub span: Span,
    pub line: usize,
    pub occurrences: usize,
    pub total_lines: usize,
    pub token_count: usize,
    pub node_kind: u16,
    pub depth: usize,
}
