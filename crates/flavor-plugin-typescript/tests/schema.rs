use flavor_core::{RawSyntaxKind, SourceText};
use flavor_plugin_typescript::{run, SourceMode, TsPluginConfig};

#[path = "../src/internal/grammar.rs"]
mod kind;

fn is_core_trivia(kind: RawSyntaxKind) -> bool {
    matches!(kind.0, 1..=4)
}

#[test]
fn cst_matches_schema() {
    let config = TsPluginConfig {
        source_mode: SourceMode::Tsx,
        ..Default::default()
    };
    let output = run(
        SourceText::new(
            "sample.tsx",
            "// leading\nexport const node = <Panel title={name}>ok</Panel>;",
        ),
        config,
    );
    let schema = kind::schema();

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
    for token in output.tokens {
        assert!(
            schema.is_token_name(token.kind) || token.kind == kind::END_OF_FILE,
            "scanner kind {:?} is not declared as a G4 token",
            token.kind
        );
    }
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}
