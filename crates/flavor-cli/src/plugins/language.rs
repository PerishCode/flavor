mod manifest;

pub(crate) use manifest::{RUST_MANIFEST, SVELTE_MANIFEST, TYPESCRIPT_MANIFEST, VUE_MANIFEST};

use std::{collections::BTreeSet, path::Path};

use crate::{
    config::{GuardConfig, NodeKind, RuleSettings},
    model::{issue, Issue},
    naming::check_name,
    plugins::{AnalysisContext, PluginOutput, ProductSet, SourceFileScope},
    rules::{
        DISPATCH_BRANCH_TOO_LONG, NAMING_TOO_MANY_WORDS, PAYLOAD_ALLOWED_INTRINSICS,
        PAYLOAD_MAX_BLOCKS, PAYLOAD_MAX_BRANCH_LINES, PAYLOAD_MAX_LINES, PAYLOAD_PRIMITIVE_SOURCES,
        RUST_PARSE_ERROR, RUST_TESTS_IN_SOURCE, SVELTE_COMPONENT_TOO_LONG, SVELTE_PARSE_ERROR,
        SVELTE_SCRIPT_TOO_LONG, SVELTE_STYLE_TOO_LONG, SVELTE_TEMPLATE_TOO_COMPLEX,
        TSX_NO_INTRINSICS, TSX_REQUIRES_PRIMITIVE, TS_PARSE_ERROR, VUE_PARSE_ERROR,
    },
};
use flavor_plugin_core::{Fact, ProductDiagnostic};

pub(crate) fn analyze_rust_source<'a>(context: &AnalysisContext<'a>) -> PluginOutput<'a> {
    let Some(scope) = context.scope.source_file_data() else {
        return PluginOutput::default();
    };

    let mut issues = Vec::new();
    let parse_rule = context
        .config
        .rule(scope.relative, NodeKind::File, RUST_PARSE_ERROR);
    push_parse_issues(
        &mut issues,
        &parse_rule,
        scope.path,
        context.products.diagnostics("rust"),
        "Rust",
    );

    let name_rule = context
        .config
        .rule(scope.relative, NodeKind::File, NAMING_TOO_MANY_WORDS);
    check_name_facts(
        &context.products,
        "rust",
        &[
            "name.function",
            "name.method",
            "name.binding",
            "name.parameter",
        ],
        &name_rule,
        scope.path,
        &mut issues,
    );

    let dispatch_rule =
        context
            .config
            .rule(scope.relative, NodeKind::File, DISPATCH_BRANCH_TOO_LONG);
    check_dispatch_branches(
        &mut issues,
        &dispatch_rule,
        scope.path,
        context.products.facts("rust", "dispatch.branch"),
        "match arm body",
    );

    let test_rule = context
        .config
        .rule(scope.relative, NodeKind::File, RUST_TESTS_IN_SOURCE);
    check_rust_tests(
        scope,
        context.products.facts("rust", "test.attribute"),
        &test_rule,
        &mut issues,
    );

    PluginOutput::issues(issues)
}

pub(crate) fn analyze_typescript_source<'a>(context: &AnalysisContext<'a>) -> PluginOutput<'a> {
    let Some(scope) = context.scope.source_file_data() else {
        return PluginOutput::default();
    };

    let mut issues = Vec::new();
    analyze_typescript_products(context.config, scope, &context.products, &mut issues);
    PluginOutput::issues(issues)
}

pub(crate) fn analyze_vue_source<'a>(context: &AnalysisContext<'a>) -> PluginOutput<'a> {
    let Some(scope) = context.scope.source_file_data() else {
        return PluginOutput::default();
    };
    let mut issues = Vec::new();
    let parse_rule = context
        .config
        .rule(scope.relative, NodeKind::File, VUE_PARSE_ERROR);
    push_parse_issues(
        &mut issues,
        &parse_rule,
        scope.path,
        context.products.diagnostics("vue-sfc"),
        "Vue SFC",
    );
    analyze_typescript_products(context.config, scope, &context.products, &mut issues);
    PluginOutput::issues(issues)
}

pub(crate) fn analyze_svelte_source<'a>(context: &AnalysisContext<'a>) -> PluginOutput<'a> {
    let Some(scope) = context.scope.source_file_data() else {
        return PluginOutput::default();
    };
    let mut issues = Vec::new();
    let parse_rule = context
        .config
        .rule(scope.relative, NodeKind::File, SVELTE_PARSE_ERROR);
    push_parse_issues(
        &mut issues,
        &parse_rule,
        scope.path,
        context.products.diagnostics("svelte"),
        "Svelte",
    );
    check_svelte_shape(context.config, scope, &context.products, &mut issues);
    analyze_typescript_products(context.config, scope, &context.products, &mut issues);
    PluginOutput::issues(issues)
}

