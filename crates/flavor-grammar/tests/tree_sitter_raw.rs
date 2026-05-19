#![cfg(feature = "tree-sitter-backend")]

use flavor_core::SourceText;
use flavor_grammar::{
    parse_tree_sitter, GrammarSpec, GrammarTree, RawAstSchema, TreeSitterParseConfig,
    TreeSitterRawAstAdapter,
};
use tree_sitter::Parser;

const PARSER: &str = r#"
parser grammar TestParser;
source_file: item* EOF;
item: function_item;
function_item: IDENTIFIER;
"#;

const LEXER: &str = r#"
lexer grammar TestLexer;
KEYWORD_FN: 'fn';
IDENTIFIER: [a-zA-Z_]+; // tree-sitter:identifier
WS: [ \t\r\n]+;
RAW_TEXT: .;
"#;

const METADATA: &str = r#"
{
  "bundle": "test",
  "grammars": [{
    "id": "test",
    "sources": {
      "lexer": "TestLexer.g4",
      "parser": "TestParser.g4"
    },
    "directives": {
      "entry": "source_file",
      "owner": "tests"
    },
    "sections": {
      "nodes": {
        "function_item": "tree-sitter:function_item"
      },
      "facts": {
        "sample": "source_file -> SampleFact -> Fact(span)"
      },
      "diagnostics": {
        "sample": "ERROR -> sample/error"
      },
      "spans": {
        "sample": "sample"
      },
      "recovery": {
        "sample": "sample"
      }
    }
  }]
}
"#;

#[test]
fn adapter_builds_tree() {
    let source = SourceText::new("sample.rs", "fn demo() {}\n");
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("rust tree-sitter grammar");
    let tree = parser.parse(source.as_str(), None).expect("tree");
    let syntax = adapter().build(tree.root_node(), &source);
    let grammar = GrammarTree::new(syntax, schema());
    let function = grammar.root().child("function_item").expect("function");
    let name = function.token("IDENTIFIER").expect("identifier");

    assert_eq!(name.text(), "demo");
    assert_eq!(function.source_text(&source), "fn demo() {}");
}

#[test]
fn adapter_error_tree() {
    let source = SourceText::new("sample.rs", "not rust");
    let syntax = adapter().build_error(&source);
    let grammar = GrammarTree::new(syntax, schema());
    let root = grammar.root();

    assert!(root.is("source_file"));
    assert_eq!(root.token("RAW_TEXT").expect("raw").text(), "not rust");
}

#[test]
fn parser_builds_output() {
    let source = SourceText::new("sample.rs", "fn demo() {}\n");
    let output = parse_tree_sitter(
        &spec().bundle().expect("valid bundle"),
        tree_sitter_rust::LANGUAGE.into(),
        source.clone(),
        parse_config(),
    )
    .expect("valid parser");
    let grammar = GrammarTree::new(output.syntax, schema());
    let function = grammar.root().child("function_item").expect("function");

    assert!(output.diagnostics.is_empty());
    assert_eq!(output.source.name(), "sample.rs");
    assert_eq!(
        function.token("IDENTIFIER").expect("identifier").text(),
        "demo"
    );
}

fn adapter() -> TreeSitterRawAstAdapter {
    let bundle = spec().bundle().expect("valid bundle");
    TreeSitterRawAstAdapter::new(&bundle, "tree-sitter", "source_file", "WS", "RAW_TEXT")
        .expect("valid adapter")
}

fn parse_config() -> TreeSitterParseConfig<'static> {
    TreeSitterParseConfig {
        backend: "tree-sitter",
        root_kind: "source_file",
        whitespace_kind: "WS",
        fallback_kind: "RAW_TEXT",
        failure_code: "test/parse/error",
        failure_message: "failed to parse test source",
        error_code: "test/parse/error",
        error_message: "invalid test syntax",
    }
}

fn schema() -> RawAstSchema {
    spec().schema().expect("valid schema")
}

fn spec() -> GrammarSpec<'static> {
    GrammarSpec::new("test", 10, &[PARSER, LEXER], METADATA)
}
