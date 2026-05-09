use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crate::{
    config::{GuardConfig, NodeKind, RuleSettings},
    naming::{check_rust_names, check_ts_names, count_name_words},
    rules::{DISPATCH_BRANCH_TOO_LONG, RUST_PARSE_ERROR, TS_PARSE_ERROR},
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

#[test]
fn flags_match_arm() {
    let mut issues = Vec::new();
    let relative = Path::new("sample.rs");
    let parse_rule = rule(relative, RUST_PARSE_ERROR);
    let repeated_body = "let a = 1;\n".repeat(25);
    let source =
        format!("fn route(x: i32) {{\nmatch x {{\n1 => {{\n{repeated_body}}}\n_ => {{}}\n}}\n}}");

    check_rust_names(
        &CONFIG,
        relative,
        "sample.rs",
        &source,
        &mut issues,
        &parse_rule,
    );

    assert!(issues
        .iter()
        .any(|issue| issue.rule == DISPATCH_BRANCH_TOO_LONG));
}

#[test]
fn flags_switch_case() {
    let mut issues = Vec::new();
    let relative = Path::new("sample.ts");
    let parse_rule = rule(relative, TS_PARSE_ERROR);
    let repeated_body = "x += 1;\n".repeat(25);
    let source = format!(
        "function route(x: number) {{\nswitch (x) {{\ncase 1:\n{repeated_body}break;\ndefault:\nbreak;\n}}\n}}"
    );

    check_ts_names(
        &CONFIG,
        relative,
        "sample.ts",
        &source,
        &mut issues,
        &parse_rule,
    );

    assert!(issues
        .iter()
        .any(|issue| issue.rule == DISPATCH_BRANCH_TOO_LONG));
}

fn rule(relative: &Path, rule_id: &'static str) -> RuleSettings {
    CONFIG.rule(relative, NodeKind::File, rule_id)
}
