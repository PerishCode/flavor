use serde::{Deserialize, Serialize};

use crate::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub span: Option<Span>,
    pub message: String,
}

impl Diagnostic {
    pub fn error(span: impl Into<Option<Span>>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            span: span.into(),
            message: message.into(),
        }
    }
}
