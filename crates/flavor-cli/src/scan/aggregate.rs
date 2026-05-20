use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{
    config::{GuardConfig, NodeKind, RuleSettings},
    model::{issue, Issue},
    path_match::path_string,
    plugins::FailureSurfaceSignal,
    rules::{
        ERROR_FAILURE_SURFACE_AGGREGATE, PAYLOAD_MAX_AGGREGATE_DIRS,
        PAYLOAD_MAX_DIRECTORY_RAW_FAILURE_RATIO_PERCENT, PAYLOAD_MAX_EXAMPLE_FILES,
        PAYLOAD_MAX_RAW_FAILURE_RATIO_PERCENT, PAYLOAD_MIN_DIRECTORY_RAW_FAILURES,
        PAYLOAD_MIN_DIRECTORY_RAW_FILES, PAYLOAD_MIN_DIRECTORY_SCANNED_FILES,
        PAYLOAD_MIN_RAW_FAILURES, PAYLOAD_MIN_RAW_FILES, PAYLOAD_MIN_SCANNED_FILES,
    },
};

const ROOT_DIR: &str = ".";

pub(super) fn check_failure_surface_aggregate(
    config: &GuardConfig,
    scanned_files: usize,
    scanned_files_by_dir: &BTreeMap<PathBuf, usize>,
    signals: &[FailureSurfaceSignal],
    issues: &mut Vec<Issue>,
) {
    check_root_aggregate(config, scanned_files, signals, issues);
    check_directory_aggregates(config, scanned_files_by_dir, signals, issues);
}

fn check_root_aggregate(
    config: &GuardConfig,
    scanned_files: usize,
    signals: &[FailureSurfaceSignal],
    issues: &mut Vec<Issue>,
) {
    let rule = config.rule(
        Path::new(ROOT_DIR),
        NodeKind::Dir,
        ERROR_FAILURE_SURFACE_AGGREGATE,
    );
    let signals = signals.iter().collect::<Vec<_>>();
    if let Some(finding) = aggregate_finding(
        &rule,
        ROOT_DIR,
        scanned_files,
        &signals,
        root_thresholds(&rule),
    ) {
        issues.push(finding.issue);
    }
}

fn check_directory_aggregates(
    config: &GuardConfig,
    scanned_files_by_dir: &BTreeMap<PathBuf, usize>,
    signals: &[FailureSurfaceSignal],
    issues: &mut Vec<Issue>,
) {
    let root_rule = config.rule(
        Path::new(ROOT_DIR),
        NodeKind::Dir,
        ERROR_FAILURE_SURFACE_AGGREGATE,
    );
    let max_dirs = root_rule.usize(PAYLOAD_MAX_AGGREGATE_DIRS).unwrap_or(3);
    if max_dirs == 0 {
        return;
    }

    let mut candidates = Vec::new();
    for (dir, signals) in signals_by_directory(signals) {
        if is_root_dir(&dir) {
            continue;
        }
        let rule = config.rule(&dir, NodeKind::Dir, ERROR_FAILURE_SURFACE_AGGREGATE);
        let scanned_files = scanned_files_by_dir.get(&dir).copied().unwrap_or_default();
        if let Some(finding) = aggregate_finding(
            &rule,
            path_string(&dir),
            scanned_files,
            &signals,
            directory_thresholds(&rule),
        ) {
            candidates.push(AggregateCandidate {
                path: dir,
                raw_sites: finding.raw_sites,
                issue: finding.issue,
            });
        }
    }

    candidates.sort_by(|left, right| {
        path_depth(&right.path)
            .cmp(&path_depth(&left.path))
            .then_with(|| right.raw_sites.cmp(&left.raw_sites))
            .then_with(|| path_string(&left.path).cmp(&path_string(&right.path)))
    });

    let mut emitted_dirs: Vec<PathBuf> = Vec::new();
    for candidate in candidates {
        if emitted_dirs
            .iter()
            .any(|emitted| is_ancestor_dir(&candidate.path, emitted))
        {
            continue;
        }
        issues.push(candidate.issue);
        emitted_dirs.push(candidate.path);
        if emitted_dirs.len() >= max_dirs {
            break;
        }
    }
}

#[derive(Debug)]
struct AggregateCandidate {
    path: PathBuf,
    raw_sites: usize,
    issue: Issue,
}

#[derive(Debug)]
struct AggregateFinding {
    raw_sites: usize,
    issue: Issue,
}

#[derive(Debug, Clone, Copy)]
struct AggregateThresholds {
    min_scanned_files: usize,
    min_raw_sites: usize,
    min_raw_files: usize,
    max_raw_ratio: usize,
    max_example_files: usize,
}

