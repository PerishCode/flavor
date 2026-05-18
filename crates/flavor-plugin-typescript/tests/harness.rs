use flavor_plugin_core::{RawSyntaxKind, SourceText};
use flavor_plugin_typescript::{run, syntax_kind::TsSyntaxKind, SourceMode, TsPluginConfig};

struct Case {
    name: &'static str,
    source: &'static str,
    mode: SourceMode,
    nodes: &'static [TsSyntaxKind],
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
            assert!(has_node(&output, *kind), "{} missing {:?}", case.name, kind);
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
                TsSyntaxKind::ExportDeclaration,
                TsSyntaxKind::ClassDeclaration,
                TsSyntaxKind::ModifierList,
                TsSyntaxKind::PropertyDeclaration,
                TsSyntaxKind::MethodDeclaration,
                TsSyntaxKind::ReturnStatement,
                TsSyntaxKind::NewExpression,
            ],
        },
        Case {
            name: "component.tsx",
            source: include_str!("../harness/cases/component.tsx"),
            mode: SourceMode::Tsx,
            nodes: &[
                TsSyntaxKind::JsxElement,
                TsSyntaxKind::JsxAttribute,
                TsSyntaxKind::JsxSpreadAttribute,
                TsSyntaxKind::JsxText,
                TsSyntaxKind::JsxExpression,
            ],
        },
        Case {
            name: "bindings.ts",
            source: include_str!("../harness/cases/bindings.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                TsSyntaxKind::FunctionDeclaration,
                TsSyntaxKind::ObjectBindingPattern,
                TsSyntaxKind::ArrayBindingPattern,
                TsSyntaxKind::BindingElement,
                TsSyntaxKind::RestElement,
                TsSyntaxKind::VariableDeclaration,
            ],
        },
        Case {
            name: "control.ts",
            source: include_str!("../harness/cases/control.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                TsSyntaxKind::FunctionDeclaration,
                TsSyntaxKind::ForStatement,
                TsSyntaxKind::IfStatement,
                TsSyntaxKind::SwitchStatement,
                TsSyntaxKind::SwitchCase,
                TsSyntaxKind::TryStatement,
                TsSyntaxKind::CatchClause,
                TsSyntaxKind::CatchBinding,
                TsSyntaxKind::ObjectBindingPattern,
                TsSyntaxKind::FinallyClause,
            ],
        },
        Case {
            name: "types.ts",
            source: include_str!("../harness/cases/types.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                TsSyntaxKind::ImportDeclaration,
                TsSyntaxKind::ImportEqualsDeclaration,
                TsSyntaxKind::ExternalModuleReference,
                TsSyntaxKind::NamedImports,
                TsSyntaxKind::ExportClause,
                TsSyntaxKind::EnumDeclaration,
                TsSyntaxKind::NamespaceDeclaration,
                TsSyntaxKind::InterfaceDeclaration,
                TsSyntaxKind::MethodSignature,
                TsSyntaxKind::TypeAliasDeclaration,
                TsSyntaxKind::UnionType,
                TsSyntaxKind::ObjectType,
                TsSyntaxKind::TypeMember,
            ],
        },
        Case {
            name: "literals.ts",
            source: include_str!("../harness/cases/literals.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                TsSyntaxKind::ExportDeclaration,
                TsSyntaxKind::VariableStatement,
                TsSyntaxKind::VariableDeclaration,
                TsSyntaxKind::BinaryExpression,
            ],
        },
        Case {
            name: "operators.ts",
            source: include_str!("../harness/cases/operators.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                TsSyntaxKind::ExportDeclaration,
                TsSyntaxKind::VariableStatement,
                TsSyntaxKind::BinaryExpression,
                TsSyntaxKind::UnaryExpression,
                TsSyntaxKind::MemberExpression,
                TsSyntaxKind::ElementAccessExpression,
            ],
        },
        Case {
            name: "type-operators.ts",
            source: include_str!("../harness/cases/type-operators.ts"),
            mode: SourceMode::TypeScript,
            nodes: &[
                TsSyntaxKind::TypeOperator,
                TsSyntaxKind::IndexedAccessType,
                TsSyntaxKind::MappedType,
                TsSyntaxKind::ConditionalType,
                TsSyntaxKind::TypeMember,
            ],
        },
    ]
}

fn has_node(output: &flavor_plugin_typescript::TsAnalysisOutput, kind: TsSyntaxKind) -> bool {
    output
        .source_file
        .syntax()
        .descendants()
        .any(|node| node.kind() == RawSyntaxKind::from(kind))
}
