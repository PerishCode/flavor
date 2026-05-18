use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::Value;

mod preferences;

use crate::{
    model::Severity,
    path_match::PathPattern,
    rules::{self, RuleTarget},
};

#[derive(Debug, Clone)]
pub(crate) struct GuardConfig {
    pub(crate) root: PathBuf,
    scan_include: Vec<PathPattern>,
    scan_exclude: Vec<PathPattern>,
    preferences: Vec<RuleMatcher>,
    overrides: Vec<RuleMatcher>,
    allow_empty_scan: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum NodeKind {
    File,
    Dir,
}

#[derive(Debug, Clone)]
pub(crate) struct RuleSettings {
    pub(crate) id: &'static str,
    pub(crate) enabled: bool,
    pub(crate) severity: Severity,
    payload: BTreeMap<String, Value>,
}

impl RuleSettings {
    #[allow(dead_code)]
    pub(crate) fn bool(&self, key: &str) -> Option<bool> {
        self.payload.get(key).and_then(Value::as_bool)
    }

    pub(crate) fn string(&self, key: &str) -> Option<&str> {
        self.payload.get(key).and_then(Value::as_str)
    }

    pub(crate) fn string_list(&self, key: &str) -> Option<Vec<String>> {
        match self.payload.get(key)? {
            Value::String(value) => Some(vec![value.clone()]),
            Value::Array(values) => values
                .iter()
                .map(|value| value.as_str().map(str::to_string))
                .collect(),
            _ => None,
        }
    }

    pub(crate) fn usize(&self, key: &str) -> Option<usize> {
        self.payload
            .get(key)
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
    }
}

/// File name flavor walks ancestors of `--root` to find.
pub(crate) const DEFAULT_CONFIG_FILENAME: &str = "flavor.json";

/// Where the active `GuardConfig` came from.
///
/// `Explicit` and `Discovered` both point at a file on disk; the split lets
/// callers tell the user when a config was picked up without being asked for,
/// so a stray `flavor.json` somewhere above the scan root never silently
/// changes behavior.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ConfigSource {
    Explicit(PathBuf),
    Discovered(PathBuf),
    BuiltIn,
}

/// Resolve which config to use.
///
/// Order:
/// 1. `--config <path>` is honoured verbatim. The directory containing the
///    explicit file becomes the project root for scan patterns (tsconfig-style).
/// 2. Otherwise, walk ancestors of `start` looking for `flavor.json`. The
///    nearest match wins; its directory becomes the project root.
/// 3. Otherwise, fall back to the built-in defaults rooted at `start`.
///
/// `start` is the walk-up entry point (typically `--root`, defaulting to the
/// current directory). It is canonicalized to make ancestor traversal robust
/// against `.` / `..` segments.
pub(crate) fn resolve(
    start: PathBuf,
    explicit: Option<PathBuf>,
) -> Result<(GuardConfig, ConfigSource), String> {
    if let Some(path) = explicit {
        let root = config_parent(&path);
        let config = GuardConfig::from_file(root, &path)?;
        return Ok((config, ConfigSource::Explicit(path)));
    }
    let start = canonicalize_start(&start)?;
    if let Some(candidate) = walk_up_for_config(&start) {
        let root = config_parent(&candidate);
        let config = GuardConfig::from_file(root, &candidate)?;
        return Ok((config, ConfigSource::Discovered(candidate)));
    }
    Ok((GuardConfig::core(start), ConfigSource::BuiltIn))
}

fn canonicalize_start(start: &Path) -> Result<PathBuf, String> {
    start
        .canonicalize()
        .map_err(|error| format!("failed to resolve {}: {error}", start.display()))
}

