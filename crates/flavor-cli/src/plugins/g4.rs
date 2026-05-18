use flavor_plugin_g4::{G4_PARSE_ERROR, PLUGIN_ID, RULES};

use crate::{
    config::{NodeKind, SourceKind},
    model::{issue, Issue},
    plugins::{
        AnalysisContext, FactUse, GrammarUse, PluginManifest, PluginOutput, ScopeDecl, ScopeKind,
        SourceFileScope,
    },
};

const SCOPES: &[ScopeDecl] = &[ScopeDecl::source_file(SourceKind::G4)];
const GRAMMARS: &[GrammarUse] = &[GrammarUse {
    scope: ScopeKind::SourceFile,
    grammar_id: "g4",
    entrypoint: "grammar_file",
}];
const FACTS: &[FactUse] = &[
    FactUse {
        grammar_id: "g4",
        key: "grammar.declaration",
        contains: &["payload.name", "payload.kind", "line"],
    },
    FactUse {
        grammar_id: "g4",
        key: "grammar.parser_rule",
        contains: &["payload.name", "payload.references", "line"],
    },
    FactUse {
        grammar_id: "g4",
        key: "grammar.lexer_token",
        contains: &["payload.name", "line"],
    },
    FactUse {
        grammar_id: "g4",
        key: "grammar.reference",
        contains: &["payload.name", "line"],
    },
];

pub(crate) const MANIFEST: PluginManifest = PluginManifest {
    id: PLUGIN_ID,
    scopes: SCOPES,
    grammars: GRAMMARS,
    facts: FACTS,
    rules: RULES,
};

pub(crate) fn analyze<'a>(context: &AnalysisContext<'a>) -> PluginOutput<'a> {
    let Some(scope) = context.scope.source_file_data() else {
        return PluginOutput::default();
    };
    let mut issues = Vec::new();
    push_parse_issues(context, scope, &mut issues);
    PluginOutput::issues(issues)
}

fn push_parse_issues(
    context: &AnalysisContext<'_>,
    scope: SourceFileScope<'_>,
    issues: &mut Vec<Issue>,
) {
    let rule = context
        .config
        .rule(scope.relative, NodeKind::File, G4_PARSE_ERROR);
    if !rule.enabled {
        return;
    }
    for diagnostic in context.products.diagnostics("g4") {
        issues.push(issue(
            rule.severity,
            rule.id,
            scope.path,
            diagnostic.line,
            format!("G4 parse error: {}", diagnostic.message),
        ));
    }
}