fn analyze_typescript_products(
    config: &GuardConfig,
    scope: SourceFileScope<'_>,
    products: &ProductSet,
    issues: &mut Vec<Issue>,
) {
    let parse_rule = config.rule(scope.relative, NodeKind::File, TS_PARSE_ERROR);
    push_parse_issues(
        issues,
        &parse_rule,
        scope.path,
        products.diagnostics("typescript"),
        "TypeScript",
    );
    check_tsx_rules(config, scope, products, issues);

    let name_rule = config.rule(scope.relative, NodeKind::File, NAMING_TOO_MANY_WORDS);
    check_name_facts(
        products,
        "typescript",
        &[
            "name.function",
            "name.method",
            "name.binding",
            "name.parameter",
        ],
        &name_rule,
        scope.path,
        issues,
    );

    let dispatch_rule = config.rule(scope.relative, NodeKind::File, DISPATCH_BRANCH_TOO_LONG);
    check_dispatch_branches(
        issues,
        &dispatch_rule,
        scope.path,
        products.facts("typescript", "dispatch.branch"),
        "switch case",
    );
}

fn check_name_facts(
    products: &ProductSet,
    grammar_id: &'static str,
    keys: &[&'static str],
    rule: &RuleSettings,
    path: &str,
    issues: &mut Vec<Issue>,
) {
    for key in keys {
        for fact in products.facts(grammar_id, key) {
            let Some(name) = fact.text("name") else {
                continue;
            };
            let Some(line) = fact.line else {
                continue;
            };
            let kind = fact
                .text("issue_kind")
                .or_else(|| fact.text("kind"))
                .unwrap_or("name");
            check_name(issues, rule, path, line, kind, name);
        }
    }
}

fn check_dispatch_branches<'a>(
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
    path: &str,
    branches: impl Iterator<Item = &'a Fact>,
    label: &str,
) {
    if !rule.enabled {
        return;
    }
    let max_lines = rule.usize(PAYLOAD_MAX_BRANCH_LINES).unwrap_or(24);
    for branch in branches {
        let Some(lines) = branch.usize("lines") else {
            continue;
        };
        let Some(line) = branch.line else {
            continue;
        };
        if lines > max_lines {
            issues.push(issue(
                rule.severity,
                rule.id,
                path,
                Some(line),
                format!("{label} spans {lines} lines; max is {max_lines}"),
            ));
        }
    }
}

fn check_rust_tests<'a>(
    scope: SourceFileScope<'_>,
    test_attributes: impl Iterator<Item = &'a Fact>,
    rule: &RuleSettings,
    issues: &mut Vec<Issue>,
) {
    if !rule.enabled || !has_src_component(scope.relative) {
        return;
    }
    for attr in test_attributes {
        let Some(line) = attr.line else {
            continue;
        };
        issues.push(issue(
            rule.severity,
            rule.id,
            scope.path,
            Some(line),
            "Rust test code belongs under a tests directory sibling to src",
        ));
    }
}

fn check_svelte_shape(
    config: &GuardConfig,
    scope: SourceFileScope<'_>,
    products: &ProductSet,
    issues: &mut Vec<Issue>,
) {
    let Some(shape) = products.facts("svelte", "descriptor.markup").next() else {
        return;
    };
    let component_rule = config.rule(scope.relative, NodeKind::File, SVELTE_COMPONENT_TOO_LONG);
    let script_rule = config.rule(scope.relative, NodeKind::File, SVELTE_SCRIPT_TOO_LONG);
    let style_rule = config.rule(scope.relative, NodeKind::File, SVELTE_STYLE_TOO_LONG);
    let template_rule = config.rule(scope.relative, NodeKind::File, SVELTE_TEMPLATE_TOO_COMPLEX);

    if component_rule.enabled {
        let max_lines = component_rule.usize(PAYLOAD_MAX_LINES).unwrap_or(500);
        let line_count = shape.usize("line_count").unwrap_or_default();
        if line_count > max_lines {
            issues.push(issue(
                component_rule.severity,
                component_rule.id,
                scope.path,
                None,
                format!(
                    "Svelte component has {} lines; max is {max_lines}; breakdown: script {} lines, markup {} non-empty lines, style {} lines",
                    line_count,
                    shape.usize("script_lines").unwrap_or_default(),
                    shape.usize("markup_lines").unwrap_or_default(),
                    shape.usize("style_lines").unwrap_or_default()
                ),
            ));
        }
    }

    if script_rule.enabled {
        let max_lines = script_rule.usize(PAYLOAD_MAX_LINES).unwrap_or(180);
        let script_lines = shape.usize("script_lines").unwrap_or_default();
        if script_lines > max_lines {
            issues.push(issue(
                script_rule.severity,
                script_rule.id,
                scope.path,
                None,
                format!(
                    "Svelte script spans {} lines across {} block(s); max is {max_lines}",
                    script_lines,
                    shape.usize("script_count").unwrap_or_default()
                ),
            ));
        }
    }

    if style_rule.enabled {
        let max_lines = style_rule.usize(PAYLOAD_MAX_LINES).unwrap_or(240);
        let style_lines = shape.usize("style_lines").unwrap_or_default();
        if style_lines > max_lines {
            issues.push(issue(
                style_rule.severity,
                style_rule.id,
                scope.path,
                None,
                format!(
                    "Svelte style spans {} lines across {} block(s); max is {max_lines}",
                    style_lines,
                    shape.usize("style_count").unwrap_or_default()
                ),
            ));
        }
    }

    if template_rule.enabled {
        let max_blocks = template_rule.usize(PAYLOAD_MAX_BLOCKS).unwrap_or(18);
        let block_count = products.facts("svelte-markup", "markup.block").count();
        let branch_count = products.facts("svelte-markup", "markup.branch").count();
        let render_count = products.facts("svelte-markup", "markup.render").count();
        if block_count > max_blocks {
            issues.push(issue(
                template_rule.severity,
                template_rule.id,
                scope.path,
                None,
                format!(
                    "Svelte template has {} control block(s), {} branch tag(s), and {} render tag(s); max blocks is {max_blocks}",
                    block_count, branch_count, render_count
                ),
            ));
        }
    }
}

