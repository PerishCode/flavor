use flavor_core::Fact;

use crate::{
    config::RuleSettings,
    model::{issue, Issue},
    rules::PAYLOAD_MAX_LINES,
};

pub(super) fn check_function_bodies<'a>(
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
    path: &str,
    bodies: impl Iterator<Item = &'a Fact>,
) {
    if !rule.enabled {
        return;
    }
    let max_lines = rule.usize(PAYLOAD_MAX_LINES).unwrap_or(80);
    for body in bodies {
        let Some(lines) = body.usize("lines") else {
            continue;
        };
        if lines <= max_lines {
            continue;
        }
        let name = body.text("name").unwrap_or("<anonymous>");
        let kind = body.text("kind").unwrap_or("function");
        issues.push(issue(
            rule.severity,
            rule.id,
            path.to_string(),
            body.line,
            format!("{kind} `{name}` spans {lines} lines; max is {max_lines}"),
        ));
    }
}
