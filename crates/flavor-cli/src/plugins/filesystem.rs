use std::path::Path;

use flavor_core::PendingIssue;
use flavor_plugin_filesystem::{
    self as plugin_fs, DirectoryChildrenInput, DirectoryChildrenRule, FilePathInput,
    ForbiddenExtensionRule, NameShapeRule, FS_CHILDREN_SHAPE, FS_FORBIDDEN_EXTENSION,
    FS_NAME_SHAPE, PLUGIN_ID, RULES,
};

use crate::{
    config::{GuardConfig, NodeKind},
    model::{issue, Issue},
    plugins::{
        AnalysisContext, DirectoryChildrenScope, FilePathScope, PluginManifest, PluginOutput,
        ScopeDecl,
    },
    rules::{
        PAYLOAD_ALLOWED, PAYLOAD_CASE, PAYLOAD_EXTENSIONS, PAYLOAD_FORBIDDEN, PAYLOAD_MAX_WORDS,
        PAYLOAD_REQUIRED,
    },
};

const SCOPES: &[ScopeDecl] = &[ScopeDecl::file_path(), ScopeDecl::directory_children()];

pub(crate) const MANIFEST: PluginManifest = PluginManifest {
    id: PLUGIN_ID,
    scopes: SCOPES,
    grammars: &[],
    facts: &[],
    rules: RULES,
};

pub(crate) fn analyze<'a>(context: &AnalysisContext<'a>) -> PluginOutput<'a> {
    let mut issues = Vec::new();
    if let Some(scope) = context.scope.file_path_data() {
        analyze_file_path(context.config, scope, &mut issues);
    }
    if let Some(scope) = context.scope.directory_children_data() {
        analyze_directory_children(context.config, scope, &mut issues);
    }
    PluginOutput::issues(issues)
}

fn analyze_file_path(config: &GuardConfig, scope: FilePathScope<'_>, issues: &mut Vec<Issue>) {
    let forbidden_rule = config.rule(scope.relative, NodeKind::File, FS_FORBIDDEN_EXTENSION);
    let name_rule = config.rule(scope.relative, NodeKind::File, FS_NAME_SHAPE);
    let pending = plugin_fs::analyze_file_path(FilePathInput {
        relative: scope.relative,
        forbidden_extension: ForbiddenExtensionRule {
            enabled: forbidden_rule.enabled,
            extensions: forbidden_rule.string_list(PAYLOAD_EXTENSIONS),
        },
        name_shape: NameShapeRule {
            enabled: name_rule.enabled,
            case: name_rule.string(PAYLOAD_CASE),
            max_words: name_rule.usize(PAYLOAD_MAX_WORDS),
        },
    });
    push_pending_issues(config, scope.relative, NodeKind::File, pending, issues);
}

fn analyze_directory_children(
    config: &GuardConfig,
    scope: DirectoryChildrenScope<'_>,
    issues: &mut Vec<Issue>,
) {
    let rule = config.rule(scope.relative, NodeKind::Dir, FS_CHILDREN_SHAPE);
    let pending = plugin_fs::analyze_directory_children(DirectoryChildrenInput {
        relative: scope.relative,
        children: scope.children,
        rule: DirectoryChildrenRule {
            enabled: rule.enabled,
            required: rule.string_list(PAYLOAD_REQUIRED).unwrap_or_default(),
            allowed: rule.string_list(PAYLOAD_ALLOWED),
            forbidden: rule.string_list(PAYLOAD_FORBIDDEN).unwrap_or_default(),
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
