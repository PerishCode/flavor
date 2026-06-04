use flavor_core::{Diagnostic, SourceText, Span, SyntaxNode, Token};

pub type TsTokenKind = &'static str;

#[derive(Debug, Clone)]
pub struct TsAnalysisOutput {
    pub source: SourceText,
    pub syntax: SyntaxNode,
    pub tokens: Vec<Token<TsTokenKind>>,
    pub facts: TsFacts,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct TsFacts {
    pub import_count: usize,
    pub export_count: usize,
    pub names: Vec<TsNameFact>,
    pub dispatch_branches: Vec<TsDispatchBranchFact>,
    pub imports: Vec<TsImportFact>,
    pub jsx_elements: Vec<TsxElementFact>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TsNameKind {
    Function,
    Method,
    Binding,
    Parameter,
}

impl TsNameKind {
    pub fn label(self) -> &'static str {
        match self {
            TsNameKind::Function => "function",
            TsNameKind::Method => "method",
            TsNameKind::Binding => "binding",
            TsNameKind::Parameter => "parameter",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TsNameFact {
    pub kind: TsNameKind,
    pub name: String,
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TsDispatchBranchFact {
    pub span: Span,
    pub line: usize,
    pub lines: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TsImportFact {
    pub source: String,
    pub type_only: bool,
    pub specifiers: Vec<TsImportSpecifier>,
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TsImportSpecifier {
    Default(String),
    Named(String),
    Namespace(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TsxElementFact {
    pub name: String,
    pub root: Option<String>,
    pub intrinsic: Option<String>,
    pub self_closing: bool,
    pub span: Span,
    pub line: usize,
}
