use flavor_core::{RawSyntaxKind, SourceText};
use flavor_plugin_rust::{run, syntax_kind::RustSyntaxKind, RustPluginConfig};

fn is_core_trivia(kind: RawSyntaxKind) -> bool {
    matches!(kind.0, 1..=4)
}

fn has_node(output: &flavor_plugin_rust::RustAnalysisOutput, kind: RustSyntaxKind) -> bool {
    output
        .syntax
        .descendants()
        .any(|node| node.kind() == RawSyntaxKind::from(kind))
}

#[test]
fn cst_matches_schema() {
    let source = r#"
#[cfg(test)]
fn route_value(input: i32) {
    let local_value = input;
    match local_value {
        1 => local_value,
        _ => 0,
    };
}
"#;
    let output = run(
        SourceText::new("sample.rs", source),
        RustPluginConfig::default(),
    );

    assert_eq!(output.syntax.text().to_string(), source);
    assert!(has_node(&output, RustSyntaxKind::FunctionItem));
    assert!(has_node(&output, RustSyntaxKind::LetDeclaration));
    assert!(has_node(&output, RustSyntaxKind::MatchExpression));
    assert!(has_node(&output, RustSyntaxKind::MatchArm));
    for node in output.syntax.descendants() {
        assert!(
            RustSyntaxKind::raw_is_node(node.kind()),
            "node kind {:?} is not declared as a G4 node",
            node.kind()
        );
    }
    for token in output
        .syntax
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        assert!(
            RustSyntaxKind::raw_is_token(token.kind()) || is_core_trivia(token.kind()),
            "token kind {:?} is not declared as a G4 token",
            token.kind()
        );
    }
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}
