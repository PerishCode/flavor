use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use tracing::debug;
use walkdir::WalkDir;

use crate::{
    config::{source_file_kind, GuardConfig, SourceKind},
    model::{Issue, ScanStats},
    path_match::{path_string, relative_path},
    plugins::{PluginHost, Scope},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ScanResult {
    pub(crate) issues: Vec<Issue>,
    pub(crate) stats: ScanStats,
}

pub(crate) fn run_scan(config: &GuardConfig) -> Result<ScanResult, String> {
    let root = canonical_root(&config.root)?;
    debug!(root = %root.display(), "starting scan");
    let host = PluginHost::bundled();
    debug_assert!(!host.manifests().is_empty());
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
            debug!(path = %relative.display(), "excluded path");
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
                host.run_scope(config, Scope::source_directory(&relative), &mut issues);
                add_child_count(&mut child_counts, &relative);
            }
            continue;
        }

        if !scanned_entry {
            continue;
        }
        stats.matched_files += 1;
        host.run_scope(config, Scope::file_path(&relative), &mut issues);

        let Some(kind) = source_file_kind(&relative) else {
            stats.unsupported_files += 1;
            debug!(path = %relative.display(), "unsupported matched file");
            continue;
        };
        if is_generated_source(path)? {
            stats.generated_files += 1;
            debug!(path = %relative.display(), "skipped generated source");
            continue;
        }

        stats.scanned_files += 1;
        debug!(path = %relative.display(), kind = ?kind, "scanning source file");
        add_child_count(&mut child_counts, &relative);
        check_source_file(config, &host, &relative, path, kind, &mut issues)?;
    }

    for (dir, children) in &direct_children {
        let count = child_counts.get(dir).copied().unwrap_or_default();
        host.run_scope(
            config,
            Scope::directory_children(dir, children, count),
            &mut issues,
        );
    }

    issues.sort_by(|left, right| {
        (left.path.as_str(), left.line.unwrap_or(0), left.rule).cmp(&(
            right.path.as_str(),
            right.line.unwrap_or(0),
            right.rule,
        ))
    });
    debug!(
        matched_files = stats.matched_files,
        scanned_files = stats.scanned_files,
        generated_files = stats.generated_files,
        unsupported_files = stats.unsupported_files,
        excluded_entries = stats.excluded_entries,
        issue_count = issues.len(),
        "finished scan",
    );
    Ok(ScanResult { issues, stats })
}

fn canonical_root(root: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(root)
        .map_err(|error| format!("failed to resolve root {}: {error}", root.display()))
}

fn add_child_count(child_counts: &mut BTreeMap<PathBuf, usize>, relative: &Path) {
    let Some(parent) = relative.parent() else {
        return;
    };
    *child_counts.entry(parent.to_path_buf()).or_default() += 1;
}

fn track_direct_child(child_map: &mut BTreeMap<PathBuf, BTreeSet<String>>, relative: &Path) {
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

fn check_source_file(
    config: &GuardConfig,
    host: &PluginHost,
    relative: &Path,
    path: &Path,
    kind: SourceKind,
    issues: &mut Vec<Issue>,
) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let relative_path = path_string(relative);
    host.run_scope(
        config,
        Scope::source_file(relative, &relative_path, &source, kind),
        issues,
    );

    Ok(())
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
