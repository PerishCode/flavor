use flavor_core::SourceText;
use flavor_grammar::{
    parse_contract, parse_contract_values, GrammarContractExpectation,
    GrammarEntryValueExpectation, GrammarSectionExpectation,
};
use flavor_plugin_typescript::{
    run, SourceMode, TsFailureMechanism, TsFailureSurfaceConfig, TsImportSpecifier, TsNameKind,
    TsPluginConfig, TsRawFailureKind, TsStructuredFailureKind,
};

const TYPESCRIPT_METADATA: &str = include_str!("../../../grammars/typescript/metadata.json");
const TSX_METADATA: &str = include_str!("../../../grammars/typescript/metadata.json");
const TYPESCRIPT_CONTRACT: GrammarContractExpectation<'static> = GrammarContractExpectation {
    name: "typescript",
    directives: &[
        ("owner", "crates/flavor-plugin-typescript"),
        ("entry", "program"),
    ],
    sections: &[
        GrammarSectionExpectation {
            name: "facts",
            entries: &[
                "dispatch.branch",
                "error.raw_failure",
                "error.structured_failure",
                "module.counts",
                "module.import",
                "name.binding",
                "name.function",
                "name.method",
                "name.parameter",
            ],
        },
        GrammarSectionExpectation {
            name: "diagnostics",
            entries: &["embedded.expression", "parse.error", "parse.missing"],
        },
        GrammarSectionExpectation {
            name: "spans",
            entries: &[
                "diagnostic.range",
                "node.line",
                "node.range",
                "script.offset",
            ],
        },
        GrammarSectionExpectation {
            name: "recovery",
            entries: &["parser.errors", "raw_ast.recovery"],
        },
    ],
};
const TYPESCRIPT_VALUES: &[GrammarEntryValueExpectation<'static>] = &[
    GrammarEntryValueExpectation {
        section: "facts",
        key: "module.import",
        contains: &[
            "TsImportFact",
            "payload.source",
            "payload.type_only",
            "payload.named_imports",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "name.function",
        contains: &[
            "TsNameFact",
            "payload.name",
            "payload.issue_kind",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "name.method",
        contains: &[
            "TsNameFact",
            "payload.name",
            "payload.issue_kind",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "name.binding",
        contains: &[
            "TsNameFact",
            "payload.name",
            "payload.issue_kind",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "name.parameter",
        contains: &[
            "TsNameFact",
            "payload.name",
            "payload.issue_kind",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "dispatch.branch",
        contains: &["TsDispatchBranchFact", "payload.lines", "span", "line"],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "error.raw_failure",
        contains: &[
            "TsRawFailureFact",
            "payload.kind",
            "payload.mechanism",
            "payload.constructor",
            "payload.callee",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "error.structured_failure",
        contains: &[
            "TsStructuredFailureFact",
            "payload.kind",
            "payload.mechanism",
            "payload.callee",
            "span",
            "line",
        ],
    },
];
const TSX_CONTRACT: GrammarContractExpectation<'static> = GrammarContractExpectation {
    name: "tsx",
    directives: &[
        ("owner", "crates/flavor-plugin-typescript"),
        ("entry", "program"),
    ],
    sections: &[
        GrammarSectionExpectation {
            name: "facts",
            entries: &[
                "jsx.element",
                "jsx.intrinsic",
                "jsx.root",
                "jsx.self_closing",
                "primitive.imports",
                "primitive.usage",
            ],
        },
        GrammarSectionExpectation {
            name: "diagnostics",
            entries: &["parse.error", "parse.missing"],
        },
        GrammarSectionExpectation {
            name: "spans",
            entries: &["element.line", "element.range", "script.offset"],
        },
        GrammarSectionExpectation {
            name: "recovery",
            entries: &["parser.errors", "raw_ast.recovery", "rule.continuation"],
        },
    ],
};
const TSX_VALUES: &[GrammarEntryValueExpectation<'static>] = &[
    GrammarEntryValueExpectation {
        section: "facts",
        key: "jsx.element",
        contains: &[
            "TsxElementFact",
            "payload.name",
            "payload.intrinsic",
            "payload.self_closing",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "jsx.self_closing",
        contains: &[
            "TsxElementFact",
            "payload.name",
            "payload.intrinsic",
            "payload.self_closing",
            "span",
            "line",
        ],
    },
];

#[test]
fn typescript_contract_sections() {
    parse_contract_values(TYPESCRIPT_METADATA, &TYPESCRIPT_CONTRACT, TYPESCRIPT_VALUES).unwrap();
}

#[test]
fn typescript_contract_facts() {
    parse_contract(TYPESCRIPT_METADATA, &TYPESCRIPT_CONTRACT).unwrap();
    let output = run(
        SourceText::new(
            "contract.ts",
            "import type { Ref } from './types';\n\
             import value, * as api from './api';\n\
             export function render(input: string) {\n\
                 const local = input;\n\
                 switch (input) {\n\
                     case 'ready': return local;\n\
                     default: return api.fallback;\n\
                 }\n\
             }\n\
             class Panel {\n\
                 save(next: string) { return next; }\n\
             }\n",
        ),
        TsPluginConfig::default(),
    );

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert_eq!(output.facts.import_count, 2);
    assert_eq!(output.facts.export_count, 1);
    assert!(output.facts.imports.iter().any(|import| {
        import.source == "./types"
            && import.type_only
            && import
                .specifiers
                .contains(&TsImportSpecifier::Named("Ref".to_string()))
    }));
    assert!(output.facts.imports.iter().any(|import| {
        import.source == "./api"
            && import
                .specifiers
                .contains(&TsImportSpecifier::Default("value".to_string()))
            && import
                .specifiers
                .contains(&TsImportSpecifier::Namespace("api".to_string()))
    }));
    let type_import = output
        .facts
        .imports
        .iter()
        .find(|import| import.source == "./types")
        .expect("type import fact");
    assert!(output
        .source
        .slice(type_import.span)
        .starts_with("import type"));
    assert_eq!(type_import.line, 1);

    let render = find_name(&output, TsNameKind::Function, "render", 3).expect("render fact");
    assert_eq!(output.source.slice(render.span), "render");
    let save = find_name(&output, TsNameKind::Method, "save", 11).expect("save fact");
    assert_eq!(output.source.slice(save.span), "save");
    let local = find_name(&output, TsNameKind::Binding, "local", 4).expect("local fact");
    assert_eq!(output.source.slice(local.span), "local");
    assert!(find_name(&output, TsNameKind::Parameter, "input", 3).is_some());
    assert!(find_name(&output, TsNameKind::Parameter, "next", 11).is_some());
    let ready_branch = output
        .facts
        .dispatch_branches
        .iter()
        .find(|branch| branch.line == 6 && branch.lines == 1)
        .expect("ready branch fact");
    assert!(output
        .source
        .slice(ready_branch.span)
        .starts_with("case 'ready'"));
}

#[test]
fn typescript_contract_diagnostics() {
    parse_contract(TYPESCRIPT_METADATA, &TYPESCRIPT_CONTRACT).unwrap();

    let output = run(
        SourceText::new("broken.ts", "class Broken"),
        TsPluginConfig::default(),
    );
    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message == "expected class body")
        .expect("missing class body diagnostic");
    assert_eq!(diagnostic.code.as_deref(), Some("ts/parse/error"));
    let span = diagnostic.span.expect("diagnostic span");
    assert!(span.start <= span.end);
    assert!(span.end as usize <= "class Broken".len());
}

#[test]
fn typescript_failure_surface_facts() {
    parse_contract(TYPESCRIPT_METADATA, &TYPESCRIPT_CONTRACT).unwrap();

    let output = run(
        SourceText::new(
            "failure.ts",
            "function load(input: string) {\n\
             ensure(input);\n\
             throw new Error('missing');\n\
             }\n\
             function later(input: string) {\n\
             return Promise.reject(new TypeError(input));\n\
             }\n\
             function callback(reject: (error: Error) => void) {\n\
             reject(new RangeError('bad'));\n\
             }\n\
             function custom(input: string) {\n\
             return DomainError.missing(input);\n\
             }\n\
             function stable(input: string) {\n\
             throw new DomainError(input);\n\
             }\n",
        ),
        TsPluginConfig {
            failure_surface: TsFailureSurfaceConfig {
                structured_guards: vec!["ensure".to_string()],
                structured_factories: vec!["DomainError".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
    );

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert_eq!(output.facts.raw_failures.len(), 3);
    let raw = &output.facts.raw_failures[0];
    assert_eq!(raw.kind, TsRawFailureKind::BuiltinError);
    assert_eq!(raw.mechanism, TsFailureMechanism::Throw);
    assert_eq!(raw.constructor.as_deref(), Some("Error"));
    assert_eq!(raw.callee, None);
    assert_eq!(raw.line, 3);
    assert!(output.source.slice(raw.span).starts_with("throw new Error"));
    assert!(output.facts.raw_failures.iter().any(|fact| {
        fact.mechanism == TsFailureMechanism::Call
            && fact.constructor.as_deref() == Some("TypeError")
            && fact.callee.as_deref() == Some("Promise.reject")
            && fact.line == 6
    }));
    assert!(output.facts.raw_failures.iter().any(|fact| {
        fact.mechanism == TsFailureMechanism::Call
            && fact.constructor.as_deref() == Some("RangeError")
            && fact.callee.as_deref() == Some("reject")
            && fact.line == 9
    }));

    assert!(output.facts.structured_failures.iter().any(|fact| {
        fact.kind == TsStructuredFailureKind::Guard
            && fact.mechanism == TsFailureMechanism::Call
            && fact.callee == "ensure"
            && fact.line == 2
    }));
    assert!(output.facts.structured_failures.iter().any(|fact| {
        fact.kind == TsStructuredFailureKind::Factory
            && fact.mechanism == TsFailureMechanism::Call
            && fact.callee == "DomainError.missing"
            && fact.line == 12
    }));
    assert!(output.facts.structured_failures.iter().any(|fact| {
        fact.kind == TsStructuredFailureKind::Factory
            && fact.mechanism == TsFailureMechanism::ThrowNew
            && fact.callee == "DomainError"
            && fact.line == 15
    }));
}

#[test]
fn tsx_contract_sections() {
    parse_contract_values(TSX_METADATA, &TSX_CONTRACT, TSX_VALUES).unwrap();
}

#[test]
fn tsx_contract_facts() {
    parse_contract(TSX_METADATA, &TSX_CONTRACT).unwrap();
    let config = TsPluginConfig {
        source_mode: SourceMode::Tsx,
        ..Default::default()
    };
    let output = run(
        SourceText::new(
            "contract.tsx",
            "import { Panel } from './ui';\n\
             const node = <Panel.Root><button /></Panel.Root>;\n",
        ),
        config,
    );

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert!(output
        .facts
        .imports
        .iter()
        .any(|import| import.source == "./ui"
            && import
                .specifiers
                .contains(&TsImportSpecifier::Named("Panel".to_string()))));
    let panel = output
        .facts
        .jsx_elements
        .iter()
        .find(|element| element.name == "Panel.Root" && element.root.as_deref() == Some("Panel"))
        .expect("Panel.Root JSX element");
    assert!(output.source.slice(panel.span).starts_with("<Panel.Root>"));
    let button = output
        .facts
        .jsx_elements
        .iter()
        .find(|element| element.intrinsic.as_deref() == Some("button"))
        .expect("button JSX element");
    assert!(button.self_closing);
    assert_eq!(output.source.slice(button.span), "<button />");
}

fn find_name<'a>(
    output: &'a flavor_plugin_typescript::TsAnalysisOutput,
    kind: TsNameKind,
    name: &str,
    line: usize,
) -> Option<&'a flavor_plugin_typescript::TsNameFact> {
    output
        .facts
        .names
        .iter()
        .find(|fact| fact.kind == kind && fact.name == name && fact.line == line)
}
