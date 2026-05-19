use flavor_core::{RawSyntaxKind, Span, SyntaxBuilder, SyntaxSpanExt};

#[test]
fn maps_syntax_spans() {
    let mut builder = SyntaxBuilder::new();
    builder.start_node(RawSyntaxKind(10));
    builder.token(RawSyntaxKind(11), "abc");
    builder.token(RawSyntaxKind(12), "\nxy");
    builder.finish_node();
    let root = builder.finish();

    assert_eq!(root.source_span(), Span::new(0, 6));

    let first_token = root
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .next()
        .expect("first token");
    assert_eq!(first_token.source_span(), Span::new(0, 3));
}
