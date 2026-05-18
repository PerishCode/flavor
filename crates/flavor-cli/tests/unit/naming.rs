use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crate::{
    config::{source_file_kind, GuardConfig},
    model::Issue,
    naming::count_name_words,
    path_match::path_string,
    plugins::{PluginHost, Scope},
    rules::{DISPATCH_BRANCH_TOO_LONG, VUE_PARSE_ERROR},
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
    let relative = Path::new("sample.rs");
    let issues = source_issues(
        relative,
        "fn guard_sample_over_limit_name() { let guard_sample_value_over_limit = 1; }",
    );

    assert_eq!(issues.len(), 2);
    assert!(issues[0].message.contains("guard_sample_over_limit_name"));
    assert!(issues[1].message.contains("guard_sample_value_over_limit"));
}

#[test]
fn groups_trait_impl_names() {
    let relative = Path::new("sample.rs");
    let issues = source_issues(
        relative,
        r#"
trait Repo {
    fn find_primary_by_account_id(&self);
}

impl Repo for SeaOrmRepo {
    fn find_primary_by_account_id(&self) {}
}

impl SeaOrmRepo {
    fn find_secondary_by_account_id(&self) {}
}
"#,
    );

    assert_eq!(issues.len(), 2);
    assert!(issues[0].message.contains("find_primary_by_account_id"));
    assert!(issues[1].message.contains("find_secondary_by_account_id"));
}

#[test]
fn ts_detects_long_names() {
    let relative = Path::new("sample.ts");
    let issues = source_issues(
        relative,
        "function rendererOperationEventHandlerName(inputValue: string) { const controllerRuntimeResultValueText = inputValue; }",
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
    let relative = Path::new("Sample.vue");
    let issues = source_issues(
        relative,
        "<template></template>\n<script setup lang=\"ts\">\nconst controllerRuntimeResultValueText = 1;\n</script>",
    );

    assert_eq!(issues[0].line, Some(3));
}

#[test]
fn vue_reports_sfc_errors() {
    let relative = Path::new("Sample.vue");
    let issues = source_issues(
        relative,
        "<script setup>const first = 1;</script>\n<script setup>const second = 2;</script>",
    );

    assert!(issues.iter().any(|issue| {
        issue.rule == VUE_PARSE_ERROR
            && issue.line == Some(2)
            && issue.message.contains("duplicate top-level <script setup>")
    }));
}

#[test]
fn vue_reports_template_exprs() {
    let relative = Path::new("Sample.vue");
    let issues = source_issues(
        relative,
        r#"<template><div :class="user.">{{ call( }}</div></template>"#,
    );

    assert!(issues.iter().any(|issue| {
        issue.rule == VUE_PARSE_ERROR && issue.message.contains("expected property name")
    }));
    assert!(issues.iter().any(|issue| {
        issue.rule == VUE_PARSE_ERROR
            && issue
                .message
                .contains("expected ')' to close call arguments")
    }));
}

#[test]
fn vue_checks_script_blocks() {
    let relative = Path::new("Sample.vue");
    let issues = source_issues(
        relative,
        "<script lang=\"ts\">\nconst rendererOperationEventHandlerName = 1;\n</script>\n<script setup lang=\"ts\">\nconst controllerRuntimeResultValueText = 1;\n</script>",
    );

    assert!(issues
        .iter()
        .any(|issue| issue.message.contains("rendererOperationEventHandlerName")));
    assert!(issues
        .iter()
        .any(|issue| issue.message.contains("controllerRuntimeResultValueText")));
}

#[test]
fn flags_match_arm() {
    let relative = Path::new("sample.rs");
    let repeated_body = "let a = 1;\n".repeat(25);
    let source =
        format!("fn route(x: i32) {{\nmatch x {{\n1 => {{\n{repeated_body}}}\n_ => {{}}\n}}\n}}");

    let issues = source_issues(relative, &source);

    assert!(issues
        .iter()
        .any(|issue| issue.rule == DISPATCH_BRANCH_TOO_LONG));
}

#[test]
fn flags_switch_case() {
    let relative = Path::new("sample.ts");
    let repeated_body = "x += 1;\n".repeat(25);
    let source = format!(
        "function route(x: number) {{\nswitch (x) {{\ncase 1:\n{repeated_body}break;\ndefault:\nbreak;\n}}\n}}"
    );

    let issues = source_issues(relative, &source);

    assert!(issues
        .iter()
        .any(|issue| issue.rule == DISPATCH_BRANCH_TOO_LONG));
}

fn source_issues(relative: &Path, source: &str) -> Vec<Issue> {
    let host = PluginHost::bundled();
    let path = path_string(relative);
    let kind = source_file_kind(relative).expect("test source should have supported extension");
    let mut issues = Vec::new();
    host.run_scope(
        &CONFIG,
        Scope::source_file(relative, &path, source, kind),
        &mut issues,
    );
    issues
}
