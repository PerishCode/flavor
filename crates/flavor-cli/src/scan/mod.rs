use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

mod fs_rules;

use crate::{
    config::{source_file_kind, GuardConfig, NodeKind, SourceKind},
    model::{issue, Issue, ScanStats},
    naming::{check_rust_names, check_svelte_names, check_ts_names},
    path_match::relative_path,
    rules::{
        FS_TOO_MANY_CHILDREN, PAYLOAD_MAX_CHILDREN, PAYLOAD_MAX_DEPTH, PAYLOAD_MAX_LINES,
        RUST_PARSE_ERROR, RUST_TESTS_IN_SOURCE, SOURCE_TOO_DEEP, SOURCE_TOO_LONG,
        SVELTE_PARSE_ERROR, TS_PARSE_ERROR,
    },
    rust_tests::check_rust_test_home,
};

use fs_rules::{check_children_shape, check_file_path_rules, track_direct_child};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ScanResult {
    pub(crate) issues: Vec<Issue>,
    pub(crate) stats: ScanStats,
}

pub(crate) fn run_scan(config: &GuardConfig) -> Result<ScanResult, String> {
    let root = canonical_root(&config.root)?;
    let mut issues = Vec::new();
    let mut stats = ScanStats::default();
    let mut child_counts = BTreeMap::<PathBuf, usize>::new();
    let mut direct_children = BTreeMap::<PathBuf, BTreeSet<String>>::new();

    let mut walk = WalkDir::new(&root).into_iter();
    while let Some(entry) = walk.next() {
        let entry = entry.map_err(|error| format!("failed to walk source tree: {error}"))?;
        let path = entry.path();
        let relative = relative_path(&root, path)?;

        if config.is_excluded(&relative) {
            stats.excluded_entries += 1;
            if entry.file_type().is_dir() {
                walk.skip_current_dir();
            }
            continue;
        }

        let scanned_entry = config.is_scanned(&relative);
        if scanned_entry {
            track_direct_child(&mut direct_children, &relative);
        }

        if entry.file_type().is_dir() {
            if scanned_entry {
                direct_children.entry(relative.clone()).or_default();
                check_source_depth(config, &relative, &mut issues);
                add_child_count(&mut child_counts, &relative);
            }
            continue;
        }

        if !scanned_entry {
            continue;
        }
        stats.matched_files += 1;
        check_file_path_rules(config, &relative, &mut issues);

        let Some(kind) = source_file_kind(&relative) else {
            stats.unsupported_files += 1;
            continue;
        };
        if is_generated_source(path)? {
            stats.generated_files += 1;
            continue;
        }

        stats.scanned_files += 1;
        add_child_count(&mut child_counts, &relative);
        check_source_file(config, &relative, path, kind, &mut issues)?;
    }

    for (dir, children) in &direct_children {
        check_children_shape(config, dir, children, &mut issues);
    }

    for (dir, count) in child_counts {
        let rule = config.rule(&dir, NodeKind::Dir, FS_TOO_MANY_CHILDREN);
        let max_children = rule.usize(PAYLOAD_MAX_CHILDREN).unwrap_or(10);
        if rule.enabled && count > max_children {
            issues.push(issue(
                rule.severity,
                rule.id,
                path_string(&dir),
                None,
                format!("directory has {count} source children; max is {max_children}"),
            ));
        }
    }

    issues.sort_by(|left, right| {
        (left.path.as_str(), left.line.unwrap_or(0), left.rule).cmp(&(
            right.path.as_str(),
            right.line.unwrap_or(0),
            right.rule,
        ))
    });
    Ok(ScanResult { issues, stats })
}

fn canonical_root(root: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(root)
        .map_err(|error| format!("failed to resolve root {}: {error}", root.display()))
}

fn check_source_depth(config: &GuardConfig, relative: &Path, issues: &mut Vec<Issue>) {
    let rule = config.rule(relative, NodeKind::Dir, SOURCE_TOO_DEEP);
    if !rule.enabled {
        return;
    }
    let Some(depth) = source_depth(relative) else {
        return;
    };
    let max_depth = rule.usize(PAYLOAD_MAX_DEPTH).unwrap_or(4);
    if depth <= max_depth {
        return;
    }

    issues.push(issue(
        rule.severity,
        rule.id,
        path_string(relative),
        None,
        format!("source directory depth is {depth}; warning threshold is {max_depth}"),
    ));
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

fn add_child_count(child_counts: &mut BTreeMap<PathBuf, usize>, relative: &Path) {
    let Some(parent) = relative.parent() else {
        return;
    };
    *child_counts.entry(parent.to_path_buf()).or_default() += 1;
}

fn check_source_file(
    config: &GuardConfig,
    relative: &Path,
    path: &Path,
    kind: SourceKind,
    issues: &mut Vec<Issue>,
) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let line_count = source.lines().count();
    let relative_path = path_string(relative);
    let length_rule = config.rule(relative, NodeKind::File, SOURCE_TOO_LONG);
    let max_lines = length_rule.usize(PAYLOAD_MAX_LINES).unwrap_or(500);

    if length_rule.enabled && line_count > max_lines {
        issues.push(issue(
            length_rule.severity,
            length_rule.id,
            relative_path.clone(),
            None,
            format!("source file has {line_count} lines; max is {max_lines}"),
        ));
    }

    match kind {
        SourceKind::Rust => {
            let parse_rule = config.rule(relative, NodeKind::File, RUST_PARSE_ERROR);
            check_rust_names(
                config,
                relative,
                &relative_path,
                &source,
                issues,
                &parse_rule,
            );
            let test_rule = config.rule(relative, NodeKind::File, RUST_TESTS_IN_SOURCE);
            check_rust_test_home(relative, &source, issues, &test_rule);
        }
        SourceKind::Svelte => {
            let svelte_parse_rule = config.rule(relative, NodeKind::File, SVELTE_PARSE_ERROR);
            let ts_parse_rule = config.rule(relative, NodeKind::File, TS_PARSE_ERROR);
            check_svelte_names(
                config,
                relative,
                &relative_path,
                &source,
                issues,
                &svelte_parse_rule,
                &ts_parse_rule,
            );
        }
        SourceKind::TypeScript => {
            let parse_rule = config.rule(relative, NodeKind::File, TS_PARSE_ERROR);
            check_ts_names(
                config,
                relative,
                &relative_path,
                &source,
                issues,
                &parse_rule,
            );
        }
    }

    Ok(())
}

pub(crate) fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn is_generated_source(path: &Path) -> Result<bool, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let header = source
        .lines()
        .take(12)
        .collect::<Vec<_>>()
        .join("\n")
        .to_ascii_lowercase();
    Ok([
        "@generated",
        "generated by",
        "do not edit",
        "code generated",
        "autogenerated",
        "auto-generated",
        "this file was generated",
    ]
    .iter()
    .any(|marker| header.contains(marker)))
}
