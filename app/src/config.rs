use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::Value;

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
    overrides: Vec<RuleMatcher>,
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
    pub(crate) fn usize(&self, key: &str) -> Option<usize> {
        self.payload
            .get(key)
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
    }
}

impl GuardConfig {
    pub(crate) fn core(root: PathBuf) -> Self {
        Self {
            root,
            scan_include: patterns(core_include_patterns()),
            scan_exclude: patterns(core_exclude_patterns()),
            overrides: Vec::new(),
        }
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
        let mut overrides = Vec::with_capacity(file.overrides.len());
        for (order, item) in file.overrides.into_iter().enumerate() {
            validate_rules(item.kind.unwrap_or(MatchKind::Any), &item.rules)?;
            overrides.push(RuleMatcher {
                pattern: PathPattern::new(&item.matches),
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
            overrides,
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
            enabled: true,
            severity: descriptor.default_severity,
            payload: descriptor
                .default_payload
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
        };

        for matcher in &self.overrides {
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
    overrides: Vec<OverrideConfigFile>,
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
    matches: String,
    #[serde(default)]
    kind: Option<MatchKind>,
    #[serde(default)]
    priority: Option<i32>,
    rules: BTreeMap<String, RuleOverride>,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum MatchKind {
    Any,
    File,
    Dir,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RuleOverride {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    severity: Option<Severity>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    payload: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone)]
struct RuleMatcher {
    pattern: PathPattern,
    kind: MatchKind,
    priority: i32,
    order: usize,
    rules: BTreeMap<String, RuleOverride>,
}

impl RuleMatcher {
    fn matches(&self, relative: &Path, kind: NodeKind) -> bool {
        let kind_matches = match (self.kind, kind) {
            (MatchKind::Any, _) => true,
            (MatchKind::File, NodeKind::File) => true,
            (MatchKind::Dir, NodeKind::Dir) => true,
            (MatchKind::File | MatchKind::Dir, _) => false,
        };
        kind_matches && self.pattern.matches(relative)
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
        Some("ts" | "tsx" | "vue") => Some(SourceKind::TypeScript),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum SourceKind {
    Rust,
    TypeScript,
}

#[allow(dead_code)]
fn _plugin_target_seam(_: RuleTarget) {}
