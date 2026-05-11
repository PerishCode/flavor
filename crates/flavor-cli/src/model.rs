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
    pub(crate) scan: ScanStats,
    pub(crate) guidance: Vec<RuleGuide>,
    pub(crate) issues: Vec<Issue>,
    #[serde(skip)]
    allow_empty_scan: bool,
}

impl Report {
    pub(crate) fn with_scan(root: PathBuf, scan: ScanStats, issues: Vec<Issue>) -> Self {
        Self::build(root, scan, issues, false)
    }

    /// Construct a report that opts out of the empty-scan failure.
    ///
    /// Used when the active `flavor.json` declared `allowEmptyScan: true` —
    /// typically a workspace-root config that intentionally excludes every
    /// submodule and delegates real checks to per-submodule configs.
    pub(crate) fn with_scan_allow_empty(
        root: PathBuf,
        scan: ScanStats,
        issues: Vec<Issue>,
    ) -> Self {
        Self::build(root, scan, issues, true)
    }

    fn build(root: PathBuf, scan: ScanStats, issues: Vec<Issue>, allow_empty_scan: bool) -> Self {
        let guidance = guides_for(&issues);
        Self {
            root: root.display().to_string(),
            scan,
            guidance,
            issues,
            allow_empty_scan,
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

    /// `true` when scan.include matched no files and the active config has
    /// not opted out of treating that as a failure.
    ///
    /// A 0-match scan is almost always a misconfigured include / exclude
    /// pattern or a wrong --root, and silently exiting 0 makes CI lie. The
    /// `allowEmptyScan` opt-out is reserved for workspace-root configs that
    /// intentionally cover nothing (they exist as walk-up boundaries while
    /// per-submodule configs do the actual work).
    pub(crate) fn is_empty_scan(&self) -> bool {
        self.scan.matched_files == 0 && !self.allow_empty_scan
    }

    /// Final process exit code.
    ///
    /// 1 if any deny issue fired, if `--strict-warnings` is set and warnings
    /// were emitted, or if the scan matched no files (unless the config opted
    /// into allowEmptyScan). 0 otherwise.
    pub(crate) fn exit_code(&self, strict_warnings: bool) -> i32 {
        if self.deny_count() > 0
            || (strict_warnings && self.warning_count() > 0)
            || self.is_empty_scan()
        {
            1
        } else {
            0
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScanStats {
    pub(crate) matched_files: usize,
    pub(crate) scanned_files: usize,
    pub(crate) generated_files: usize,
    pub(crate) unsupported_files: usize,
    pub(crate) excluded_entries: usize,
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
