use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{config::GuardConfig, rules::SHAPE_REPEATED_TOKEN_PATTERN, scan::run_scan};

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
