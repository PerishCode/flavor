use std::{collections::BTreeSet, path::Path};

use flavor_plugin_core::PendingIssue;

pub const PLUGIN_ID: &str = "flavor-plugin-source-structure";

pub const FS_TOO_MANY_CHILDREN: &str = "core/fs/too-many-children";
pub const SOURCE_TOO_DEEP: &str = "core/source/too-deep";
pub const SOURCE_TOO_LONG: &str = "core/source/too-long";

pub const RULES: &[&str] = &[FS_TOO_MANY_CHILDREN, SOURCE_TOO_DEEP, SOURCE_TOO_LONG];

#[derive(Debug, Clone)]
pub struct SourceFileInput<'a> {
    pub path: &'a str,
    pub source: &'a str,
    pub rule: SourceFileRule,
}

#[derive(Debug, Clone)]
pub struct SourceFileRule {
    pub enabled: bool,
    pub max_lines: usize,
}

#[derive(Debug, Clone)]
pub struct SourceDirectoryInput<'a> {
    pub relative: &'a Path,
    pub rule: SourceDirectoryRule,
}

#[derive(Debug, Clone)]
pub struct SourceDirectoryRule {
    pub enabled: bool,
    pub max_depth: usize,
}

#[derive(Debug, Clone)]
pub struct DirectoryChildrenInput<'a> {
    pub relative: &'a Path,
    pub source_child_count: usize,
    pub children: &'a BTreeSet<String>,
    pub rule: DirectoryChildrenRule,
}

#[derive(Debug, Clone)]
pub struct DirectoryChildrenRule {
    pub enabled: bool,
    pub max_children: usize,
}

pub fn analyze_source_file(input: SourceFileInput<'_>) -> Vec<PendingIssue> {
    let line_count = input.source.lines().count();
    if input.rule.enabled && line_count > input.rule.max_lines {
        return vec![PendingIssue::new(
            SOURCE_TOO_LONG,
            input.path,
            None,
            format!(
                "source file has {line_count} lines; max is {}",
                input.rule.max_lines
            ),
        )];
    }
    Vec::new()
}

pub fn analyze_source_directory(input: SourceDirectoryInput<'_>) -> Vec<PendingIssue> {
    if !input.rule.enabled {
        return Vec::new();
    }
    let Some(depth) = source_depth(input.relative) else {
        return Vec::new();
    };
    if depth <= input.rule.max_depth {
        return Vec::new();
    }

    vec![PendingIssue::new(
        SOURCE_TOO_DEEP,
        path_string(input.relative),
        None,
        format!(
            "source directory depth is {depth}; warning threshold is {}",
            input.rule.max_depth
        ),
    )]
}

pub fn analyze_directory_children(input: DirectoryChildrenInput<'_>) -> Vec<PendingIssue> {
    if input.rule.enabled && input.source_child_count > input.rule.max_children {
        return vec![PendingIssue::new(
            FS_TOO_MANY_CHILDREN,
            path_string(input.relative),
            None,
            format!(
                "directory has {} source children; max is {}",
                input.source_child_count, input.rule.max_children
            ),
        )];
    }
    Vec::new()
}

fn source_depth(relative: &Path) -> Option<usize> {
    let mut depth = None;
    for component in relative.components() {
        if let Some(depth) = depth.as_mut() {
            *depth += 1;
            continue;
        }
        if component.as_os_str() == "src" {
            depth = Some(0);
        }
    }
    depth
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
