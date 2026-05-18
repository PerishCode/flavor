use flavor_core::{RawSyntaxKind, Span, SyntaxBuilder, SyntaxKindSchema, SyntaxSpanExt};

#[derive(Clone, Copy)]
enum TestKind {
    Node = 10,
    Token = 11,
}

impl From<TestKind> for RawSyntaxKind {
    fn from(kind: TestKind) -> Self {
        Self(kind as u16)
    }
}

impl SyntaxKindSchema for TestKind {
    fn raw_is_node(kind: RawSyntaxKind) -> bool {
        kind == RawSyntaxKind::from(Self::Node)
    }

    fn raw_is_token(kind: RawSyntaxKind) -> bool {
        kind == RawSyntaxKind::from(Self::Token)
    }
}

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

#[test]
fn schema_builder_methods() {
    let mut builder = SyntaxBuilder::new();
    builder.start_schema_node(TestKind::Node);
    builder.schema_token(TestKind::Token, "abc");
    builder.finish_node();
    let root = builder.finish();

    assert_eq!(root.kind(), RawSyntaxKind::from(TestKind::Node));
    assert_eq!(root.text().to_string(), "abc");
}

#[test]
#[should_panic(expected = "schema kind must be a node")]
fn schema_node_rejects_token() {
    let mut builder = SyntaxBuilder::new();
    builder.start_schema_node(TestKind::Token);
}

#[test]
#[should_panic(expected = "schema kind must be a token")]
fn schema_token_rejects_node() {
    let mut builder = SyntaxBuilder::new();
    builder.schema_token(TestKind::Node, "abc");
}
