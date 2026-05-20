use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::GuardConfig,
    rules::{
        ERROR_FAILURE_SURFACE_AGGREGATE, ERROR_FAILURE_SURFACE_MATURITY,
        SHAPE_REPEATED_TOKEN_PATTERN,
    },
    scan::run_scan,
};

#[test]
fn helper_keys_are_opaque() {
    let key = crate::plugins::helper::line_key(
        "rust",
        "shape.repeated_token_pattern",
        "src/lib.rs",
        Some(12),
    );
    assert_eq!(key, "rust:shape.repeated_token_pattern:src/lib.rs:12");

    let mut aggregation = crate::plugins::helper::IssueAggregation::default();
    assert!(aggregation.accepts(key.as_str()));
    assert!(!aggregation.accepts(key.as_str()));
    assert!(aggregation.accepts("rust:shape.repeated_token_pattern:src/lib.rs:13"));
}

#[test]
fn warns_repeated_shapes() {
    let root = test_root("shape-repeat");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("lib.rs"), repeated_handlers(10)).unwrap();

    let config = config_from(
        &root,
        r#"{
            "scan": { "include": ["tools/*/src/**"] }
        }"#,
    );
    let result = run_scan(&config).unwrap();

    assert!(result.issues.iter().any(|issue| {
        issue.rule == SHAPE_REPEATED_TOKEN_PATTERN
            && issue.message.contains("appears 10 times")
            && issue.message.contains("across")
    }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn lower_thresholds_report() {
    let root = test_root("shape-repeat-low");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("lib.rs"), repeated_handlers(2)).unwrap();

    let config = config_from(
        &root,
        r#"{
            "scan": { "include": ["tools/*/src/**"] },
            "overrides": [
                {
                    "match": "tools/*/src/**",
                    "kind": "file",
                    "rules": {
                        "shape/repeated-token-pattern": {
                            "payload": {
                                "min_occurrences": 2,
                                "min_total_lines": 1,
                                "min_lines": 1,
                                "max_lines": 80,
                                "min_tokens": 1,
                                "max_tokens": 240,
                                "min_nodes": 1,
                                "token_bucket_size": 1,
                                "max_reports": 16
                            }
                        }
                    }
                }
            ]
        }"#,
    );
    let result = run_scan(&config).unwrap();

    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.rule == SHAPE_REPEATED_TOKEN_PATTERN),
        "expected lower threshold shape rule to report, got {:?}",
        result.issues
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn warns_failure_surface() {
    let root = test_root("failure-surface");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        source_dir.join("errors.ts"),
        r#"function one() { throw new Error("one"); }
function two() { throw new Error("two"); }
function three() { throw new Error("three"); }
function four() { throw new Error("four"); }
function five() { throw new Error("five"); }
"#,
    )
    .unwrap();

    let config = test_config(&root, "tools/*/src/**");
    let result = run_scan(&config).unwrap();

    let issue = result
        .issues
        .iter()
        .find(|issue| issue.rule == ERROR_FAILURE_SURFACE_MATURITY)
        .expect("failure surface warning");
    assert!(
        issue.message.contains("raw failure construction appears 5"),
        "unexpected message: {}",
        issue.message
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn structured_surface_ratio() {
    let root = test_root("failure-surface-structured");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        source_dir.join("errors.ts"),
        r#"function load(value: string) {
  ensure(value);
  ensure(value);
  ensure(value);
  ensure(value);
  ensure(value);
  throw new Error("one");
  throw new Error("two");
  throw new Error("three");
  throw new Error("four");
  throw new Error("five");
}
"#,
    )
    .unwrap();

    let config = config_from(
        &root,
        r#"{
            "scan": { "include": ["tools/*/src/**"] },
            "overrides": [
                {
                    "match": "tools/demo/src/**/*.ts",
                    "kind": "file",
                    "rules": {
                        "core/error/failure-surface-maturity": {
                            "payload": {
                                "max_raw_failures": 4,
                                "max_raw_failure_ratio_percent": 60,
                                "structured_guards": ["ensure"]
                            }
                        }
                    }
                }
            ]
        }"#,
    );
    let result = run_scan(&config).unwrap();

    assert!(
        !result
            .issues
            .iter()
            .any(|issue| issue.rule == ERROR_FAILURE_SURFACE_MATURITY),
        "structured guard facts should keep raw failure ratio below the warning line: {:?}",
        result.issues
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn warns_reject_surface() {
    let root = test_root("failure-surface-reject");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        source_dir.join("async.ts"),
        r#"function one() { return Promise.reject(new Error("one")); }
function two() { return Promise.reject(new TypeError("two")); }
function three(reject: (error: Error) => void) { reject(new RangeError("three")); }
function four() { return Promise.reject(new Error("four")); }
function five() { return Promise.reject(new Error("five")); }
"#,
    )
    .unwrap();

    let config = test_config(&root, "tools/*/src/**");
    let result = run_scan(&config).unwrap();

    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.rule == ERROR_FAILURE_SURFACE_MATURITY
                && issue.message.contains("raw failure construction appears 5")),
        "expected reject calls to count as raw failure surface: {:?}",
        result.issues
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn warns_failure_aggregate() {
    let root = test_root("failure-surface-aggregate");
    let source_dir = root.join("tools/demo/src");
    fs::create_dir_all(&source_dir).unwrap();
    for index in 0..4 {
        fs::write(
            source_dir.join(format!("raw_{index}.ts")),
            format!(
                "function a{index}() {{ throw new Error('a'); }}\n\
                 function b{index}() {{ throw new TypeError('b'); }}\n\
                 function c{index}() {{ return Promise.reject(new RangeError('c')); }}\n"
            ),
        )
        .unwrap();
    }
    for index in 0..6 {
        fs::write(
            source_dir.join(format!("clean_{index}.ts")),
            format!("function ok{index}() {{ return {index}; }}\n"),
        )
        .unwrap();
    }

    let config = test_config(&root, "tools/*/src/**");
    let result = run_scan(&config).unwrap();

    assert!(
        result.issues.iter().any(|issue| {
            issue.rule == ERROR_FAILURE_SURFACE_AGGREGATE
                && issue.path == "."
                && issue.message.contains("12 time(s) across 4 file(s)")
        }),
        "expected repo-level failure surface aggregate warning: {:?}",
        result.issues
    );
    assert!(
        !result
            .issues
            .iter()
            .any(|issue| issue.rule == ERROR_FAILURE_SURFACE_MATURITY),
        "aggregate fixture should not need per-file warnings: {:?}",
        result.issues
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn warns_deepest_failure_aggregate() {
    let root = test_root("failure-surface-dir-aggregate");
    let source_dir = root.join("tools/demo/src/hot");
    fs::create_dir_all(&source_dir).unwrap();
    for index in 0..3 {
        fs::write(
            source_dir.join(format!("raw_{index}.ts")),
            format!(
                "function a{index}() {{ throw new Error('a'); }}\n\
                 function b{index}() {{ throw new TypeError('b'); }}\n\
                 function c{index}() {{ return Promise.reject(new RangeError('c')); }}\n"
            ),
        )
        .unwrap();
    }
    for index in 0..3 {
        fs::write(
            source_dir.join(format!("clean_{index}.ts")),
            format!("function ok{index}() {{ return {index}; }}\n"),
        )
        .unwrap();
    }

    let config = test_config(&root, "tools/*/src/**");
    let result = run_scan(&config).unwrap();
    let aggregate_issues = result
        .issues
        .iter()
        .filter(|issue| issue.rule == ERROR_FAILURE_SURFACE_AGGREGATE)
        .collect::<Vec<_>>();

    assert_eq!(
        aggregate_issues.len(),
        1,
        "expected deepest directory aggregate only: {:?}",
        result.issues
    );
    let issue = aggregate_issues[0];
    assert_eq!(issue.path, "tools/demo/src/hot");
    assert!(
        issue
            .message
            .contains("9 time(s) across 3 file(s) in 6 scanned file(s)"),
        "unexpected message: {}",
        issue.message
    );
    assert!(
        !result
            .issues
            .iter()
            .any(|issue| issue.rule == ERROR_FAILURE_SURFACE_MATURITY),
        "directory aggregate fixture should stay below per-file thresholds: {:?}",
        result.issues
    );

    let _ = fs::remove_dir_all(root);
}

fn repeated_handlers(count: usize) -> String {
    let mut source = String::new();
    for index in 0..count {
        source.push_str(&format!(
            "fn handle_{index}(input_{index}: i32) -> i32 {{\n\
                let local_{index} = input_{index};\n\
                match local_{index} {{\n\
                    0 => {{\n\
                        let next_{index}_a = local_{index};\n\
                        next_{index}_a\n\
                    }}\n\
                    1 => {{\n\
                        let next_{index}_b = local_{index};\n\
                        next_{index}_b\n\
                    }}\n\
                    2 => {{\n\
                        let next_{index}_c = local_{index};\n\
                        next_{index}_c\n\
                    }}\n\
                    _ => {{\n\
                        local_{index}\n\
                    }}\n\
                }}\n\
             }}\n\n"
        ));
    }
    source
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
