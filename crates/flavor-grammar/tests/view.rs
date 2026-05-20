use flavor_core::SourceText;
use flavor_grammar::{
    GrammarTree, RawAstBuilder, RawAstSchema, RawAstSymbol, RawAstSymbolKind, TokenTextRun,
};

#[test]
fn string_kind_view() {
    let schema = schema();
    let mut builder = RawAstBuilder::new(&schema);
    builder.start_node("root");
    builder.start_node("item");
    builder.token("WS", "  ");
    builder.token("NAME", "alpha");
    builder.finish_node();
    builder.finish_node();

    let source = SourceText::new("sample", "  alpha");
    let tree = GrammarTree::new(builder.finish(), schema);
    let root = tree.root();
    assert!(root.is("root"));
    let item = root.child("item").expect("item");
    assert_eq!(item.kind_name(), Some("item"));
    assert_eq!(item.source_text(&source), "  alpha");
    assert_eq!(item.trimmed_source_text(&source), "alpha");

    let name = item.token("NAME").expect("name token");
    assert_eq!(name.text(), "alpha");
    assert_eq!(name.line(&source), 1);
    assert_eq!(tree.find("item").count(), 1);
}

#[test]
fn atomic_view_helpers() {
    let schema = schema();
    let mut builder = RawAstBuilder::new(&schema);
    builder.start_node("root");
    builder.token("WS", " ");
    builder.token("PREFIX", "<");
    builder.token("NAME", "Foo");
    builder.token("DOT", ".");
    builder.token("NAME", "Bar");
    builder.start_node("item");
    builder.token("NAME", "child");
    builder.finish_node();
    builder.finish_node();

    let tree = GrammarTree::new(builder.finish(), schema);
    let root = tree.root();

    assert!(root.is_any(&["root", "other"]));
    assert_eq!(
        root.child_any(&["missing", "item"])
            .expect("item")
            .kind_name(),
        Some("item")
    );
    assert_eq!(root.child_token_text("PREFIX").as_deref(), Some("<"));
    assert_eq!(root.child_token_text_any(&["NAME"]).as_deref(), Some("Foo"));
    assert_eq!(root.token_text("NAME").as_deref(), Some("Foo"));
    assert_eq!(
        root.last_token_text_any(&["NAME"]).as_deref(),
        Some("child")
    );
    assert_eq!(root.head_token_text_any(&["NAME"]).as_deref(), Some("Foo"));
    assert!(root.has_token("PREFIX"));
    assert_eq!(
        root.token_run_text(TokenTextRun::new(&["NAME"], &["DOT"]).with_skip(&["PREFIX"]))
            .as_deref(),
        Some("Foo.Bar")
    );
}

fn schema() -> RawAstSchema {
    RawAstSchema::new(
        "test",
        vec![
            symbol("root", RawAstSymbolKind::Node, 10),
            symbol("item", RawAstSymbolKind::Node, 11),
            symbol("NAME", RawAstSymbolKind::Token, 12),
            symbol("WS", RawAstSymbolKind::Token, 13),
            symbol("DOT", RawAstSymbolKind::Token, 14),
            symbol("PREFIX", RawAstSymbolKind::Token, 15),
        ],
    )
}

fn symbol(name: &str, kind: RawAstSymbolKind, raw_kind: u16) -> RawAstSymbol {
    RawAstSymbol {
        name: name.to_string(),
        kind,
        raw_kind,
    }
}
