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

    assert_eq!(source, ConfigSource::Discovered(discovered));

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
