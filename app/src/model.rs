use std::{collections::BTreeSet, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::rules;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Severity {
    Deny,
    Warning,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(crate) struct Issue {
    pub(crate) severity: Severity,
    pub(crate) rule: &'static str,
    pub(crate) path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) line: Option<usize>,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(crate) struct RuleGuide {
    pub(crate) rule: &'static str,
    pub(crate) bad_flavor: &'static str,
    pub(crate) action_hint: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct Report {
    pub(crate) root: String,
    pub(crate) guidance: Vec<RuleGuide>,
    pub(crate) issues: Vec<Issue>,
}

impl Report {
    pub(crate) fn new(root: PathBuf, issues: Vec<Issue>) -> Self {
        let guidance = guides_for(&issues);
        Self {
            root: root.display().to_string(),
            guidance,
            issues,
        }
    }

    pub(crate) fn deny_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Deny)
            .count()
    }

    pub(crate) fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Warning)
            .count()
    }
}

fn guides_for(issues: &[Issue]) -> Vec<RuleGuide> {
    issues
        .iter()
        .map(|issue| issue.rule)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(rule_guide)
        .collect()
}

fn rule_guide(rule: &str) -> Option<RuleGuide> {
    rules::descriptor(rule).map(|descriptor| RuleGuide {
        rule: descriptor.id,
        bad_flavor: descriptor.bad_flavor,
        action_hint: descriptor.action_hint,
    })
}

pub(crate) fn issue(
    severity: Severity,
    rule: &'static str,
    path: impl Into<String>,
    line: Option<usize>,
    message: impl Into<String>,
) -> Issue {
    Issue {
        severity,
        rule,
        path: path.into(),
        line,
        message: message.into(),
    }
}
