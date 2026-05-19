use flavor_core::{RawSyntaxKind, SourceText};
use flavor_plugin_rust::{run, RustAnalysisOutput, RustPluginConfig};

#[path = "../src/internal/grammar.rs"]
mod grammar;

fn is_core_trivia(kind: RawSyntaxKind) -> bool {
    matches!(kind.0, 1..=4)
}

fn has_node(output: &RustAnalysisOutput, kind: grammar::Kind) -> bool {
    let schema = grammar::schema();
    output
        .syntax
        .descendants()
        .any(|node| node.kind() == schema.raw_kind(kind))
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
    let schema = grammar::schema();

    assert_eq!(output.syntax.text().to_string(), source);
    assert!(has_node(&output, grammar::FUNCTION_ITEM));
    assert!(has_node(&output, grammar::LET_DECLARATION));
    assert!(has_node(&output, grammar::MATCH_EXPRESSION));
    assert!(has_node(&output, grammar::MATCH_ARM));
    for node in output.syntax.descendants() {
        assert!(
            schema.raw_is_node(node.kind()),
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
            schema.raw_is_token(token.kind()) || is_core_trivia(token.kind()),
            "token kind {:?} is not declared as a G4 token",
            token.kind()
        );
    }
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}
