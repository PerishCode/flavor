use flavor_g4::{
    parse_contract, parse_contract_values, GrammarContractExpectation,
    GrammarEntryValueExpectation, GrammarSectionExpectation,
};
use flavor_plugin_core::SourceText;
use flavor_plugin_typescript::{run, SourceMode, TsImportSpecifier, TsNameKind, TsPluginConfig};

const TYPESCRIPT_GRAMMAR: &str = include_str!("../../../grammars/typescript/flavor.g4.json");
const TSX_GRAMMAR: &str = include_str!("../../../grammars/typescript/flavor.g4.json");
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
            entries: &["legacy.cst", "tree_sitter.errors", "tree_sitter.missing"],
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
            entries: &[
                "rule.continuation",
                "tree_sitter.errors",
                "tree_sitter.missing",
            ],
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
    parse_contract_values(TYPESCRIPT_GRAMMAR, &TYPESCRIPT_CONTRACT, TYPESCRIPT_VALUES).unwrap();
}

#[test]
fn typescript_contract_facts() {
    parse_contract(TYPESCRIPT_GRAMMAR, &TYPESCRIPT_CONTRACT).unwrap();
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
        .source_file
        .source()
        .slice(type_import.span)
        .starts_with("import type"));
    assert_eq!(type_import.line, 1);

    let render = find_name(&output, TsNameKind::Function, "render", 3).expect("render fact");
    assert_eq!(output.source_file.source().slice(render.span), "render");
    let save = find_name(&output, TsNameKind::Method, "save", 11).expect("save fact");
    assert_eq!(output.source_file.source().slice(save.span), "save");
    let local = find_name(&output, TsNameKind::Binding, "local", 4).expect("local fact");
    assert_eq!(output.source_file.source().slice(local.span), "local");
    assert!(find_name(&output, TsNameKind::Parameter, "input", 3).is_some());
    assert!(find_name(&output, TsNameKind::Parameter, "next", 11).is_some());
    let ready_branch = output
        .facts
        .dispatch_branches
        .iter()
        .find(|branch| branch.line == 6 && branch.lines == 1)
        .expect("ready branch fact");
    assert!(output
        .source_file
        .source()
        .slice(ready_branch.span)
        .starts_with("case 'ready'"));
}

#[test]
fn typescript_contract_diagnostics() {
    parse_contract(TYPESCRIPT_GRAMMAR, &TYPESCRIPT_CONTRACT).unwrap();

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
fn tsx_contract_sections() {
    parse_contract_values(TSX_GRAMMAR, &TSX_CONTRACT, TSX_VALUES).unwrap();
}

#[test]
fn tsx_contract_facts() {
    parse_contract(TSX_GRAMMAR, &TSX_CONTRACT).unwrap();
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
    assert!(output
        .source_file
        .source()
        .slice(panel.span)
        .starts_with("<Panel.Root>"));
    let button = output
        .facts
        .jsx_elements
        .iter()
        .find(|element| element.intrinsic.as_deref() == Some("button"))
        .expect("button JSX element");
    assert!(button.self_closing);
    assert_eq!(output.source_file.source().slice(button.span), "<button />");
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
