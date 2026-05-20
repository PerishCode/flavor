use crate::{config::RuleSettings, model::Issue, naming::check_name, plugins::ProductSet};

const NAME_FACT_KEYS: &[&str] = &[
    "name.function",
    "name.method",
    "name.binding",
    "name.parameter",
];

pub(super) fn check_name_facts(
    products: &ProductSet,
    grammar_id: &'static str,
    rule: &RuleSettings,
    path: &str,
    issues: &mut Vec<Issue>,
) {
    for key in NAME_FACT_KEYS {
        for fact in products.facts(grammar_id, key) {
            let Some(name) = fact.text("name") else {
                continue;
            };
            let Some(line) = fact.line else {
                continue;
            };
            let kind = fact
                .text("issue_kind")
                .or_else(|| fact.text("kind"))
                .unwrap_or("name");
            check_name(issues, rule, path, line, kind, name);
        }
    }
}
