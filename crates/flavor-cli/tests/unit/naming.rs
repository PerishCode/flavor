use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    sync::LazyLock,
};

use crate::{
    config::{source_file_kind, GuardConfig},
    model::Issue,
    naming::count_name_words,
    path_match::path_string,
    plugins::{PluginHost, Scope},
    rules::{DISPATCH_BRANCH_TOO_LONG, NAMING_AFFIX_PRESSURE, VUE_PARSE_ERROR},
};

static CONFIG: LazyLock<GuardConfig> = LazyLock::new(|| GuardConfig::core(PathBuf::from(".")));
static TEMP_SEQ: AtomicUsize = AtomicUsize::new(0);

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
fn ignores_small_affix_buckets() {
    let relative = Path::new("sample.rs");
    let issues = source_issues(
        relative,
        r#"
fn name_fact() {}
fn line_count_fact() {}
fn descriptor_block_fact() {}
fn embedded_script_fact() {}
fn named_span_fact() {}
"#,
    );

    assert!(!issues
        .iter()
        .any(|issue| issue.rule == NAMING_AFFIX_PRESSURE));
}

#[test]
fn reports_affix_pressure() {
    let relative = Path::new("sample.rs");
    let config = config_with_affix_threshold(3);
    let issues = source_issues_with_config(
        &config,
        relative,
        r#"
fn parse_token() {}
fn parse_statement() {}
fn parse_expression() {}
"#,
    );

    let issue = issues
        .iter()
        .find(|issue| issue.rule == NAMING_AFFIX_PRESSURE)
        .expect("affix pressure issue");
    assert_eq!(issue.line, Some(2));
    assert!(issue
        .message
        .contains("3 function names share the prefix `parse`"));
    assert!(issue.message.contains("move from the name into"));
    assert!(issue.message.contains("too broad"));
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
    source_issues_with_config(&CONFIG, relative, source)
}

fn source_issues_with_config(config: &GuardConfig, relative: &Path, source: &str) -> Vec<Issue> {
    let host = PluginHost::bundled();
    let path = path_string(relative);
    let kind = source_file_kind(relative).expect("test source should have supported extension");
    let mut issues = Vec::new();
    host.run_scope(
        config,
        Scope::source_file(relative, &path, source, kind),
        &mut issues,
    );
    issues
}

fn config_with_affix_threshold(min_occurrences: usize) -> GuardConfig {
    let root = temp_root("naming-affix");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("flavor.toml");
    fs::write(
        &path,
        format!(
            r#"[scan]
include = ["**/*.rs"]

[[overrides]]
match = "**/*.rs"
kind = "file"

[overrides.rules."core/naming/affix-pressure".payload]
min_occurrences = {min_occurrences}
"#
        ),
    )
    .unwrap();
    let config = GuardConfig::from_file(root.clone(), &path).unwrap();
    fs::remove_dir_all(root).unwrap();
    config
}

fn temp_root(slug: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "flavor-{slug}-{}-{}",
        std::process::id(),
        TEMP_SEQ.fetch_add(1, Ordering::Relaxed)
    ))
}
