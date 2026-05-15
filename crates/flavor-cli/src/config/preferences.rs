use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::{
    config::{MatchKind, MatchPatterns, RuleMatcher, RuleOverride},
    path_match::PathPattern,
    rules::{
        FS_CHILDREN_SHAPE, FS_FORBIDDEN_EXTENSION, FS_NAME_SHAPE, FS_TOO_MANY_CHILDREN,
        PAYLOAD_ALLOWED, PAYLOAD_ALLOWED_INTRINSICS, PAYLOAD_CASE, PAYLOAD_EXTENSIONS,
        PAYLOAD_FORBIDDEN, PAYLOAD_MAX_WORDS, PAYLOAD_PRIMITIVE_SOURCES, PAYLOAD_REQUIRED,
        TSX_NO_INTRINSICS, TSX_REQUIRES_PRIMITIVE,
    },
};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PreferenceConfigFile {
    name: String,
    #[serde(rename = "match")]
    matches: MatchPatterns,
    #[serde(default)]
    priority: Option<i32>,
    #[serde(default, rename = "primitiveSources")]
    primitive_sources: Vec<String>,
    #[serde(default, rename = "allowedIntrinsics")]
    allowed_intrinsics: Vec<String>,
}

pub(crate) fn expand(items: Vec<PreferenceConfigFile>) -> Result<Vec<RuleMatcher>, String> {
    let mut matchers = Vec::new();
    for (index, item) in items.into_iter().enumerate() {
        match item.name.as_str() {
            "frontend/renderer-boundary" => {
                expand_renderer_boundary(index, item, &mut matchers)?;
            }
            other => {
                return Err(format!(
                    "unknown flavor preference set: {other}; known preferences: frontend/renderer-boundary"
                ));
            }
        }
    }
    matchers.sort_by_key(|item| (item.priority, item.order));
    Ok(matchers)
}

fn expand_renderer_boundary(
    index: usize,
    item: PreferenceConfigFile,
    matchers: &mut Vec<RuleMatcher>,
) -> Result<(), String> {
    let roots = preference_roots(index, item.matches)?;
    let primitive_sources = checked_list(
        item.primitive_sources,
        "frontend/renderer-boundary primitiveSources",
    )?;
    if primitive_sources.is_empty() {
        return Err(
            "preference frontend/renderer-boundary requires non-empty primitiveSources".to_string(),
        );
    }
    let allowed_intrinsics = checked_list(
        item.allowed_intrinsics,
        "frontend/renderer-boundary allowedIntrinsics",
    )?;
    let priority = item.priority.unwrap_or_default();
    let mut order = index * 10;

    push_matcher(
        matchers,
        roots.iter().map(|root| root.to_string()).collect(),
        MatchKind::Dir,
        priority,
        order,
        rule(
            FS_CHILDREN_SHAPE,
            payload([
                (
                    PAYLOAD_REQUIRED,
                    list(["lib", "components", "views", "app.tsx", "main.tsx"]),
                ),
                (
                    PAYLOAD_ALLOWED,
                    list([
                        "lib",
                        "components",
                        "views",
                        "app.tsx",
                        "main.tsx",
                        "env.d.ts",
                    ]),
                ),
                (
                    PAYLOAD_FORBIDDEN,
                    list(["styles", "atoms", "primitives", "ui"]),
                ),
            ]),
        ),
    );
    order += 1;

    push_matcher(
        matchers,
        roots.iter().map(|root| root.to_string()).collect(),
        MatchKind::Dir,
        priority,
        order,
        (
            FS_TOO_MANY_CHILDREN,
            RuleOverride::disabled(
                "renderer-boundary direct child shape supersedes generic child-count pressure",
            ),
        ),
    );
    order += 1;

    push_matcher(
        matchers,
        suffixed_patterns(&roots, "**"),
        MatchKind::File,
        priority,
        order,
        rule(
            FS_FORBIDDEN_EXTENSION,
            payload([(PAYLOAD_EXTENSIONS, list(["css", "scss", "sass"]))]),
        ),
    );
    order += 1;

    push_matcher(
        matchers,
        suffixed_patterns(&roots, "**/*.tsx"),
        MatchKind::File,
        priority,
        order,
        rule(
            TSX_NO_INTRINSICS,
            payload([(
                PAYLOAD_ALLOWED_INTRINSICS,
                list(allowed_intrinsics.iter().map(String::as_str)),
            )]),
        ),
    );
    order += 1;

    push_matcher(
        matchers,
        suffixed_patterns(&roots, "components/**/*.tsx"),
        MatchKind::File,
        priority,
        order,
        rule(
            TSX_REQUIRES_PRIMITIVE,
            payload([(
                PAYLOAD_PRIMITIVE_SOURCES,
                list(primitive_sources.iter().map(String::as_str)),
            )]),
        ),
    );
    order += 1;

    push_matcher(
        matchers,
        suffixed_patterns(&roots, "components/**/*.tsx")
            .into_iter()
            .chain(suffixed_patterns(&roots, "views/**/*.tsx"))
            .collect(),
        MatchKind::File,
        priority,
        order,
        rule(
            FS_NAME_SHAPE,
            payload([(PAYLOAD_CASE, Value::from("pascal"))]),
        ),
    );
    order += 1;

    push_matcher(
        matchers,
        suffixed_patterns(&roots, "lib/**"),
        MatchKind::File,
        priority,
        order,
        rule(
            FS_NAME_SHAPE,
            payload([(PAYLOAD_MAX_WORDS, Value::from(1))]),
        ),
    );

    Ok(())
}

fn preference_roots(index: usize, matches: MatchPatterns) -> Result<Vec<String>, String> {
    let roots = checked_list(matches.into_vec(), "preference match")?;
    if roots.is_empty() {
        return Err(format!(
            "preference at index {index} has empty 'match'; use a glob string or a non-empty array"
        ));
    }
    Ok(roots)
}

fn checked_list(values: Vec<String>, field: &str) -> Result<Vec<String>, String> {
    let mut checked = Vec::with_capacity(values.len());
    for value in values {
        let value = value.trim().to_string();
        if value.is_empty() {
            return Err(format!("{field} contains an empty string"));
        }
        checked.push(value);
    }
    Ok(checked)
}

fn suffixed_patterns(roots: &[String], suffix: &str) -> Vec<String> {
    roots
        .iter()
        .map(|root| {
            let root = root.trim_end_matches('/');
            if suffix.is_empty() {
                root.to_string()
            } else {
                format!("{root}/{suffix}")
            }
        })
        .collect()
}

fn push_matcher(
    matchers: &mut Vec<RuleMatcher>,
    raw_patterns: Vec<String>,
    kind: MatchKind,
    priority: i32,
    order: usize,
    rule: (&'static str, RuleOverride),
) {
    let patterns = raw_patterns
        .iter()
        .map(|pattern| PathPattern::new(pattern))
        .collect();
    let mut rules = BTreeMap::new();
    rules.insert(rule.0.to_string(), rule.1);
    matchers.push(RuleMatcher {
        patterns,
        kind,
        priority,
        order,
        rules,
    });
}

fn rule(rule_id: &'static str, payload: BTreeMap<String, Value>) -> (&'static str, RuleOverride) {
    (rule_id, RuleOverride::enabled_with_payload(payload))
}

fn payload<const N: usize>(entries: [(&'static str, Value); N]) -> BTreeMap<String, Value> {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

fn list<'a>(values: impl IntoIterator<Item = &'a str>) -> Value {
    Value::Array(values.into_iter().map(Value::from).collect())
}
