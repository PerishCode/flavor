use crate::{
    config::SourceKind,
    plugins::{FactUse, GrammarUse, PluginManifest, ScopeDecl, ScopeKind},
    rules::{
        DISPATCH_BRANCH_TOO_LONG, ERROR_FAILURE_SURFACE_AGGREGATE, ERROR_FAILURE_SURFACE_MATURITY,
        NAMING_TOO_MANY_WORDS, RUST_PARSE_ERROR, RUST_TESTS_IN_SOURCE,
        SHAPE_REPEATED_TOKEN_PATTERN, SVELTE_COMPONENT_TOO_LONG, SVELTE_PARSE_ERROR,
        SVELTE_SCRIPT_TOO_LONG, SVELTE_STYLE_TOO_LONG, SVELTE_TEMPLATE_TOO_COMPLEX,
        TSX_NO_INTRINSICS, TSX_REQUIRES_PRIMITIVE, TS_PARSE_ERROR, VUE_PARSE_ERROR,
    },
};

const RUST_SCOPES: &[ScopeDecl] = &[ScopeDecl::source_file(SourceKind::Rust)];
const TYPESCRIPT_SCOPES: &[ScopeDecl] = &[ScopeDecl::source_file(SourceKind::TypeScript)];
const VUE_SCOPES: &[ScopeDecl] = &[ScopeDecl::source_file(SourceKind::Vue)];
const SVELTE_SCOPES: &[ScopeDecl] = &[ScopeDecl::source_file(SourceKind::Svelte)];

const RUST_GRAMMARS: &[GrammarUse] = &[grammar(ScopeKind::SourceFile, "rust", "source_file")];
const TYPESCRIPT_GRAMMARS: &[GrammarUse] = &[
    grammar(ScopeKind::SourceFile, "typescript", "program"),
    grammar(ScopeKind::SourceFile, "tsx", "program"),
];
const VUE_GRAMMARS: &[GrammarUse] = &[
    grammar(ScopeKind::SourceFile, "vue-sfc", "sfc_document"),
    grammar(ScopeKind::SourceFile, "vue-template", "template_document"),
    grammar(ScopeKind::SourceFile, "typescript", "program"),
    grammar(ScopeKind::SourceFile, "tsx", "program"),
];
const SVELTE_GRAMMARS: &[GrammarUse] = &[
    grammar(ScopeKind::SourceFile, "svelte", "svelte_document"),
    grammar(ScopeKind::SourceFile, "svelte-markup", "markup_document"),
    grammar(ScopeKind::SourceFile, "typescript", "program"),
    grammar(ScopeKind::SourceFile, "tsx", "program"),
];

const RUST_RULES: &[&str] = &[
    DISPATCH_BRANCH_TOO_LONG,
    NAMING_TOO_MANY_WORDS,
    RUST_PARSE_ERROR,
    RUST_TESTS_IN_SOURCE,
    SHAPE_REPEATED_TOKEN_PATTERN,
];
const RUST_FACTS: &[FactUse] = &[
    fact(
        "rust",
        "name.function",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "rust",
        "name.method",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "rust",
        "name.binding",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "rust",
        "name.parameter",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "rust",
        "dispatch.branch",
        &["payload.lines", "span", "line"],
    ),
    fact(
        "rust",
        "shape.repeated_token_pattern",
        &[
            "payload.occurrences",
            "payload.total_lines",
            "payload.token_count",
            "payload.node_kind",
            "payload.depth",
            "span",
            "line",
        ],
    ),
    fact("rust", "test.attribute", &["span", "line"]),
];

const TYPESCRIPT_RULES: &[&str] = &[
    DISPATCH_BRANCH_TOO_LONG,
    ERROR_FAILURE_SURFACE_AGGREGATE,
    ERROR_FAILURE_SURFACE_MATURITY,
    NAMING_TOO_MANY_WORDS,
    TS_PARSE_ERROR,
    TSX_NO_INTRINSICS,
    TSX_REQUIRES_PRIMITIVE,
];
const TYPESCRIPT_FACTS: &[FactUse] = &[
    fact(
        "typescript",
        "module.import",
        &[
            "payload.source",
            "payload.type_only",
            "payload.named_imports",
            "span",
            "line",
        ],
    ),
    fact(
        "typescript",
        "name.function",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "name.method",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "name.binding",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "name.parameter",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "dispatch.branch",
        &["payload.lines", "span", "line"],
    ),
    fact(
        "typescript",
        "error.raw_failure",
        &[
            "payload.kind",
            "payload.mechanism",
            "payload.constructor",
            "payload.callee",
            "span",
            "line",
        ],
    ),
    fact(
        "typescript",
        "error.structured_failure",
        &[
            "payload.kind",
            "payload.mechanism",
            "payload.callee",
            "span",
            "line",
        ],
    ),
    fact(
        "tsx",
        "jsx.element",
        &[
            "payload.name",
            "payload.intrinsic",
            "payload.self_closing",
            "line",
        ],
    ),
    fact(
        "tsx",
        "jsx.self_closing",
        &[
            "payload.name",
            "payload.intrinsic",
            "payload.self_closing",
            "line",
        ],
    ),
];

