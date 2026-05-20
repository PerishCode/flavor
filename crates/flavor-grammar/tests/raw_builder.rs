use flavor_core::{SourceText, Span, Token, Trivia, TriviaKind};
use flavor_grammar::{GrammarTree, RawAstBuilder, RawAstSchema, RawAstSymbol, RawAstSymbolKind};

#[test]
fn source_token_preserves_trivia() {
    let source = SourceText::new("sample", "  alpha");
    let schema = schema();
    let mut token = Token::new("NAME", Span::new(2, 7));
    token
        .leading
        .push(Trivia::new(TriviaKind::Whitespace, Span::new(0, 2)));

    let mut builder = RawAstBuilder::new(&schema);
    builder.start_node("root");
    builder.source_token(&source, &token);
    builder.finish_node();

    let tree = GrammarTree::new(builder.finish(), schema);
    let root = tree.root();
    let name = root.token("NAME").expect("name");

    assert_eq!(root.source_text(&source), "  alpha");
    assert_eq!(name.text(), "alpha");
    assert_eq!(name.source_text(&source), "alpha");
}

fn schema() -> RawAstSchema {
    RawAstSchema::new(
        "test",
        vec![
            RawAstSymbol {
                name: "root".to_string(),
                kind: RawAstSymbolKind::Node,
                raw_kind: 10,
            },
            RawAstSymbol {
                name: "NAME".to_string(),
                kind: RawAstSymbolKind::Token,
                raw_kind: 11,
            },
        ],
    )
}
