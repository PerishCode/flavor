use crate::{
    config::RuleSettings,
    model::{issue, Issue},
    plugins::FailureSurfaceSignal,
    rules::{PAYLOAD_MAX_RAW_FAILURES, PAYLOAD_MAX_RAW_FAILURE_RATIO_PERCENT},
};
use flavor_core::Fact;

pub(super) fn check_failure_surface<'a>(
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
    path: &str,
    raw_failures: impl Iterator<Item = &'a Fact>,
    structured_failures: impl Iterator<Item = &'a Fact>,
) {
    if !rule.enabled {
        return;
    }

    let raw_failures = raw_failures.collect::<Vec<_>>();
    let raw_count = raw_failures.len();
    let max_raw = rule.usize(PAYLOAD_MAX_RAW_FAILURES).unwrap_or(4);
    if raw_count <= max_raw {
        return;
    }

    let structured_count = structured_failures.count();
    let total = raw_count + structured_count;
    let raw_ratio = raw_count.saturating_mul(100) / total.max(1);
    let max_ratio = rule
        .usize(PAYLOAD_MAX_RAW_FAILURE_RATIO_PERCENT)
        .unwrap_or(60);
    if raw_ratio <= max_ratio {
        return;
    }

    let structured_context = if structured_count == 0 {
        "no structured failure surface facts observed".to_string()
    } else {
        format!("{structured_count} structured failure surface fact(s) observed")
    };
    issues.push(issue(
        rule.severity,
        rule.id,
        path,
        raw_failures.first().and_then(|fact| fact.line),
        format!(
            "raw failure construction appears {raw_count} time(s) ({raw_ratio}% of observed failure surface); max raw is {max_raw} and max raw ratio is {max_ratio}%; {structured_context}{}",
            raw_failure_examples(&raw_failures)
        ),
    ));
}

pub(super) fn failure_surface_signal<'a>(
    path: &str,
    raw_failures: impl Iterator<Item = &'a Fact>,
    structured_failures: impl Iterator<Item = &'a Fact>,
) -> Option<FailureSurfaceSignal> {
    let raw_failures = raw_failures.collect::<Vec<_>>();
    let structured_count = structured_failures.count();
    if raw_failures.is_empty() && structured_count == 0 {
        return None;
    }
    Some(FailureSurfaceSignal {
        path: path.to_string(),
        raw_count: raw_failures.len(),
        structured_count,
        examples: raw_failures
            .iter()
            .take(3)
            .filter_map(|fact| raw_failure_example(fact))
            .collect(),
    })
}

fn raw_failure_examples(failures: &[&Fact]) -> String {
    let examples = failures
        .iter()
        .take(3)
        .filter_map(|fact| raw_failure_example(fact))
        .collect::<Vec<_>>();
    if examples.is_empty() {
        String::new()
    } else {
        format!("; examples: {}", examples.join(", "))
    }
}

fn raw_failure_example(fact: &Fact) -> Option<String> {
    let constructor = fact
        .text("constructor")
        .filter(|constructor| !constructor.is_empty());
    let callee = fact.text("callee").filter(|callee| !callee.is_empty());
    match (callee, constructor, fact.text("kind")) {
        (Some(callee), Some(constructor), _) => Some(format!("{callee}(new {constructor})")),
        (_, Some(constructor), _) => Some(format!("new {constructor}")),
        (_, _, Some(kind)) => Some(kind.to_string()),
        _ => None,
    }
}
