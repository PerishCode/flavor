use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::GuardConfig,
    model::Report,
    rules::{
        FS_TOO_MANY_CHILDREN, FUNCTION_TOO_LONG, NAMING_TOO_MANY_WORDS, RUST_TESTS_IN_SOURCE,
        SOURCE_TOO_DEEP, SOURCE_TOO_LONG, SVELTE_COMPONENT_TOO_LONG, SVELTE_PARSE_ERROR,
        SVELTE_SCRIPT_TOO_LONG, SVELTE_STYLE_TOO_LONG, SVELTE_TEMPLATE_TOO_COMPLEX,
    },
    scan::run_scan,
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
    let issues = issues(&config);

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
    let issues = issues(&config);

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
    let issues = issues(&config);

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
    let issues = issues(&config);

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
    let issues = issues(&config);

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
    let issues = issues(&config);

    assert!(issues.iter().any(|issue| issue.rule == SOURCE_TOO_LONG));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn override_match_array() {
    let root = test_root("match-array");
    fs::create_dir_all(root.join("tools/demo/src")).unwrap();
    fs::create_dir_all(root.join("tools/other/src")).unwrap();
    fs::write(
        root.join("tools/demo/src/large.rs"),
        "fn ok() {}\n".repeat(501),
    )
    .unwrap();
    fs::write(
        root.join("tools/other/src/large.rs"),
        "fn ok() {}\n".repeat(501),
    )
    .unwrap();

    let config = config_from(
        &root,
        r#"{
            "scan": { "include": ["tools/*/src/**"] },
            "overrides": [
                {
                    "match": ["tools/demo/src/**", "tools/other/src/**"],
                    "kind": "file",
                    "rules": {
                        "core/source/too-long": {
                            "enabled": false,
                            "reason": "fixtures intentionally exercise line-count pressure"
                        }
                    }
                }
            ]
        }"#,
    );
    let issues = issues(&config);

    assert!(
        !issues.iter().any(|issue| issue.rule == SOURCE_TOO_LONG),
        "array-form match should silence the rule across both paths; got: {issues:?}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn empty_match_rejected() {
    let root = test_root("match-empty");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("flavor.json");
    fs::write(
        &path,
        r#"{
            "scan": { "include": ["**/*.rs"] },
            "overrides": [
                {
                    "match": [],
                    "kind": "file",
                    "rules": {}
                }
            ]
        }"#,
    )
    .unwrap();

    let error = GuardConfig::from_file(root.clone(), &path).unwrap_err();
    assert!(
        error.contains("empty 'match'"),
        "expected empty-match error, got: {error}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn reports_scan_coverage() {
    let root = test_root("scan-coverage");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("main.rs"), "fn ok() {}\n").unwrap();
    fs::write(source_dir.join("notes.md"), "# notes\n").unwrap();
    fs::write(
        source_dir.join("client.ts"),
        "/* Generated by fixture. Do not edit manually. */\nconst generatedNameOverLimit = 1;\n",
    )
    .unwrap();
    fs::create_dir_all(source_dir.join("ignored")).unwrap();
    fs::write(source_dir.join("ignored/file.rs"), "fn ignored() {}\n").unwrap();

    let config = config_from(
        &root,
        r#"{
            "scan": {
                "include": ["tools/*/src/**"],
                "exclude": ["**/ignored/**"]
            }
        }"#,
    );
    let result = run_scan(&config).unwrap();

    assert_eq!(result.stats.matched_files, 3);
    assert_eq!(result.stats.scanned_files, 1);
    assert_eq!(result.stats.generated_files, 1);
    assert_eq!(result.stats.unsupported_files, 1);
    assert!(result.stats.excluded_entries > 0);
    assert!(!result
        .issues
        .iter()
        .any(|issue| issue.rule == NAMING_TOO_MANY_WORDS));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn mixed_language_parity() {
    let root = test_root("mixed-parity");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        source_dir.join("lib.rs"),
        "#[cfg(test)] mod tests { #[test] fn guard_sample_over_limit_name() {} }\n",
    )
    .unwrap();
    fs::write(
        source_dir.join("client.ts"),
        "function rendererOperationEventHandlerName(inputValue: string) { return inputValue; }\n",
    )
    .unwrap();
    fs::write(
        source_dir.join("worker.py"),
        "def python_operation_event_handler_name(input_value):\n    return input_value\n",
    )
    .unwrap();
    fs::write(
        source_dir.join("App.vue"),
        "<script setup lang=\"ts\">\nconst controllerRuntimeResultValueText = 1;\n</script>",
    )
    .unwrap();
    fs::write(
        source_dir.join("Panel.svelte"),
        "<script lang=\"ts\">\nconst panelRendererOperationEventHandlerName = 1;\n</script>",
    )
    .unwrap();

    let config = test_config(&root, "tools/*/src/**");
    let result = run_scan(&config).unwrap();
    let report = Report::with_scan(config.root.clone(), result.stats, result.issues.clone());

    assert_eq!(result.stats.scanned_files, 5);
    assert_eq!(report.exit_code(false), 1);
    assert!(result
        .issues
        .iter()
        .any(|issue| issue.rule == RUST_TESTS_IN_SOURCE && issue.path.ends_with("lib.rs")));
    for path in ["client.ts", "worker.py", "App.vue", "Panel.svelte"] {
        assert!(result
            .issues
            .iter()
            .any(|issue| { issue.rule == NAMING_TOO_MANY_WORDS && issue.path.ends_with(path) }));
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scans_python_function_shape() {
    let root = test_root("python-shape");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        source_dir.join("worker.py"),
        "def work(input_value):\n    if input_value:\n        return input_value\n    return None\n",
    )
    .unwrap();

    let config = config_from(
        &root,
        r#"{
            "scan": { "include": ["tools/*/src/**"] },
            "overrides": [
                {
                    "match": "tools/demo/src/*.py",
                    "kind": "file",
                    "rules": {
                        "core/function/too-long": { "payload": { "max_lines": 2 } },
                        "core/dispatch/branch-too-long": { "payload": { "max_branch_lines": 1 } }
                    }
                }
            ]
        }"#,
    );
    let issues = issues(&config);

    assert!(issues.iter().any(|issue| issue.rule == FUNCTION_TOO_LONG));
    assert!(issues
        .iter()
        .any(|issue| issue.rule == crate::rules::DISPATCH_BRANCH_TOO_LONG));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scans_svelte_scripts() {
    let root = test_root("svelte");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        source_dir.join("Panel.svelte"),
        r#"<script lang="ts">
  const rendererOperationEventHandlerName = 1;
</script>

<section>{rendererOperationEventHandlerName}</section>
"#,
    )
    .unwrap();

    let config = test_config(&root, "tools/*/src/**");
    let issues = issues(&config);

    assert!(issues
        .iter()
        .any(|issue| issue.rule == NAMING_TOO_MANY_WORDS
            && issue.message.contains("rendererOperationEventHandlerName")));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn reports_svelte_descriptor_errors() {
    let root = test_root("svelte-error");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        source_dir.join("Panel.svelte"),
        "<style>.panel { color: red; }",
    )
    .unwrap();

    let config = test_config(&root, "tools/*/src/**");
    let issues = issues(&config);

    assert!(issues.iter().any(|issue| {
        issue.rule == SVELTE_PARSE_ERROR && issue.message.contains("missing closing </style>")
    }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn warns_svelte_shape_rules() {
    let root = test_root("svelte-shape");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        source_dir.join("Panel.svelte"),
        r#"<script>
  let active = true;
  let label = "ready";
</script>

{#if active}
  <p>{label}</p>
{/if}

<style>
  p { color: red; }
  p strong { font-weight: 700; }
</style>
"#,
    )
    .unwrap();

    let config = config_from(
        &root,
        r#"{
            "scan": { "include": ["tools/*/src/**"] },
            "overrides": [
                {
                    "match": "tools/demo/src/*.svelte",
                    "kind": "file",
                    "rules": {
                        "svelte/component/too-long": { "payload": { "max_lines": 4 } },
                        "svelte/script/too-long": { "payload": { "max_lines": 1 } },
                        "svelte/style/too-long": { "payload": { "max_lines": 1 } },
                        "svelte/template/too-complex": { "payload": { "max_blocks": 0 } }
                    }
                }
            ]
        }"#,
    );
    let issues = issues(&config);

    assert!(issues
        .iter()
        .any(|issue| issue.rule == SVELTE_COMPONENT_TOO_LONG));
    assert!(issues
        .iter()
        .any(|issue| issue.rule == SVELTE_SCRIPT_TOO_LONG));
    assert!(issues
        .iter()
        .any(|issue| issue.rule == SVELTE_STYLE_TOO_LONG));
    assert!(issues
        .iter()
        .any(|issue| issue.rule == SVELTE_TEMPLATE_TOO_COMPLEX));

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

fn issues(config: &GuardConfig) -> Vec<crate::model::Issue> {
    run_scan(config).unwrap().issues
}
