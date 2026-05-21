use flavor_core::{Diagnostic, SourceText, Span, SyntaxNode};

#[derive(Debug, Clone)]
pub struct PythonAnalysisOutput {
    pub source: SourceText,
    pub syntax: SyntaxNode,
    pub facts: PythonFacts,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct PythonFacts {
    pub names: Vec<PythonNameFact>,
    pub function_bodies: Vec<PythonFunctionBodyFact>,
    pub dispatch_branches: Vec<PythonDispatchBranchFact>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PythonNameKind {
    Function,
    Method,
    Binding,
    Parameter,
}

impl PythonNameKind {
    pub fn label(self) -> &'static str {
        match self {
            PythonNameKind::Function => "function",
            PythonNameKind::Method => "method",
            PythonNameKind::Binding => "binding",
            PythonNameKind::Parameter => "parameter",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PythonNameFact {
    pub kind: PythonNameKind,
    pub name: String,
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PythonFunctionBodyFact {
    pub name: String,
    pub kind: &'static str,
    pub span: Span,
    pub line: usize,
    pub lines: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PythonDispatchBranchFact {
    pub span: Span,
    pub line: usize,
    pub lines: usize,
}