const VUE_RULES: &[&str] = &[
    DISPATCH_BRANCH_TOO_LONG,
    ERROR_FAILURE_SURFACE_AGGREGATE,
    ERROR_FAILURE_SURFACE_MATURITY,
    NAMING_TOO_MANY_WORDS,
    TS_PARSE_ERROR,
    TSX_NO_INTRINSICS,
    TSX_REQUIRES_PRIMITIVE,
    VUE_PARSE_ERROR,
];
const VUE_FACTS: &[FactUse] = &[
    fact(
        "vue-sfc",
        "descriptor.script",
        &["payload.content", "payload.start_line"],
    ),
    fact(
        "vue-sfc",
        "descriptor.script_setup",
        &["payload.content", "payload.start_line"],
    ),
    fact(
        "vue-sfc",
        "script.embedded",
        &["payload.content", "payload.lang", "payload.start_line"],
    ),
    fact(
        "typescript",
        "module.import",
        &["payload.source", "payload.named_imports", "span", "line"],
    ),
    fact(
        "typescript",
        "name.function",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "name.method",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "name.binding",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "name.parameter",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "dispatch.branch",
        &["payload.lines", "span", "line"],
    ),
    fact(
        "typescript",
        "error.raw_failure",
        &[
            "payload.kind",
            "payload.mechanism",
            "payload.constructor",
            "payload.callee",
            "span",
            "line",
        ],
    ),
    fact(
        "typescript",
        "error.structured_failure",
        &[
            "payload.kind",
            "payload.mechanism",
            "payload.callee",
            "span",
            "line",
        ],
    ),
    fact(
        "tsx",
        "jsx.element",
        &[
            "payload.name",
            "payload.intrinsic",
            "payload.self_closing",
            "line",
        ],
    ),
    fact(
        "tsx",
        "jsx.self_closing",
        &[
            "payload.name",
            "payload.intrinsic",
            "payload.self_closing",
            "line",
        ],
    ),
];

const SVELTE_RULES: &[&str] = &[
    DISPATCH_BRANCH_TOO_LONG,
    ERROR_FAILURE_SURFACE_AGGREGATE,
    ERROR_FAILURE_SURFACE_MATURITY,
    NAMING_TOO_MANY_WORDS,
    SVELTE_COMPONENT_TOO_LONG,
    SVELTE_PARSE_ERROR,
    SVELTE_SCRIPT_TOO_LONG,
    SVELTE_STYLE_TOO_LONG,
    SVELTE_TEMPLATE_TOO_COMPLEX,
    TS_PARSE_ERROR,
    TSX_NO_INTRINSICS,
    TSX_REQUIRES_PRIMITIVE,
];
const SVELTE_FACTS: &[FactUse] = &[
    fact(
        "svelte",
        "descriptor.module_script",
        &["payload.content", "payload.start_line"],
    ),
    fact(
        "svelte",
        "descriptor.script",
        &["payload.content", "payload.start_line"],
    ),
    fact(
        "svelte",
        "descriptor.styles",
        &["payload.style_count", "payload.style_lines"],
    ),
    fact(
        "svelte",
        "descriptor.markup",
        &[
            "payload.line_count",
            "payload.markup_lines",
            "payload.markup_block_count",
        ],
    ),
    fact(
        "svelte-markup",
        "markup.block",
        &["payload.name", "span", "line"],
    ),
    fact(
        "svelte-markup",
        "markup.branch",
        &["payload.name", "span", "line"],
    ),
    fact(
        "svelte-markup",
        "markup.render",
        &["payload.name", "span", "line"],
    ),
    fact(
        "typescript",
        "module.import",
        &["payload.source", "payload.named_imports", "span", "line"],
    ),
    fact(
        "typescript",
        "name.function",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "name.method",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "name.binding",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "name.parameter",
        &["payload.name", "payload.issue_kind", "line"],
    ),
    fact(
        "typescript",
        "dispatch.branch",
        &["payload.lines", "span", "line"],
    ),
    fact(
        "typescript",
        "error.raw_failure",
        &[
            "payload.kind",
            "payload.mechanism",
            "payload.constructor",
            "payload.callee",
            "span",
            "line",
        ],
    ),
    fact(
        "typescript",
        "error.structured_failure",
        &[
            "payload.kind",
            "payload.mechanism",
            "payload.callee",
            "span",
            "line",
        ],
    ),
    fact(
        "tsx",
        "jsx.element",
        &[
            "payload.name",
            "payload.intrinsic",
            "payload.self_closing",
            "line",
        ],
    ),
    fact(
        "tsx",
        "jsx.self_closing",
        &[
            "payload.name",
            "payload.intrinsic",
            "payload.self_closing",
            "line",
        ],
    ),
];

pub(crate) const RUST_MANIFEST: PluginManifest = PluginManifest {
    id: "flavor-plugin-rust",
    scopes: RUST_SCOPES,
    grammars: RUST_GRAMMARS,
    facts: RUST_FACTS,
    rules: RUST_RULES,
};

pub(crate) const TYPESCRIPT_MANIFEST: PluginManifest = PluginManifest {
    id: "flavor-plugin-typescript",
    scopes: TYPESCRIPT_SCOPES,
    grammars: TYPESCRIPT_GRAMMARS,
    facts: TYPESCRIPT_FACTS,
    rules: TYPESCRIPT_RULES,
};

pub(crate) const VUE_MANIFEST: PluginManifest = PluginManifest {
    id: "flavor-plugin-vue",
    scopes: VUE_SCOPES,
    grammars: VUE_GRAMMARS,
    facts: VUE_FACTS,
    rules: VUE_RULES,
};

pub(crate) const SVELTE_MANIFEST: PluginManifest = PluginManifest {
    id: "flavor-plugin-svelte",
    scopes: SVELTE_SCOPES,
    grammars: SVELTE_GRAMMARS,
    facts: SVELTE_FACTS,
    rules: SVELTE_RULES,
};

const fn grammar(
    scope: ScopeKind,
    grammar_id: &'static str,
    entrypoint: &'static str,
) -> GrammarUse {
    GrammarUse {
        scope,
        grammar_id,
        entrypoint,
    }
}

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
