use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

use crate::{
    model::Severity,
    path_match::PathPattern,
    rules::{self, RuleTarget},
};

#[derive(Debug, Clone)]
pub(crate) struct GuardConfig {
    config_root: PathBuf,
    pub(crate) root: PathBuf,
    scan_include: Vec<PathPattern>,
    scan_exclude: Vec<PathPattern>,
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
    pub(crate) fn usize(&self, key: &str) -> Option<usize> {
        self.payload
            .get(key)
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
    }
}

pub(crate) const DEFAULT_CONFIG_FILENAMES: &[&str] = &["flavor.toml", "flavor.yaml", "flavor.json"];
pub(crate) const ENV_CONFIG: &str = "FLAVOR_CONFIG";

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ConfigSource {
    Explicit(PathBuf),
    Env(PathBuf),
    Discovered(PathBuf),
    BuiltIn,
}

pub(crate) fn resolve(
    start: PathBuf,
    explicit: Option<PathBuf>,
) -> Result<(GuardConfig, ConfigSource), String> {
    let scan_root = canonicalize_start(&start)?;

    if let Some(path) = explicit {
        let config_root = config_parent(&path)?;
        let config = GuardConfig::from_file(config_root, scan_root, &path)?;
        return Ok((config, ConfigSource::Explicit(path)));
    }

    if let Some(path) = env::var_os(ENV_CONFIG).map(PathBuf::from) {
        let config_root = config_parent(&path)?;
        let config = GuardConfig::from_file(config_root, scan_root, &path)?;
        return Ok((config, ConfigSource::Env(path)));
    }

    if let Some(candidate) = walk_up_for_config(&scan_root) {
        let config_root = config_parent(&candidate)?;
        let config = GuardConfig::from_file(config_root, scan_root, &candidate)?;
        return Ok((config, ConfigSource::Discovered(candidate)));
    }

    Ok((GuardConfig::core(scan_root), ConfigSource::BuiltIn))
}

fn canonicalize_start(start: &Path) -> Result<PathBuf, String> {
    start
        .canonicalize()
        .map_err(|error| format!("failed to resolve {}: {error}", start.display()))
}