fn aggregate_finding(
    rule: &RuleSettings,
    path: impl Into<String>,
    scanned_files: usize,
    signals: &[&FailureSurfaceSignal],
    thresholds: AggregateThresholds,
) -> Option<AggregateFinding> {
    if !rule.enabled || scanned_files < thresholds.min_scanned_files {
        return None;
    }

    let raw_sites = signals.iter().map(|signal| signal.raw_count).sum::<usize>();
    let raw_files = signals.iter().filter(|signal| signal.raw_count > 0).count();
    let structured_sites = signals
        .iter()
        .map(|signal| signal.structured_count)
        .sum::<usize>();

    if raw_sites < thresholds.min_raw_sites || raw_files < thresholds.min_raw_files {
        return None;
    }

    let total_sites = raw_sites + structured_sites;
    let raw_ratio = raw_sites.saturating_mul(100) / total_sites.max(1);
    if raw_ratio <= thresholds.max_raw_ratio {
        return None;
    }

    Some(AggregateFinding {
        raw_sites,
        issue: issue(
            rule.severity,
            rule.id,
            path,
            None,
            aggregate_message(
                scanned_files,
                raw_sites,
                raw_files,
                structured_sites,
                raw_ratio,
                thresholds.max_raw_ratio,
                top_raw_files(signals, thresholds.max_example_files),
            ),
        ),
    })
}

fn root_thresholds(rule: &RuleSettings) -> AggregateThresholds {
    AggregateThresholds {
        min_scanned_files: rule.usize(PAYLOAD_MIN_SCANNED_FILES).unwrap_or(10),
        min_raw_sites: rule.usize(PAYLOAD_MIN_RAW_FAILURES).unwrap_or(12),
        min_raw_files: rule.usize(PAYLOAD_MIN_RAW_FILES).unwrap_or(4),
        max_raw_ratio: rule
            .usize(PAYLOAD_MAX_RAW_FAILURE_RATIO_PERCENT)
            .unwrap_or(70),
        max_example_files: rule.usize(PAYLOAD_MAX_EXAMPLE_FILES).unwrap_or(5),
    }
}

fn directory_thresholds(rule: &RuleSettings) -> AggregateThresholds {
    AggregateThresholds {
        min_scanned_files: rule.usize(PAYLOAD_MIN_DIRECTORY_SCANNED_FILES).unwrap_or(6),
        min_raw_sites: rule.usize(PAYLOAD_MIN_DIRECTORY_RAW_FAILURES).unwrap_or(8),
        min_raw_files: rule.usize(PAYLOAD_MIN_DIRECTORY_RAW_FILES).unwrap_or(3),
        max_raw_ratio: rule
            .usize(PAYLOAD_MAX_DIRECTORY_RAW_FAILURE_RATIO_PERCENT)
            .unwrap_or(70),
        max_example_files: rule.usize(PAYLOAD_MAX_EXAMPLE_FILES).unwrap_or(5),
    }
}

fn aggregate_message(
    scanned_files: usize,
    raw_sites: usize,
    raw_files: usize,
    structured_sites: usize,
    raw_ratio: usize,
    max_ratio: usize,
    top_files: Vec<String>,
) -> String {
    let top = if top_files.is_empty() {
        String::new()
    } else {
        format!("; top files: {}", top_files.join(", "))
    };
    format!(
        "raw failure construction appears {raw_sites} time(s) across {raw_files} file(s) in {scanned_files} scanned file(s) ({raw_ratio}% of observed failure surface); structured failure surface appears {structured_sites} time(s); max raw ratio is {max_ratio}%{top}"
    )
}

fn top_raw_files(signals: &[&FailureSurfaceSignal], max_files: usize) -> Vec<String> {
    let mut signals = signals
        .iter()
        .filter(|signal| signal.raw_count > 0)
        .copied()
        .collect::<Vec<_>>();
    signals.sort_by(|left, right| {
        right
            .raw_count
            .cmp(&left.raw_count)
            .then_with(|| left.path.cmp(&right.path))
    });
    signals
        .into_iter()
        .take(max_files)
        .map(file_summary)
        .collect()
}

fn file_summary(signal: &FailureSurfaceSignal) -> String {
    if signal.examples.is_empty() {
        format!("{} ({})", signal.path, signal.raw_count)
    } else {
        format!(
            "{} ({}, e.g. {})",
            signal.path,
            signal.raw_count,
            signal.examples.join(", ")
        )
    }
}

fn signals_by_directory(
    signals: &[FailureSurfaceSignal],
) -> BTreeMap<PathBuf, Vec<&FailureSurfaceSignal>> {
    let mut grouped: BTreeMap<PathBuf, Vec<&FailureSurfaceSignal>> = BTreeMap::new();
    for signal in signals {
        for dir in ancestor_dirs(Path::new(&signal.path)) {
            grouped.entry(dir).or_default().push(signal);
        }
    }
    grouped
}

fn ancestor_dirs(path: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut current = normalize_dir(path.parent().unwrap_or_else(|| Path::new(ROOT_DIR)));
    loop {
        dirs.push(current.clone());
        if is_root_dir(&current) {
            break;
        }
        current = normalize_dir(current.parent().unwrap_or_else(|| Path::new(ROOT_DIR)));
    }
    dirs
}

fn normalize_dir(path: &Path) -> PathBuf {
    if path.as_os_str().is_empty() {
        PathBuf::from(ROOT_DIR)
    } else {
        path.to_path_buf()
    }
}

fn is_root_dir(path: &Path) -> bool {
    path.as_os_str() == ROOT_DIR
}

fn is_ancestor_dir(ancestor: &Path, descendant: &Path) -> bool {
    !is_root_dir(ancestor) && ancestor != descendant && descendant.starts_with(ancestor)
}

fn path_depth(path: &Path) -> usize {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .filter(|segment| !segment.is_empty() && *segment != ROOT_DIR)
        .count()
}
