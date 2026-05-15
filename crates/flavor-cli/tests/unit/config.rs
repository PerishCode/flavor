use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::config::{resolve, ConfigSource, DEFAULT_CONFIG_FILENAME};

const SAMPLE_CONFIG: &str = r#"{ "scan": { "include": ["**/*.rs"] } }"#;

#[test]
fn resolves_explicit_config_path() {
    let root = test_root("explicit");
    fs::create_dir_all(&root).unwrap();
    let explicit = root.join("custom.json");
    fs::write(&explicit, SAMPLE_CONFIG).unwrap();

    let (_, source) = resolve(root.clone(), Some(explicit.clone())).unwrap();

    assert_eq!(source, ConfigSource::Explicit(explicit));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn discovers_flavor_json_root() {
    let root = test_root("discovery");
    fs::create_dir_all(&root).unwrap();
    let discovered = root.join(DEFAULT_CONFIG_FILENAME);
    fs::write(&discovered, SAMPLE_CONFIG).unwrap();

    let (_, source) = resolve(root.clone(), None).unwrap();

    // resolve() canonicalizes the start path so ancestor traversal handles
    // `.` / `..` segments; the discovered path therefore comes back canonical.
    // On macOS that means /var/folders/... -> /private/var/folders/...
    assert_eq!(source, ConfigSource::Discovered(canonical(&discovered)));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn walk_up_finds_ancestor() {
    let root = test_root("walk-up-ancestor");
    let nested = root.join("a/b/c");
    fs::create_dir_all(&nested).unwrap();
    let config = root.join(DEFAULT_CONFIG_FILENAME);
    fs::write(&config, SAMPLE_CONFIG).unwrap();

    let (guard, source) = resolve(nested.clone(), None).unwrap();

    assert_eq!(source, ConfigSource::Discovered(canonical(&config)));
    // Project root is the discovered config's directory, not the start path.
    assert_eq!(guard.root, canonical(&root));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn nearest_config_wins() {
    let root = test_root("walk-up-nearest");
    let middle = root.join("inner");
    let leaf = middle.join("leaf");
    fs::create_dir_all(&leaf).unwrap();
    fs::write(root.join(DEFAULT_CONFIG_FILENAME), SAMPLE_CONFIG).unwrap();
    let nearer = middle.join(DEFAULT_CONFIG_FILENAME);
    fs::write(&nearer, SAMPLE_CONFIG).unwrap();

    let (guard, source) = resolve(leaf, None).unwrap();

    // Walk-up from leaf hits middle/flavor.json before root/flavor.json.
    assert_eq!(source, ConfigSource::Discovered(canonical(&nearer)));
    assert_eq!(guard.root, canonical(&middle));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn loads_allow_empty() {
    let root = test_root("allow-empty-scan");
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join(DEFAULT_CONFIG_FILENAME);
    fs::write(
        &config_path,
        r#"{ "scan": { "include": ["**/*.rs"] }, "allowEmptyScan": true }"#,
    )
    .unwrap();

    let (guard, _) = resolve(root.clone(), None).unwrap();

    assert!(guard.allow_empty_scan());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn allow_empty_default_off() {
    let root = test_root("allow-empty-scan-default");
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join(DEFAULT_CONFIG_FILENAME);
    fs::write(&config_path, SAMPLE_CONFIG).unwrap();

    let (guard, _) = resolve(root.clone(), None).unwrap();

    assert!(!guard.allow_empty_scan());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn rejects_unknown_preference() {
    let root = test_root("unknown-preference");
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join(DEFAULT_CONFIG_FILENAME);
    fs::write(
        &config_path,
        r#"{
            "scan": { "include": ["**/*.rs"] },
            "preferences": [
                { "name": "frontend/mystery", "match": "src" }
            ]
        }"#,
    )
    .unwrap();

    let error = resolve(root.clone(), Some(config_path)).unwrap_err();

    assert!(
        error.contains("unknown flavor preference set"),
        "expected unknown-preference error, got: {error}"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn renderer_preference_requires_sources() {
    let root = test_root("preference-sources");
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join(DEFAULT_CONFIG_FILENAME);
    fs::write(
        &config_path,
        r#"{
            "scan": { "include": ["src/**"] },
            "preferences": [
                { "name": "frontend/renderer-boundary", "match": "src" }
            ]
        }"#,
    )
    .unwrap();

    let error = resolve(root.clone(), Some(config_path)).unwrap_err();

    assert!(
        error.contains("requires non-empty primitiveSources"),
        "expected primitive source error, got: {error}"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn built_in_fallback() {
    let root = test_root("builtin");
    fs::create_dir_all(&root).unwrap();

    let (_, source) = resolve(root.clone(), None).unwrap();

    assert_eq!(source, ConfigSource::BuiltIn);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn propagates_explicit_path_errors() {
    let root = test_root("explicit-missing");
    fs::create_dir_all(&root).unwrap();
    let missing = root.join("missing.json");

    let error = resolve(root.clone(), Some(missing)).unwrap_err();

    assert!(
        error.contains("failed to read config"),
        "expected read-failure message, got: {error}"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn explicit_overrides_discovery() {
    let root = test_root("explicit-wins");
    fs::create_dir_all(&root).unwrap();
    // A flavor.json sitting at root should be ignored when --config points
    // elsewhere; otherwise users can't tell which config was loaded.
    fs::write(root.join(DEFAULT_CONFIG_FILENAME), SAMPLE_CONFIG).unwrap();
    let explicit = root.join("other.json");
    fs::write(&explicit, SAMPLE_CONFIG).unwrap();

    let (_, source) = resolve(root.clone(), Some(explicit.clone())).unwrap();

    assert_eq!(source, ConfigSource::Explicit(explicit));

    let _ = fs::remove_dir_all(&root);
}

fn test_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "flavor-config-{name}-{}-{}",
        std::process::id(),
        next_seq()
    ));
    let _ = fs::remove_dir_all(&root);
    root
}

fn next_seq() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    SEQ.fetch_add(1, Ordering::Relaxed)
}

#[allow(dead_code)]
fn root_to_string(path: &Path) -> String {
    path.display().to_string()
}

/// Returns the canonical (symlink-resolved, absolute) form of `path`.
///
/// Resolve always canonicalizes the walk-up start, so any `Discovered` path
/// the test compares against has to canonicalize too — macOS resolves
/// `/var/folders/...` to `/private/var/folders/...`, and Linux variants can
/// differ similarly.
fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().expect("test path canonicalizes")
}
