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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub span: Option<Span>,
    pub message: String,
}

impl Diagnostic {
    pub fn error(span: impl Into<Option<Span>>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code: None,
            span: span.into(),
            message: message.into(),
        }
    }

    pub fn error_code(
        span: impl Into<Option<Span>>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code: Some(code.into()),
            span: span.into(),
            message: message.into(),
        }
    }
}
