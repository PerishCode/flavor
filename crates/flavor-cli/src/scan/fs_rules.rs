use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use crate::{
    config::{GuardConfig, NodeKind, RuleSettings},
    model::{issue, Issue},
    naming::count_name_words,
    rules::{
        FS_CHILDREN_SHAPE, FS_FORBIDDEN_EXTENSION, FS_NAME_SHAPE, PAYLOAD_ALLOWED, PAYLOAD_CASE,
        PAYLOAD_EXTENSIONS, PAYLOAD_FORBIDDEN, PAYLOAD_MAX_WORDS, PAYLOAD_REQUIRED,
    },
    scan::path_string,
};

pub(crate) fn check_file_path_rules(
    config: &GuardConfig,
    relative: &Path,
    issues: &mut Vec<Issue>,
) {
    check_forbidden_extension(config, relative, issues);
    check_name_shape(config, relative, issues);
}

pub(crate) fn check_children_shape(
    config: &GuardConfig,
    relative: &Path,
    children: &BTreeSet<String>,
    issues: &mut Vec<Issue>,
) {
    let rule = config.rule(relative, NodeKind::Dir, FS_CHILDREN_SHAPE);
    if !rule.enabled {
        return;
    }
    report_missing_children(relative, children, issues, &rule);
    report_unexpected_children(relative, children, issues, &rule);
    report_forbidden_children(relative, children, issues, &rule);
}

pub(crate) fn track_direct_child(
    child_map: &mut std::collections::BTreeMap<PathBuf, BTreeSet<String>>,
    relative: &Path,
) {
    let Some(parent) = relative.parent() else {
        return;
    };
    let Some(name) = relative.file_name().and_then(|name| name.to_str()) else {
        return;
    };
    child_map
        .entry(parent.to_path_buf())
        .or_default()
        .insert(name.to_string());
}

fn check_forbidden_extension(config: &GuardConfig, relative: &Path, issues: &mut Vec<Issue>) {
    let rule = config.rule(relative, NodeKind::File, FS_FORBIDDEN_EXTENSION);
    if !rule.enabled {
        return;
    }
    let forbidden = normalized_extensions(rule.string_list(PAYLOAD_EXTENSIONS));
    let Some(extension) = relative
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.trim_start_matches('.').to_ascii_lowercase())
    else {
        return;
    };
    if !forbidden.contains(&extension) {
        return;
    }

    issues.push(issue(
        rule.severity,
        rule.id,
        path_string(relative),
        None,
        format!("file extension `.{extension}` is forbidden in this boundary"),
    ));
}

fn check_name_shape(config: &GuardConfig, relative: &Path, issues: &mut Vec<Issue>) {
    let rule = config.rule(relative, NodeKind::File, FS_NAME_SHAPE);
    if !rule.enabled {
        return;
    }
    let Some(name) = relative.file_stem().and_then(|value| value.to_str()) else {
        return;
    };
    if rule.string(PAYLOAD_CASE) == Some("pascal") && !is_pascal_case(name) {
        issues.push(issue(
            rule.severity,
            rule.id,
            path_string(relative),
            None,
            format!("file name `{name}` should be PascalCase"),
        ));
    }
    if let Some(max_words) = rule.usize(PAYLOAD_MAX_WORDS) {
        let word_count = file_name_word_count(name);
        if word_count > max_words {
            issues.push(issue(
                rule.severity,
                rule.id,
                path_string(relative),
                None,
                format!("file name `{name}` has {word_count} words; max is {max_words}"),
            ));
        }
    }
}

fn report_missing_children(
    relative: &Path,
    children: &BTreeSet<String>,
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
) {
    let missing = rule
        .string_list(PAYLOAD_REQUIRED)
        .unwrap_or_default()
        .into_iter()
        .filter(|child| !children.contains(child))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return;
    }
    issues.push(issue(
        rule.severity,
        rule.id,
        path_string(relative),
        None,
        format!(
            "directory is missing required direct children: {}",
            missing.join(", ")
        ),
    ));
}

fn report_unexpected_children(
    relative: &Path,
    children: &BTreeSet<String>,
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
) {
    let Some(allowed) = rule.string_list(PAYLOAD_ALLOWED) else {
        return;
    };
    let allowed = allowed.into_iter().collect::<BTreeSet<_>>();
    let unexpected = children
        .iter()
        .filter(|child| !allowed.contains(*child))
        .cloned()
        .collect::<Vec<_>>();
    if unexpected.is_empty() {
        return;
    }
    issues.push(issue(
        rule.severity,
        rule.id,
        path_string(relative),
        None,
        format!(
            "directory has unexpected direct children: {}",
            unexpected.join(", ")
        ),
    ));
}

fn report_forbidden_children(
    relative: &Path,
    children: &BTreeSet<String>,
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
) {
    let forbidden = rule
        .string_list(PAYLOAD_FORBIDDEN)
        .unwrap_or_default()
        .into_iter()
        .filter(|child| children.contains(child))
        .collect::<Vec<_>>();
    if forbidden.is_empty() {
        return;
    }
    issues.push(issue(
        rule.severity,
        rule.id,
        path_string(relative),
        None,
        format!(
            "directory contains forbidden direct children: {}",
            forbidden.join(", ")
        ),
    ));
}

fn normalized_extensions(values: Option<Vec<String>>) -> BTreeSet<String> {
    values
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.trim_start_matches('.').to_ascii_lowercase())
        .collect()
}

fn file_name_word_count(name: &str) -> usize {
    name.split('.').map(count_name_words).sum()
}

fn is_pascal_case(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_uppercase() && chars.all(|ch| ch.is_ascii_alphanumeric())
}
