use flavor_core::Fact;
use tracing::debug;

use crate::{
    config::RuleSettings,
    model::{issue, Issue},
    plugins::helper::{self, IssueAggregation},
};

const PATTERN_KEY: &str = "shape.repeated_token_pattern";

pub(super) fn check_repeated_token_patterns<'a>(
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
    path: &str,
    patterns: impl Iterator<Item = &'a Fact>,
) {
    if !rule.enabled {
        return;
    }
    let mut aggregation = IssueAggregation::default();
    for pattern in patterns {
        let occurrences = pattern.usize("occurrences").unwrap_or_default();
        let total_lines = pattern.usize("total_lines").unwrap_or_default();
        let token_count = pattern.usize("token_count").unwrap_or_default();
        let key = helper::line_key("rust", PATTERN_KEY, path, pattern.line);
        if !aggregation.accepts(key.as_str()) {
            debug!(
                path,
                line = pattern.line,
                occurrences,
                total_lines,
                token_count,
                aggregation_key = key,
                "aggregated repeated Rust token pattern",
            );
            continue;
        }
        debug!(
            path,
            line = pattern.line,
            occurrences,
            total_lines,
            token_count,
            aggregation_key = key,
            "reported repeated Rust token pattern",
        );
        issues.push(issue(
            rule.severity,
            rule.id,
            path,
            pattern.line,
            format!(
                "repeated Rust token pattern appears {occurrences} times across {total_lines} lines ({token_count} normalized token(s) per occurrence)"
            ),
        ));
    }
}