fn walk_up_for_config(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        for filename in DEFAULT_CONFIG_FILENAMES {
            let candidate = ancestor.join(filename);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn config_parent(config_path: &Path) -> Result<PathBuf, String> {
    let parent = match config_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    };
    parent.canonicalize().map_err(|error| {
        format!(
            "failed to resolve config root {}: {error}",
            parent.display()
        )
    })
}

impl GuardConfig {
    pub(crate) fn core(root: PathBuf) -> Self {
        Self {
            config_root: root.clone(),
            root,
            scan_include: patterns(core_include_patterns()).expect("built-in includes are valid"),
            scan_exclude: patterns(core_exclude_patterns()).expect("built-in excludes are valid"),
            overrides: Vec::new(),
            allow_empty_scan: false,
        }
    }

    pub(crate) fn allow_empty_scan(&self) -> bool {
        self.allow_empty_scan
    }

    pub(crate) fn config_root(&self) -> &Path {
        &self.config_root
    }

    pub(crate) fn from_file(
        config_root: PathBuf,
        scan_root: PathBuf,
        config_path: &Path,
    ) -> Result<Self, String> {
        if !scan_root.starts_with(&config_root) {
            return Err(format!(
                "scan root {} must be inside config root {}",
                scan_root.display(),
                config_root.display()
            ));
        }

        let source = fs::read_to_string(config_path)
            .map_err(|error| format!("failed to read config {}: {error}", config_path.display()))?;
        let file: GuardConfigFile = parse_config(config_path, &source)?;
        Self::from_config_file(config_root, scan_root, file)
    }

    fn from_config_file(
        config_root: PathBuf,
        scan_root: PathBuf,
        file: GuardConfigFile,
    ) -> Result<Self, String> {
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
                .collect::<Result<Vec<_>, _>>()?;
            overrides.push(RuleMatcher {
                patterns,
                kind: item.kind.unwrap_or(MatchKind::Any),
                priority: item.priority.unwrap_or_default(),
                order,
                rules: item.rules,
            });
        }
        overrides.sort_by_key(|item| (item.priority, item.order));

        let scan = file.scan.unwrap_or_default();
        let scan_include = match scan.include {
            Some(values) => required_patterns(values, "scan.include")?,
            None => patterns(core_include_patterns())?,
        };
        let scan_exclude = match scan.exclude {
            Some(values) => patterns(values)?,
            None => patterns(core_exclude_patterns())?,
        };

        Ok(Self {
            config_root,
            root: scan_root,
            scan_include,
            scan_exclude,
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
    #[serde(default)]
    scan: Option<ScanConfigFile>,
    #[serde(default)]
    overrides: Vec<OverrideConfigFile>,
    #[serde(default)]
    allow_empty_scan: bool,
}

#[derive(Debug, Default, Deserialize)]
struct ScanConfigFile {
    #[serde(default)]
    include: Option<Vec<String>>,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum MatchPatterns {
    One(String),
    Many(Vec<String>),
}

impl MatchPatterns {
    fn into_vec(self) -> Vec<String> {
        match self {
            MatchPatterns::One(value) => vec![value],
            MatchPatterns::Many(values) => values,
        }
    }
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
    patterns: Vec<PathPattern>,
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
        kind_matches
            && self
                .patterns
                .iter()
                .any(|pattern| pattern.matches(relative))
    }
}

fn parse_config<T: DeserializeOwned>(config_path: &Path, source: &str) -> Result<T, String> {
    match config_path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("json") => serde_json::from_str(source).map_err(|error| {
            format!(
                "failed to parse JSON config {}: {error}",
                config_path.display()
            )
        }),
        Some("yaml") | Some("yml") => serde_yaml::from_str(source).map_err(|error| {
            format!(
                "failed to parse YAML config {}: {error}",
                config_path.display()
            )
        }),
        Some("toml") => toml::from_str(source).map_err(|error| {
            format!(
                "failed to parse TOML config {}: {error}",
                config_path.display()
            )
        }),
        Some(extension) => Err(format!(
            "unsupported config extension '.{extension}' for {}",
            config_path.display()
        )),
        None => Err(format!(
            "config path {} has no supported extension",
            config_path.display()
        )),
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
    patterns(values)
}

fn patterns(values: Vec<String>) -> Result<Vec<PathPattern>, String> {
    values
        .into_iter()
        .map(|pattern| PathPattern::new(&pattern))
        .collect()
}

fn core_include_patterns() -> Vec<String> {
    [
        "src/**",
        "tests/**",
        "test/**",
        "lib/**",
        "app/**",
        "pages/**",
        "components/**",
        "apps/*/src/**",
        "apps/*/tests/**",
        "apps/renderer/server/src/**",
        "apps/renderer/vite/src/**",
        "apps/tauri/src-tauri/src/**",
        "apps/tauri/src-tauri/tests/**",
        "crates/*/src/**",
        "crates/*/tests/**",
        "packages/*/src/**",
        "packages/*/tests/**",
        "tools/*/src/**",
        "tools/*/tests/**",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn core_exclude_patterns() -> Vec<String> {
    [
        ".git/**",
        ".task/**",
        ".tmp/**",
        "**/.next/**",
        "**/coverage/**",
        "**/node_modules/**",
        "**/target/**",
        "**/dist/**",
        "**/.vite/**",
        "**/.vite-temp/**",
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
        Some("svelte") => Some(SourceKind::Svelte),
        Some("ts" | "tsx" | "vue") => Some(SourceKind::TypeScript),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum SourceKind {
    Rust,
    Svelte,
    TypeScript,
}
