use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use crate::config::{resolve, ConfigSource, DEFAULT_CONFIG_FILENAMES, ENV_CONFIG};

const SAMPLE_CONFIG: &str = r#"{ "scan": { "include": ["**/*.rs"] } }"#;

#[test]
fn resolves_explicit_config_path() {
    let _env = clear_env_config();
    let root = test_root("explicit");
    fs::create_dir_all(&root).unwrap();
    let explicit = root.join("custom.json");
    fs::write(&explicit, SAMPLE_CONFIG).unwrap();

    let (_, source) = resolve(root.clone(), Some(explicit.clone())).unwrap();

    assert_eq!(source, ConfigSource::Explicit(explicit));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn env_config_wins() {
    let root = test_root("env");
    fs::create_dir_all(&root).unwrap();
    let env_config = root.join("selected.toml");
    fs::write(&env_config, "[scan]\ninclude = [\"**/*.rs\"]\n").unwrap();
    fs::write(root.join("flavor.toml"), "[scan]\ninclude = [\"src/**\"]\n").unwrap();

    with_env_config(&env_config, || {
        let (_, source) = resolve(root.clone(), None).unwrap();
        assert_eq!(source, ConfigSource::Env(env_config.clone()));
    });

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn discovers_default_priority() {
    let _env = clear_env_config();
    let root = test_root("discovery-priority");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("flavor.json"), SAMPLE_CONFIG).unwrap();
    fs::write(
        root.join("flavor.yaml"),
        "scan:\n  include:\n    - src/**\n",
    )
    .unwrap();
    let selected = root.join("flavor.toml");
    fs::write(&selected, "[scan]\ninclude = [\"**/*.rs\"]\n").unwrap();

    let (_, source) = resolve(root.clone(), None).unwrap();

    assert_eq!(DEFAULT_CONFIG_FILENAMES[0], "flavor.toml");
    assert_eq!(source, ConfigSource::Discovered(canonical(&selected)));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn walk_up_keeps_root() {
    let _env = clear_env_config();
    let root = test_root("walk-up-ancestor");
    let nested = root.join("a/b/c");
    fs::create_dir_all(&nested).unwrap();
    let config = root.join("flavor.json");
    fs::write(&config, SAMPLE_CONFIG).unwrap();

    let (guard, source) = resolve(nested.clone(), None).unwrap();
    let canonical_root = canonical(&root);

    assert_eq!(source, ConfigSource::Discovered(canonical(&config)));
    assert_eq!(guard.root, canonical(&nested));
    assert_eq!(guard.config_root(), canonical_root.as_path());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn nearest_directory_wins() {
    let _env = clear_env_config();
    let root = test_root("walk-up-nearest");
    let middle = root.join("inner");
    let leaf = middle.join("leaf");
    fs::create_dir_all(&leaf).unwrap();
    fs::write(
        root.join("flavor.toml"),
        "[scan]\ninclude = [\"**/*.rs\"]\n",
    )
    .unwrap();
    let nearer = middle.join("flavor.json");
    fs::write(&nearer, SAMPLE_CONFIG).unwrap();

    let (_, source) = resolve(leaf, None).unwrap();

    assert_eq!(source, ConfigSource::Discovered(canonical(&nearer)));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn loads_allow_empty() {
    let _env = clear_env_config();
    let root = test_root("allow-empty-scan");
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join("flavor.json");
    fs::write(
        &config_path,
        r#"{ "scan": { "include": ["**/*.rs"] }, "allow_empty_scan": true }"#,
    )
    .unwrap();

    let (guard, _) = resolve(root.clone(), None).unwrap();

    assert!(guard.allow_empty_scan());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn defaults_without_scan() {
    let _env = clear_env_config();
    let root = test_root("default-scan");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("flavor.toml"), "overrides = []\n").unwrap();

    let (guard, _) = resolve(root.clone(), None).unwrap();

    assert!(guard.is_scanned("src/main.rs".as_ref()));
    assert!(guard.is_scanned("crates/demo/tests/case.rs".as_ref()));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn built_in_fallback() {
    let _env = clear_env_config();
    let root = test_root("builtin");
    fs::create_dir_all(&root).unwrap();

    let (_, source) = resolve(root.clone(), None).unwrap();

    assert_eq!(source, ConfigSource::BuiltIn);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn propagates_explicit_path_errors() {
    let _env = clear_env_config();
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
fn explicit_overrides_env() {
    let root = test_root("explicit-wins");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("flavor.toml"), "[scan]\ninclude = [\"src/**\"]\n").unwrap();
    let env_config = root.join("env.json");
    fs::write(&env_config, SAMPLE_CONFIG).unwrap();
    let explicit = root.join("other.yaml");
    fs::write(&explicit, "scan:\n  include:\n    - '**/*.rs'\n").unwrap();

    with_env_config(&env_config, || {
        let (_, source) = resolve(root.clone(), Some(explicit.clone())).unwrap();
        assert_eq!(source, ConfigSource::Explicit(explicit.clone()));
    });

    let _ = fs::remove_dir_all(&root);
}

fn with_env_config(path: &Path, run: impl FnOnce()) {
    let _env = set_env_config(Some(path));
    run();
}

fn clear_env_config() -> EnvGuard {
    set_env_config(None)
}

fn set_env_config(path: Option<&Path>) -> EnvGuard {
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let previous = env::var_os(ENV_CONFIG);
    if let Some(path) = path {
        env::set_var(ENV_CONFIG, path);
    } else {
        env::remove_var(ENV_CONFIG);
    }
    EnvGuard {
        previous,
        _lock: lock,
    }
}

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct EnvGuard {
    previous: Option<std::ffi::OsString>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(previous) = &self.previous {
            env::set_var(ENV_CONFIG, previous);
        } else {
            env::remove_var(ENV_CONFIG);
        }
    }
}

fn test_root(name: &str) -> PathBuf {
    let root = env::temp_dir().join(format!(
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

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().expect("test path canonicalizes")
}
