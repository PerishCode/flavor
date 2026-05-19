use flavor_core::SourceText;
use flavor_plugin_typescript::{run, SourceMode, TsAnalysisOutput, TsPluginConfig};

#[path = "../src/internal/grammar.rs"]
mod kind;

use kind::Kind;

struct Case {
    name: &'static str,
    source: &'static str,
    mode: SourceMode,
    nodes: &'static [Kind],
}

#[test]
fn parses_harness_cases() {
    for case in cases() {
        let config = TsPluginConfig {
            source_mode: case.mode,
            ..Default::default()
        };
        let output = run(SourceText::new(case.name, case.source), config);

        assert!(
            output.diagnostics.is_empty(),
            "{} diagnostics: {:?}",
            case.name,
            output.diagnostics
        );
        for kind in case.nodes {
            assert!(has_node(&output, kind), "{} missing {:?}", case.name, kind);
        }
    }
}

fn cases() -> &'static [Case] {
    &[
        Case {
            name: "modern-class.ts",
            source: include_str!("../harness/cases/modern-class.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                kind::EXPORT_DECLARATION,
                kind::CLASS_DECLARATION,
                kind::MODIFIER_LIST,
                kind::PROPERTY_DECLARATION,
                kind::METHOD_DECLARATION,
                kind::RETURN_STATEMENT,
                kind::NEW_EXPRESSION,
            ],
        },
        Case {
            name: "component.tsx",
            source: include_str!("../harness/cases/component.tsx"),
            mode: SourceMode::Tsx,
            nodes: &[
                kind::JSX_ELEMENT,
                kind::JSX_ATTRIBUTE,
                kind::JSX_SPREAD_ATTRIBUTE,
                kind::JSX_TEXT,
                kind::JSX_EXPRESSION,
            ],
        },
        Case {
            name: "bindings.ts",
            source: include_str!("../harness/cases/bindings.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                kind::FUNCTION_DECLARATION,
                kind::OBJECT_BINDING_PATTERN,
                kind::ARRAY_BINDING_PATTERN,
                kind::BINDING_ELEMENT,
                kind::REST_ELEMENT,
                kind::VARIABLE_DECLARATION,
            ],
        },
        Case {
            name: "control.ts",
            source: include_str!("../harness/cases/control.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                kind::FUNCTION_DECLARATION,
                kind::FOR_STATEMENT,
                kind::IF_STATEMENT,
                kind::SWITCH_STATEMENT,
                kind::SWITCH_CASE,
                kind::TRY_STATEMENT,
                kind::CATCH_CLAUSE,
                kind::CATCH_BINDING,
                kind::OBJECT_BINDING_PATTERN,
                kind::FINALLY_CLAUSE,
            ],
        },
        Case {
            name: "types.ts",
            source: include_str!("../harness/cases/types.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                kind::IMPORT_DECLARATION,
                kind::IMPORT_EQUALS_DECLARATION,
                kind::EXTERNAL_MODULE_REFERENCE,
                kind::NAMED_IMPORTS,
                kind::EXPORT_CLAUSE,
                kind::ENUM_DECLARATION,
                kind::NAMESPACE_DECLARATION,
                kind::INTERFACE_DECLARATION,
                kind::METHOD_SIGNATURE,
                kind::TYPE_ALIAS_DECLARATION,
                kind::UNION_TYPE,
                kind::OBJECT_TYPE,
                kind::TYPE_MEMBER,
            ],
        },
        Case {
            name: "literals.ts",
            source: include_str!("../harness/cases/literals.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                kind::EXPORT_DECLARATION,
                kind::VARIABLE_STATEMENT,
                kind::VARIABLE_DECLARATION,
                kind::BINARY_EXPRESSION,
            ],
        },
        Case {
            name: "operators.ts",
            source: include_str!("../harness/cases/operators.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                kind::EXPORT_DECLARATION,
                kind::VARIABLE_STATEMENT,
                kind::BINARY_EXPRESSION,
                kind::UNARY_EXPRESSION,
                kind::MEMBER_EXPRESSION,
                kind::ELEMENT_ACCESS_EXPRESSION,
            ],
        },
        Case {
            name: "type-operators.ts",
            source: include_str!("../harness/cases/type-operators.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                kind::TYPE_OPERATOR,
                kind::INDEXED_ACCESS_TYPE,
                kind::MAPPED_TYPE,
                kind::CONDITIONAL_TYPE,
                kind::TYPE_MEMBER,
            ],
        },
    ]
}

fn has_node(output: &TsAnalysisOutput, kind: Kind) -> bool {
    output
        .syntax
        .descendants()
        .any(|node| node.kind() == kind::schema().raw_kind(kind))
}
