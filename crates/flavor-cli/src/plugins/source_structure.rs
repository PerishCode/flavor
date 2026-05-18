use std::path::Path;

use flavor_core::PendingIssue;
use flavor_plugin_source_structure::{
    self as plugin_source, DirectoryChildrenInput, DirectoryChildrenRule, SourceDirectoryInput,
    SourceDirectoryRule, SourceFileInput, SourceFileRule, FS_TOO_MANY_CHILDREN, PLUGIN_ID, RULES,
    SOURCE_TOO_DEEP, SOURCE_TOO_LONG,
};

use crate::{
    config::{GuardConfig, NodeKind},
    model::{issue, Issue},
    plugins::{
        AnalysisContext, DirectoryChildrenScope, PluginManifest, PluginOutput, ScopeDecl,
        SourceDirectoryScope, SourceFileScope,
    },
    rules::{PAYLOAD_MAX_CHILDREN, PAYLOAD_MAX_DEPTH, PAYLOAD_MAX_LINES},
};

const SCOPES: &[ScopeDecl] = &[
    ScopeDecl::any_source_file(),
    ScopeDecl::source_directory(),
    ScopeDecl::directory_children(),
];

pub(crate) const MANIFEST: PluginManifest = PluginManifest {
    id: PLUGIN_ID,
    scopes: SCOPES,
    grammars: &[],
    facts: &[],
    rules: RULES,
};

pub(crate) fn analyze<'a>(context: &AnalysisContext<'a>) -> PluginOutput<'a> {
    let mut issues = Vec::new();
    if let Some(scope) = context.scope.source_file_data() {
        analyze_source_file(context.config, scope, &mut issues);
    }
    if let Some(scope) = context.scope.source_directory_data() {
        analyze_source_directory(context.config, scope, &mut issues);
    }
    if let Some(scope) = context.scope.directory_children_data() {
        analyze_directory_children(context.config, scope, &mut issues);
    }
    PluginOutput::issues(issues)
}

fn analyze_source_file(config: &GuardConfig, scope: SourceFileScope<'_>, issues: &mut Vec<Issue>) {
    let rule = config.rule(scope.relative, NodeKind::File, SOURCE_TOO_LONG);
    let pending = plugin_source::analyze_source_file(SourceFileInput {
        path: scope.path,
        source: scope.source,
        rule: SourceFileRule {
            enabled: rule.enabled,
            max_lines: rule.usize(PAYLOAD_MAX_LINES).unwrap_or(500),
        },
    });
    push_pending_issues(config, scope.relative, NodeKind::File, pending, issues);
}

fn analyze_source_directory(
    config: &GuardConfig,
    scope: SourceDirectoryScope<'_>,
    issues: &mut Vec<Issue>,
) {
    let rule = config.rule(scope.relative, NodeKind::Dir, SOURCE_TOO_DEEP);
    let pending = plugin_source::analyze_source_directory(SourceDirectoryInput {
        relative: scope.relative,
        rule: SourceDirectoryRule {
            enabled: rule.enabled,
            max_depth: rule.usize(PAYLOAD_MAX_DEPTH).unwrap_or(4),
        },
    });
    push_pending_issues(config, scope.relative, NodeKind::Dir, pending, issues);
}

fn analyze_directory_children(
    config: &GuardConfig,
    scope: DirectoryChildrenScope<'_>,
    issues: &mut Vec<Issue>,
) {
    let rule = config.rule(scope.relative, NodeKind::Dir, FS_TOO_MANY_CHILDREN);
    let pending = plugin_source::analyze_directory_children(DirectoryChildrenInput {
        relative: scope.relative,
        source_child_count: scope.source_child_count,
        children: scope.children,
        rule: DirectoryChildrenRule {
            enabled: rule.enabled,
            max_children: rule.usize(PAYLOAD_MAX_CHILDREN).unwrap_or(10),
        },
    });
    push_pending_issues(config, scope.relative, NodeKind::Dir, pending, issues);
}

fn push_pending_issues(
    config: &GuardConfig,
    relative: &Path,
    kind: NodeKind,
    pending: Vec<PendingIssue>,
    issues: &mut Vec<Issue>,
) {
    for pending in pending {
        let rule = config.rule(relative, kind, pending.rule_id);
        issues.push(issue(
            rule.severity,
            rule.id,
            pending.path,
            pending.line,
            pending.message,
        ));
    }
}
