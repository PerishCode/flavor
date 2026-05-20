use crate::{
    config::RuleSettings,
    model::{issue, Issue},
    rules::PAYLOAD_MAX_BRANCH_LINES,
};
use flavor_core::Fact;

pub(super) fn check_dispatch_branches<'a>(
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
    path: &str,
    branches: impl Iterator<Item = &'a Fact>,
    label: &str,
) {
    if !rule.enabled {
        return;
    }
    let max_lines = rule.usize(PAYLOAD_MAX_BRANCH_LINES).unwrap_or(24);
    for branch in branches {
        let Some(lines) = branch.usize("lines") else {
            continue;
        };
        let Some(line) = branch.line else {
            continue;
        };
        if lines > max_lines {
            issues.push(issue(
                rule.severity,
                rule.id,
                path,
                Some(line),
                format!("{label} spans {lines} lines; max is {max_lines}"),
            ));
        }
    }
}
