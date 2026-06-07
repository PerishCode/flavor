use crate::{
    config::NodeKind,
    plugins::{AnalysisContext, PluginOutput},
    rules::{
        DISPATCH_BRANCH_TOO_LONG, FUNCTION_TOO_LONG, NAMING_AFFIX_PRESSURE, NAMING_TOO_MANY_WORDS,
        PYTHON_PARSE_ERROR,
    },
};

use super::{check_dispatch_branches, check_function_bodies, check_name_facts, push_parse_issues};

pub(crate) fn analyze_python_source<'a>(context: &AnalysisContext<'a>) -> PluginOutput<'a> {
    let Some(scope) = context.scope.source_file_data() else {
        return PluginOutput::default();
    };

    let mut issues = Vec::new();
    let parse_rule = context
        .config
        .rule(scope.relative, NodeKind::File, PYTHON_PARSE_ERROR);
    push_parse_issues(
        &mut issues,
        &parse_rule,
        scope.path,
        context.products.diagnostics("python"),
        "Python",
    );

    let name_rule = context
        .config
        .rule(scope.relative, NodeKind::File, NAMING_TOO_MANY_WORDS);
    let affix_rule = context
        .config
        .rule(scope.relative, NodeKind::File, NAMING_AFFIX_PRESSURE);
    check_name_facts(
        &context.products,
        "python",
        &name_rule,
        &affix_rule,
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
        context.products.facts("python", "dispatch.branch"),
        "Python branch body",
    );

    let function_rule = context
        .config
        .rule(scope.relative, NodeKind::File, FUNCTION_TOO_LONG);
    check_function_bodies(
        &mut issues,
        &function_rule,
        scope.path,
        context.products.facts("python", "function.body"),
    );

    PluginOutput::issues(issues)
}
