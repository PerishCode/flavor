use flavor_core::GrammarProduct;

#[test]
fn plugin_exposes_g4_facts() {
    let mut products = Vec::<GrammarProduct>::new();
    flavor_plugin_g4::plugin::satisfy(
        &|grammar_id| (grammar_id == "g4").then_some("grammar_file"),
        "SampleLexer.g4",
        r#"
lexer grammar SampleLexer;
IDENTIFIER: [a-zA-Z_]+;
"#,
        &mut products,
    );

    let product = products.first().expect("g4 product");
    assert_eq!(product.grammar_id, "g4");
    assert!(product.diagnostics.is_empty());
    assert!(product.facts.iter().any(|fact| {
        fact.key == "grammar.declaration" && fact.text("name") == Some("SampleLexer")
    }));
    assert!(product
        .facts
        .iter()
        .any(|fact| fact.key == "grammar.lexer_token" && fact.text("name") == Some("IDENTIFIER")));
}

#[test]
fn plugin_exposes_parse_diagnostics() {
    let mut products = Vec::<GrammarProduct>::new();
    flavor_plugin_g4::plugin::satisfy(
        &|grammar_id| (grammar_id == "g4").then_some("grammar_file"),
        "SampleParser.g4",
        "parser grammar SampleParser;\nsource_file: missing_rule EOF;\n",
        &mut products,
    );

    let diagnostic = products
        .first()
        .and_then(|product| product.diagnostics.first())
        .expect("g4 diagnostic");
    assert!(diagnostic.message.contains("missing_rule"));
    assert_eq!(diagnostic.line, Some(2));
}
