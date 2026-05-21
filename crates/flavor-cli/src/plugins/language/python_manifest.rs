use crate::{
    config::SourceKind,
    plugins::{FactUse, GrammarUse, PluginManifest, ScopeDecl, ScopeKind},
    rules::{
        DISPATCH_BRANCH_TOO_LONG, FUNCTION_TOO_LONG, NAMING_TOO_MANY_WORDS, PYTHON_PARSE_ERROR,
    },
};

const PYTHON_SCOPES: &[ScopeDecl] = &[ScopeDecl::source_file(SourceKind::Python)];
const PYTHON_GRAMMARS: &[GrammarUse] = &[GrammarUse {
    scope: ScopeKind::SourceFile,
    grammar_id: "python",
    entrypoint: "program",
}];
const PYTHON_RULES: &[&str] = &[
    DISPATCH_BRANCH_TOO_LONG,
    FUNCTION_TOO_LONG,
    NAMING_TOO_MANY_WORDS,
    PYTHON_PARSE_ERROR,
];
const PYTHON_FACTS: &[FactUse] = &[
    fact(
        "python",
        "name.function",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "python",
        "name.method",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "python",
        "name.binding",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "python",
        "name.parameter",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "python",
        "function.body",
        &[
            "payload.name",
            "payload.kind",
            "payload.lines",
            "span",
            "line",
        ],
    ),
    fact(
        "python",
        "dispatch.branch",
        &["payload.lines", "span", "line"],
    ),
];

pub(crate) const PYTHON_MANIFEST: PluginManifest = PluginManifest {
    id: "flavor-plugin-python",
    scopes: PYTHON_SCOPES,
    grammars: PYTHON_GRAMMARS,
    facts: PYTHON_FACTS,
    rules: PYTHON_RULES,
};

const fn fact(
    grammar_id: &'static str,
    key: &'static str,
    contains: &'static [&'static str],
) -> FactUse {
    FactUse {
        grammar_id,
        key,
        contains,
    }
}
