use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crate::{
    config::{GuardConfig, NodeKind, RuleSettings},
    naming::{check_rust_names, check_ts_names, count_name_words},
    rules::{RUST_PARSE_ERROR, TS_PARSE_ERROR},
};

static CONFIG: LazyLock<GuardConfig> = LazyLock::new(|| GuardConfig::core(PathBuf::from(".")));

#[test]
fn counts_name_words() {
    assert_eq!(count_name_words("controller_operation_event"), 3);
    assert_eq!(count_name_words("controllerOperationEvent"), 3);
    assert_eq!(count_name_words("HTTPClient"), 2);
    assert_eq!(count_name_words("guard_sample_over_limit_name"), 5);
}

#[test]
fn rust_detects_long_names() {
    let mut issues = Vec::new();
    let relative = Path::new("sample.rs");
    let parse_rule = rule(relative, RUST_PARSE_ERROR);
    check_rust_names(
        &CONFIG,
        relative,
        "sample.rs",
        "fn guard_sample_over_limit_name() { let guard_sample_value_over_limit = 1; }",
        &mut issues,
        &parse_rule,
    );

    assert_eq!(issues.len(), 2);
    assert!(issues[0].message.contains("guard_sample_over_limit_name"));
    assert!(issues[1].message.contains("guard_sample_value_over_limit"));
}

#[test]
fn ts_detects_long_names() {
    let mut issues = Vec::new();
    let relative = Path::new("sample.ts");
    let parse_rule = rule(relative, TS_PARSE_ERROR);
    check_ts_names(
        &CONFIG,
        relative,
        "sample.ts",
        "function rendererOperationEventHandlerName(inputValue: string) { const controllerRuntimeResultValueText = inputValue; }",
        &mut issues,
        &parse_rule,
    );

    assert_eq!(issues.len(), 2);
    assert!(issues[0]
        .message
        .contains("rendererOperationEventHandlerName"));
    assert!(issues[1]
        .message
        .contains("controllerRuntimeResultValueText"));
}

#[test]
fn vue_offsets_lines() {
    let mut issues = Vec::new();
    let relative = Path::new("Sample.vue");
    let parse_rule = rule(relative, TS_PARSE_ERROR);
    check_ts_names(
        &CONFIG,
        relative,
        "Sample.vue",
        "<template></template>\n<script setup lang=\"ts\">\nconst controllerRuntimeResultValueText = 1;\n</script>",
        &mut issues,
        &parse_rule,
    );

    assert_eq!(issues[0].line, Some(3));
}

fn rule(relative: &Path, rule_id: &'static str) -> RuleSettings {
    CONFIG.rule(relative, NodeKind::File, rule_id)
}
