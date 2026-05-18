#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PendingIssue {
    pub rule_id: &'static str,
    pub path: String,
    pub line: Option<usize>,
    pub message: String,
}

impl PendingIssue {
    pub fn new(
        rule_id: &'static str,
        path: impl Into<String>,
        line: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule_id,
            path: path.into(),
            line,
            message: message.into(),
        }
    }
}
