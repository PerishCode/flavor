use std::{collections::BTreeSet, path::Path};

use flavor_core::PendingIssue;

pub const PLUGIN_ID: &str = "flavor-plugin-filesystem";

pub const FS_CHILDREN_SHAPE: &str = "core/fs/children-shape";
pub const FS_FORBIDDEN_EXTENSION: &str = "core/fs/forbidden-extension";
pub const FS_NAME_SHAPE: &str = "core/fs/name-shape";

pub const RULES: &[&str] = &[FS_CHILDREN_SHAPE, FS_FORBIDDEN_EXTENSION, FS_NAME_SHAPE];

#[derive(Debug, Clone)]
pub struct FilePathInput<'a> {
    pub relative: &'a Path,
    pub forbidden_extension: ForbiddenExtensionRule,
    pub name_shape: NameShapeRule<'a>,
}

#[derive(Debug, Clone)]
pub struct ForbiddenExtensionRule {
    pub enabled: bool,
    pub extensions: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct NameShapeRule<'a> {
    pub enabled: bool,
    pub case: Option<&'a str>,
    pub max_words: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct DirectoryChildrenInput<'a> {
    pub relative: &'a Path,
    pub children: &'a BTreeSet<String>,
    pub rule: DirectoryChildrenRule,
}

#[derive(Debug, Clone)]
pub struct DirectoryChildrenRule {
    pub enabled: bool,
    pub required: Vec<String>,
    pub allowed: Option<Vec<String>>,
    pub forbidden: Vec<String>,
}

pub fn analyze_file_path(input: FilePathInput<'_>) -> Vec<PendingIssue> {
    let mut issues = Vec::new();
    check_forbidden_extension(&input, &mut issues);
    check_name_shape(&input, &mut issues);
    issues
}

pub fn analyze_directory_children(input: DirectoryChildrenInput<'_>) -> Vec<PendingIssue> {
    let mut issues = Vec::new();
    if !input.rule.enabled {
        return issues;
    }
    report_missing_children(&input, &mut issues);
    report_unexpected_children(&input, &mut issues);
    report_forbidden_children(&input, &mut issues);
    issues
}

fn check_forbidden_extension(input: &FilePathInput<'_>, issues: &mut Vec<PendingIssue>) {
    if !input.forbidden_extension.enabled {
        return;
    }
    let forbidden = normalized_extensions(input.forbidden_extension.extensions.as_deref());
    let Some(extension) = input
        .relative
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.trim_start_matches('.').to_ascii_lowercase())
    else {
        return;
    };
    if !forbidden.contains(&extension) {
        return;
    }

    issues.push(PendingIssue::new(
        FS_FORBIDDEN_EXTENSION,
        path_string(input.relative),
        None,
        format!("file extension `.{extension}` is forbidden in this boundary"),
    ));
}

fn check_name_shape(input: &FilePathInput<'_>, issues: &mut Vec<PendingIssue>) {
    if !input.name_shape.enabled {
        return;
    }
    let Some(name) = input.relative.file_stem().and_then(|value| value.to_str()) else {
        return;
    };
    if input.name_shape.case == Some("pascal") && !is_pascal_case(name) {
        issues.push(PendingIssue::new(
            FS_NAME_SHAPE,
            path_string(input.relative),
            None,
            format!("file name `{name}` should be PascalCase"),
        ));
    }
    if let Some(max_words) = input.name_shape.max_words {
        let word_count = file_name_word_count(name);
        if word_count > max_words {
            issues.push(PendingIssue::new(
                FS_NAME_SHAPE,
                path_string(input.relative),
                None,
                format!("file name `{name}` has {word_count} words; max is {max_words}"),
            ));
        }
    }
}

fn report_missing_children(input: &DirectoryChildrenInput<'_>, issues: &mut Vec<PendingIssue>) {
    let missing = input
        .rule
        .required
        .iter()
        .filter(|child| !input.children.contains(*child))
        .cloned()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return;
    }
    issues.push(PendingIssue::new(
        FS_CHILDREN_SHAPE,
        path_string(input.relative),
        None,
        format!(
            "directory is missing required direct children: {}",
            missing.join(", ")
        ),
    ));
}

fn report_unexpected_children(input: &DirectoryChildrenInput<'_>, issues: &mut Vec<PendingIssue>) {
    let Some(allowed) = input.rule.allowed.as_ref() else {
        return;
    };
    let allowed = allowed.iter().collect::<BTreeSet<_>>();
    let unexpected = input
        .children
        .iter()
        .filter(|child| !allowed.contains(*child))
        .cloned()
        .collect::<Vec<_>>();
    if unexpected.is_empty() {
        return;
    }
    issues.push(PendingIssue::new(
        FS_CHILDREN_SHAPE,
        path_string(input.relative),
        None,
        format!(
            "directory has unexpected direct children: {}",
            unexpected.join(", ")
        ),
    ));
}

fn report_forbidden_children(input: &DirectoryChildrenInput<'_>, issues: &mut Vec<PendingIssue>) {
    let forbidden = input
        .rule
        .forbidden
        .iter()
        .filter(|child| input.children.contains(*child))
        .cloned()
        .collect::<Vec<_>>();
    if forbidden.is_empty() {
        return;
    }
    issues.push(PendingIssue::new(
        FS_CHILDREN_SHAPE,
        path_string(input.relative),
        None,
        format!(
            "directory contains forbidden direct children: {}",
            forbidden.join(", ")
        ),
    ));
}

fn normalized_extensions(values: Option<&[String]>) -> BTreeSet<String> {
    values
        .unwrap_or_default()
        .iter()
        .map(|value| value.trim_start_matches('.').to_ascii_lowercase())
        .collect()
}

fn file_name_word_count(name: &str) -> usize {
    name.split('.').map(count_name_words).sum()
}

fn count_name_words(name: &str) -> usize {
    split_name_words(name).len()
}

fn split_name_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let normalized = name
        .strip_prefix("r#")
        .unwrap_or(name)
        .trim_matches('_')
        .trim_matches('$');

    for part in normalized
        .split(['_', '-', '$'])
        .filter(|part| !part.is_empty())
    {
        split_camel_part(part, &mut words);
    }

    words
}

fn split_camel_part(part: &str, words: &mut Vec<String>) {
    let chars: Vec<char> = part.chars().collect();
    let mut current = String::new();

    for (index, ch) in chars.iter().enumerate() {
        if should_start_word(&chars, index) && !current.is_empty() {
            words.push(current);
            current = String::new();
        }
        current.push(*ch);
    }

    if !current.is_empty() {
        words.push(current);
    }
}

fn should_start_word(chars: &[char], index: usize) -> bool {
    if index == 0 || !chars[index].is_uppercase() {
        return false;
    }

    let prev = chars[index - 1];
    let next = chars.get(index + 1).copied();

    prev.is_lowercase()
        || prev.is_ascii_digit()
        || (prev.is_uppercase() && next.is_some_and(char::is_lowercase))
}

fn is_pascal_case(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_uppercase() && chars.all(|ch| ch.is_ascii_alphanumeric())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