fn check_tsx_rules(
    config: &GuardConfig,
    scope: SourceFileScope<'_>,
    products: &ProductSet,
    issues: &mut Vec<Issue>,
) {
    let intrinsic_rule = config.rule(scope.relative, NodeKind::File, TSX_NO_INTRINSICS);
    let primitive_rule = config.rule(scope.relative, NodeKind::File, TSX_REQUIRES_PRIMITIVE);
    if !intrinsic_rule.enabled && !primitive_rule.enabled {
        return;
    }

    let allowed_intrinsics = intrinsic_rule
        .string_list(PAYLOAD_ALLOWED_INTRINSICS)
        .unwrap_or_default()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let primitive_sources = primitive_rule
        .string_list(PAYLOAD_PRIMITIVE_SOURCES)
        .unwrap_or_default()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let primitive_imports = primitive_imports(
        products.facts("typescript", "module.import"),
        &primitive_sources,
    );

    let mut used_primitive = false;
    for element in products
        .facts("tsx", "jsx.element")
        .chain(products.facts("tsx", "jsx.self_closing"))
    {
        if is_primitive_usage(element, &primitive_imports) {
            used_primitive = true;
        }
        let intrinsic = element.text("intrinsic").unwrap_or_default();
        if !intrinsic.is_empty()
            && intrinsic_rule.enabled
            && !allowed_intrinsics.contains(intrinsic)
        {
            issues.push(issue(
                intrinsic_rule.severity,
                intrinsic_rule.id,
                scope.path,
                element.line,
                format!("JSX intrinsic element `<{intrinsic}>` is not allowed in this boundary"),
            ));
        }
    }

    let has_jsx = products.facts("tsx", "jsx.element").next().is_some()
        || products.facts("tsx", "jsx.self_closing").next().is_some();
    if !primitive_rule.enabled || !has_jsx || used_primitive {
        return;
    }
    let sources = if primitive_sources.is_empty() {
        "configured primitive sources".to_string()
    } else {
        primitive_sources
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    };
    issues.push(issue(
        primitive_rule.severity,
        primitive_rule.id,
        scope.path,
        None,
        format!("component JSX does not compose a primitive from {sources}"),
    ));
}

#[derive(Debug, Default)]
struct PrimitiveImports {
    named: BTreeSet<String>,
    namespaces: BTreeSet<String>,
}

fn primitive_imports<'a>(
    imports: impl Iterator<Item = &'a Fact>,
    primitive_sources: &BTreeSet<String>,
) -> PrimitiveImports {
    let mut primitive_imports = PrimitiveImports::default();
    for import in imports {
        let Some(source) = import.text("source") else {
            continue;
        };
        if import.bool("type_only").unwrap_or(false) || !primitive_sources.contains(source) {
            continue;
        }
        for name in import.texts("default_imports").unwrap_or_default() {
            primitive_imports.named.insert(name.clone());
        }
        for name in import.texts("named_imports").unwrap_or_default() {
            primitive_imports.named.insert(name.clone());
        }
        for name in import.texts("namespace_imports").unwrap_or_default() {
            primitive_imports.namespaces.insert(name.clone());
        }
    }
    primitive_imports
}

fn is_primitive_usage(element: &Fact, imports: &PrimitiveImports) -> bool {
    let name = element.text("name").unwrap_or_default();
    let root = element.text("root").unwrap_or_default();
    imports.named.contains(name) || (!root.is_empty() && imports.namespaces.contains(root))
}

fn push_parse_issues<'a>(
    issues: &mut Vec<Issue>,
    rule: &RuleSettings,
    path: &str,
    diagnostics: impl Iterator<Item = &'a ProductDiagnostic>,
    language: &str,
) {
    if !rule.enabled {
        return;
    }
    for diagnostic in diagnostics {
        issues.push(issue(
            rule.severity,
            rule.id,
            path,
            diagnostic.line,
            format!("failed to parse {language} source: {}", diagnostic.message),
        ));
    }
}

fn has_src_component(relative: &Path) -> bool {
    relative
        .components()
        .any(|component| component.as_os_str() == "src")
}