fn walk_up_for_config(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        let candidate = ancestor.join(DEFAULT_CONFIG_FILENAME);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Project root for a config file path: the directory containing it.
///
/// `Path::parent` returns `Some("")` for bare filenames like `flavor.json`,
/// which downstream code reads as the empty path and fails to canonicalize.
/// Treat both `None` and an empty parent as "the current directory".
fn config_parent(config_path: &Path) -> PathBuf {
    match config_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

impl GuardConfig {
    pub(crate) fn core(root: PathBuf) -> Self {
        Self {
            root,
            scan_include: patterns(core_include_patterns()),
            scan_exclude: patterns(core_exclude_patterns()),
            preferences: Vec::new(),
            overrides: Vec::new(),
            allow_empty_scan: false,
        }
    }

    /// Whether the active config opted out of the "0 files matched" failure.
    ///
    /// A workspace-root `flavor.json` that intentionally excludes every
    /// submodule (delegating real checks to per-submodule configs) sets this
    /// so the 0-match warning + exit 1 from PR #6 stays quiet.
    pub(crate) fn allow_empty_scan(&self) -> bool {
        self.allow_empty_scan
    }

    pub(crate) fn from_file(root: PathBuf, config_path: &Path) -> Result<Self, String> {
        let source = fs::read_to_string(config_path)
            .map_err(|error| format!("failed to read config {}: {error}", config_path.display()))?;
        let file: GuardConfigFile = serde_json::from_str(&source).map_err(|error| {
            format!("failed to parse config {}: {error}", config_path.display())
        })?;
        Self::from_config_file(root, file)
    }

    fn from_config_file(root: PathBuf, file: GuardConfigFile) -> Result<Self, String> {
        let scan = file.scan;
        let preferences = preferences::expand(file.preferences)?;
        let mut overrides = Vec::with_capacity(file.overrides.len());
        for (order, item) in file.overrides.into_iter().enumerate() {
            validate_rules(item.kind.unwrap_or(MatchKind::Any), &item.rules)?;
            let raw_patterns = item.matches.into_vec();
            if raw_patterns.is_empty() {
                return Err(format!(
                    "override at index {order} has empty 'match'; use a glob string or a non-empty array"
                ));
            }
            let patterns = raw_patterns
                .iter()
                .map(|pattern| PathPattern::new(pattern))
                .collect();
            overrides.push(RuleMatcher {
                patterns,
                kind: item.kind.unwrap_or(MatchKind::Any),
                priority: item.priority.unwrap_or_default(),
                order,
                rules: item.rules,
            });
        }
        overrides.sort_by_key(|item| (item.priority, item.order));

        Ok(Self {
            root,
            scan_include: required_patterns(scan.include, "scan.include")?,
            scan_exclude: patterns(scan.exclude.unwrap_or_default()),
            preferences,
            overrides,
            allow_empty_scan: file.allow_empty_scan,
        })
    }

    pub(crate) fn is_scanned(&self, relative: &Path) -> bool {
        self.scan_include
            .iter()
            .any(|pattern| pattern.matches(relative))
            && !self.is_excluded(relative)
    }

    pub(crate) fn is_excluded(&self, relative: &Path) -> bool {
        self.scan_exclude
            .iter()
            .any(|pattern| pattern.matches(relative))
    }

    pub(crate) fn rule(
        &self,
        relative: &Path,
        kind: NodeKind,
        rule_id: &'static str,
    ) -> RuleSettings {
        let descriptor = rules::descriptor(rule_id).expect("built-in rule descriptor must exist");
        let mut settings = RuleSettings {
            id: descriptor.id,
            enabled: descriptor.default_enabled,
            severity: descriptor.default_severity,
            payload: descriptor
                .default_payload
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
        };

        for matcher in self.preferences.iter().chain(&self.overrides) {
            if !matcher.matches(relative, kind) {
                continue;
            }
            let Some(override_rule) = matcher.rules.get(rule_id) else {
                continue;
            };
            if let Some(enabled) = override_rule.enabled {
                settings.enabled = enabled;
            }
            if let Some(severity) = override_rule.severity {
                settings.severity = severity;
            }
            if let Some(payload) = &override_rule.payload {
                for (key, value) in payload {
                    settings.payload.insert(key.clone(), value.clone());
                }
            }
        }

        settings
    }
}

#[derive(Debug, Deserialize)]
struct GuardConfigFile {
    scan: ScanConfigFile,
    #[serde(default)]
    preferences: Vec<preferences::PreferenceConfigFile>,
    #[serde(default)]
    overrides: Vec<OverrideConfigFile>,
    #[serde(default, rename = "allowEmptyScan")]
    allow_empty_scan: bool,
}

#[derive(Debug, Deserialize)]
struct ScanConfigFile {
    include: Vec<String>,
    #[serde(default)]
    exclude: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OverrideConfigFile {
    #[serde(rename = "match")]
    matches: MatchPatterns,
    #[serde(default)]
    kind: Option<MatchKind>,
    #[serde(default)]
    priority: Option<i32>,
    rules: BTreeMap<String, RuleOverride>,
}

/// Accepts either a single `match: "<glob>"` or `match: ["<glob>", ...]` to
/// scope one override entry over multiple paths without duplicating the
/// surrounding `kind` / `priority` / `rules` block.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum MatchPatterns {
    One(String),
    Many(Vec<String>),
}

impl MatchPatterns {
    pub(crate) fn into_vec(self) -> Vec<String> {
        match self {
            MatchPatterns::One(value) => vec![value],
            MatchPatterns::Many(values) => values,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MatchKind {
    Any,
    File,
    Dir,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RuleOverride {
    #[serde(default)]
    pub(crate) enabled: Option<bool>,
    #[serde(default)]
    pub(crate) severity: Option<Severity>,
    #[serde(default)]
    pub(crate) reason: Option<String>,
    #[serde(default)]
    pub(crate) payload: Option<BTreeMap<String, Value>>,
}

impl RuleOverride {
    pub(crate) fn enabled_with_payload(payload: BTreeMap<String, Value>) -> Self {
        Self {
            enabled: Some(true),
            severity: None,
            reason: None,
            payload: Some(payload),
        }
    }

    pub(crate) fn disabled(reason: &'static str) -> Self {
        Self {
            enabled: Some(false),
            severity: None,
            reason: Some(reason.to_string()),
            payload: None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuleMatcher {
    pub(crate) patterns: Vec<PathPattern>,
    pub(crate) kind: MatchKind,
    pub(crate) priority: i32,
    pub(crate) order: usize,
    pub(crate) rules: BTreeMap<String, RuleOverride>,
}

impl RuleMatcher {
    fn matches(&self, relative: &Path, kind: NodeKind) -> bool {
        let kind_matches = match (self.kind, kind) {
            (MatchKind::Any, _) => true,
            (MatchKind::File, NodeKind::File) => true,
            (MatchKind::Dir, NodeKind::Dir) => true,
            (MatchKind::File | MatchKind::Dir, _) => false,
        };
        kind_matches
            && self
                .patterns
                .iter()
                .any(|pattern| pattern.matches(relative))
    }
}

fn validate_rules(kind: MatchKind, rules: &BTreeMap<String, RuleOverride>) -> Result<(), String> {
    for (rule_id, rule) in rules {
        let Some(descriptor) = rules::descriptor(rule_id) else {
            return Err(format!(
                "unknown flavor rule id: {rule_id}; known ids: {}",
                rules::known_rule_ids().join(", ")
            ));
        };
        if !rule_target_matches(kind, descriptor.target) {
            return Err(format!(
                "rule {rule_id} targets {:?}, but override kind is {:?}",
                descriptor.target, kind
            ));
        }
        if rule.enabled == Some(false)
            && rule
                .reason
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
        {
            return Err(format!("disabled rule {rule_id} must include a reason"));
        }
    }
    Ok(())
}

fn rule_target_matches(kind: MatchKind, target: RuleTarget) -> bool {
    matches!(
        (kind, target),
        (MatchKind::Any, _)
            | (MatchKind::File, RuleTarget::File)
            | (MatchKind::Dir, RuleTarget::Dir)
    )
}

fn required_patterns(values: Vec<String>, field: &str) -> Result<Vec<PathPattern>, String> {
    if values.is_empty() {
        return Err(format!(
            "config field '{field}' must contain at least one pattern"
        ));
    }
    Ok(patterns(values))
}

fn patterns(values: Vec<String>) -> Vec<PathPattern> {
    values
        .into_iter()
        .map(|pattern| PathPattern::new(&pattern))
        .collect()
}

fn core_include_patterns() -> Vec<String> {
    [
        "apps/*/src/**",
        "apps/*/tests/**",
        "apps/renderer/server/src/**",
        "apps/renderer/vite/src/**",
        "apps/tauri/src-tauri/src/**",
        "apps/tauri/src-tauri/tests/**",
        "crates/*/src/**",
        "crates/*/tests/**",
        "grammars/**",
        "tools/*/src/**",
        "tools/*/tests/**",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn core_exclude_patterns() -> Vec<String> {
    [
        "**/node_modules/**",
        "**/target/**",
        "**/dist/**",
        "**/.vite/**",
        "**/.vite-temp/**",
        ".tmp/**",
        "apps/tauri/src-tauri/gen/**",
        "packages/client/src/gen/**",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

pub(crate) fn source_file_kind(path: &Path) -> Option<SourceKind> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("rs") => Some(SourceKind::Rust),
        Some("g4") => Some(SourceKind::G4),
        Some("svelte") => Some(SourceKind::Svelte),
        Some("ts" | "tsx") => Some(SourceKind::TypeScript),
        Some("vue") => Some(SourceKind::Vue),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum SourceKind {
    G4,
    Rust,
    Svelte,
    TypeScript,
    Vue,
}

#[allow(dead_code)]
fn _plugin_target_seam(_: RuleTarget) {}
