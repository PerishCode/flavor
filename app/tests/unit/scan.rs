use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::GuardConfig,
    rules::{FS_TOO_MANY_CHILDREN, RUST_TESTS_IN_SOURCE, SOURCE_TOO_DEEP, SOURCE_TOO_LONG},
    scan::run_checks,
};

#[test]
fn scans_structure_limits() {
    let root = test_root("structure");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    for index in 0..11 {
        fs::write(source_dir.join(format!("file_{index}.rs")), "fn ok() {}\n").unwrap();
    }
    fs::write(source_dir.join("large.rs"), "fn ok() {}\n".repeat(501)).unwrap();

    let config = test_config(&root, "tools/*/src/**");
    let issues = run_checks(&config).unwrap();

    assert!(issues.iter().any(|issue| issue.rule == SOURCE_TOO_LONG));
    assert!(issues
        .iter()
        .any(|issue| issue.rule == FS_TOO_MANY_CHILDREN));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn warns_deep_source_dirs() {
    let root = test_root("depth");
    fs::create_dir_all(root.join("tools/demo/src/a/b/c/d/e")).unwrap();
    fs::write(
        root.join("tools/demo/src/a/b/c/d/e/file.rs"),
        "fn ok() {}\n",
    )
    .unwrap();

    let config = test_config(&root, "tools/*/src/**");
    let issues = run_checks(&config).unwrap();

    assert!(issues.iter().any(|issue| issue.rule == SOURCE_TOO_DEEP));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn counts_depth_after_src() {
    let root = test_root("src-depth");
    fs::create_dir_all(root.join("apps/renderer/vite/src/agents")).unwrap();
    fs::write(
        root.join("apps/renderer/vite/src/agents/model.ts"),
        "const ok = 1;\n",
    )
    .unwrap();

    let config = test_config(&root, "apps/renderer/vite/src/**");
    let issues = run_checks(&config).unwrap();

    assert!(!issues.iter().any(|issue| issue.rule == SOURCE_TOO_DEEP));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn denies_src_rust_tests() {
    let root = test_root("rust-tests");
    fs::create_dir_all(root.join("tools/demo/src")).unwrap();
    fs::write(
        root.join("tools/demo/src/lib.rs"),
        "#[cfg(test)] mod tests { #[test] fn sample() {} }\n",
    )
    .unwrap();

    let config = test_config(&root, "tools/*/src/**");
    let issues = run_checks(&config).unwrap();

    assert!(issues
        .iter()
        .any(|issue| issue.rule == RUST_TESTS_IN_SOURCE));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn override_disables_rule() {
    let root = test_root("disable-rule");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("large.rs"), "fn ok() {}\n".repeat(501)).unwrap();

    let config = config_from(
        &root,
        r#"{
            "scan": { "include": ["tools/*/src/**"] },
            "overrides": [
                {
                    "match": "tools/demo/src/**",
                    "kind": "file",
                    "rules": {
                        "core/source/too-long": {
                            "enabled": false,
                            "reason": "fixture intentionally exercises line-count pressure"
                        }
                    }
                }
            ]
        }"#,
    );
    let issues = run_checks(&config).unwrap();

    assert!(!issues.iter().any(|issue| issue.rule == SOURCE_TOO_LONG));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn override_priority_wins() {
    let root = test_root("override-priority");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("large.rs"), "fn ok() {}\n".repeat(501)).unwrap();

    let config = config_from(
        &root,
        r#"{
            "scan": { "include": ["tools/*/src/**"] },
            "overrides": [
                {
                    "match": "tools/**",
                    "kind": "file",
                    "priority": 0,
                    "rules": {
                        "core/source/too-long": {
                            "enabled": false,
                            "reason": "lower-priority workspace sweep waiver"
                        }
                    }
                },
                {
                    "match": "tools/demo/src/*.rs",
                    "kind": "file",
                    "priority": 10,
                    "rules": {
                        "core/source/too-long": {
                            "enabled": true,
                            "payload": { "max_lines": 10 }
                        }
                    }
                }
            ]
        }"#,
    );
    let issues = run_checks(&config).unwrap();

    assert!(issues.iter().any(|issue| issue.rule == SOURCE_TOO_LONG));

    let _ = fs::remove_dir_all(root);
}

fn test_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("flavor-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    root
}

fn test_config(root: &Path, include: &str) -> GuardConfig {
    config_from(
        root,
        &format!(r#"{{ "scan": {{ "include": ["{include}"] }} }}"#),
    )
}

fn config_from(root: &Path, source: &str) -> GuardConfig {
    let path = root.join("flavor.json");
    fs::write(&path, source).unwrap();
    GuardConfig::from_file(root.to_path_buf(), &path).unwrap()
}
