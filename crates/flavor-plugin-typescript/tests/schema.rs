use flavor_core::{RawSyntaxKind, SourceText};
use flavor_plugin_typescript::{run, syntax_kind::TsSyntaxKind, SourceMode, TsPluginConfig};

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

    for node in output.source_file.syntax().descendants() {
        assert!(
            TsSyntaxKind::raw_is_node(node.kind()),
            "node kind {:?} is not declared as a G4 node",
            node.kind()
        );
    }
    for token in output
        .source_file
        .syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        assert!(
            TsSyntaxKind::raw_is_token(token.kind()) || is_core_trivia(token.kind()),
            "token kind {:?} is not declared as a G4 token",
            token.kind()
        );
    }
    for token in output.source_file.tokens() {
        assert!(
            token.kind.is_token() || token.kind == TsSyntaxKind::EndOfFile,
            "scanner kind {:?} is not declared as a G4 token",
            token.kind
        );
    }
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}
