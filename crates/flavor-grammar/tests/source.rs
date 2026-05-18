use std::{
    fs,
    path::{Path, PathBuf},
};

use flavor_grammar::{
    parse_g4_source_validated, parse_metadata_validated, validate_metadata_source_shape, G4Source,
    GrammarMetadata, RawAstSchema, RawAstSymbolKind,
};

#[test]
fn parses_g4_source_shape() {
    let source = r#"
parser grammar SampleParser;
options { tokenVocab=SampleLexer; }

source: item* EOF;
item: name value?;
name: IDENTIFIER;
value: STRING;
"#;

    let parsed = parse_g4_source_validated(source).unwrap();
    assert_eq!(parsed.name, "SampleParser");
    assert!(parsed.defines_parser_rule("source"));
    assert!(parsed.defines_parser_rule("item"));
    assert!(!parsed.defines_parser_rule("IDENTIFIER"));
}

#[test]
fn parses_g4_rule_bindings() {
    let source = r#"
lexer grammar SampleLexer;
IDENTIFIER: [a-zA-Z_]+; // tree-sitter:identifier
STRING_LITERAL: '"' ~["]* '"';
"#;

    let parsed = parse_g4_source_validated(source).unwrap();
    let identifier = parsed
        .lexer_tokens
        .iter()
        .find(|rule| rule.name == "IDENTIFIER")
        .unwrap();
    assert_eq!(identifier.body, "[a-zA-Z_]+");
    assert_eq!(identifier.bindings[0].backend, "tree-sitter");
    assert_eq!(identifier.bindings[0].name, "identifier");
    let string = parsed
        .lexer_tokens
        .iter()
        .find(|rule| rule.name == "STRING_LITERAL")
        .unwrap();
    assert!(string.bindings.is_empty());
}

#[test]
fn bad_g4_shape_rejected() {
    let source = r#"
parser grammar SampleParser;

source: missing_rule EOF;
source: EOF;
"#;

    let errors = parse_g4_source_validated(source).unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("missing_rule")));
    assert!(errors
        .iter()
        .any(|error| error.message.contains("duplicate parser rule `source`")));
}

#[test]
fn derives_raw_ast_schema() {
    let lexer = parse_g4_source_validated(
        r#"
lexer grammar SampleLexer;
IDENTIFIER: [a-zA-Z_]+;
STRING_LITERAL: '"' ~["]* '"';
"#,
    )
    .unwrap();
    let parser = parse_g4_source_validated(
        r#"
parser grammar SampleParser;
source_file: item* EOF;
item: IDENTIFIER STRING_LITERAL?;
"#,
    )
    .unwrap();

    let schema = RawAstSchema::from_g4_sources("sample", 100, &[parser, lexer]).unwrap();
    assert_eq!(schema.grammar_id, "sample");
    assert_eq!(
        schema.symbol("source_file").map(|symbol| (
            symbol.variant.as_str(),
            symbol.kind,
            symbol.raw_kind
        )),
        Some(("SourceFile", RawAstSymbolKind::Node, 100))
    );
    assert_eq!(
        schema.symbol("STRING_LITERAL").map(|symbol| (
            symbol.variant.as_str(),
            symbol.kind,
            symbol.raw_kind
        )),
        Some(("StringLiteral", RawAstSymbolKind::Token, 103))
    );
    let rendered = schema
        .render_rust_enum("SampleSyntaxKind", "flavor_core::RawSyntaxKind")
        .unwrap();
    assert!(rendered.contains("pub enum SampleSyntaxKind"));
    assert!(rendered.contains("SourceFile = 100"));
    assert!(rendered.contains("StringLiteral = 103"));
    assert!(rendered.contains("impl From<SampleSyntaxKind> for flavor_core::RawSyntaxKind"));
    assert!(rendered.contains("impl flavor_core::SyntaxKindSchema for SampleSyntaxKind"));
    assert!(rendered.contains("pub fn is_node"));
    assert!(rendered.contains("pub fn is_token"));
    assert!(rendered.contains("pub fn raw_is_node"));
    assert!(rendered.contains("matches!(kind.0, 100..=101)"));
    assert!(rendered.contains("matches!(kind.0, 102..=103)"));
    let rendered_with_fallback = schema
        .render_rust_enum_fallback(
            "SampleSyntaxKind",
            "flavor_core::RawSyntaxKind",
            Some("Item"),
        )
        .unwrap();
    assert!(rendered_with_fallback.contains("pub fn from_raw"));
    assert!(rendered_with_fallback.contains("_ => Self::Item"));
}

#[test]
fn renders_rust_adapters() {
    let lexer = parse_g4_source_validated(
        r#"
lexer grammar SampleLexer;
KEYWORD_FN: 'fn';
IDENTIFIER: [a-zA-Z_]+; // tree-sitter:identifier
WS: [ \t]+;
RAW_TEXT: .;
"#,
    )
    .unwrap();
    let parser = parse_g4_source_validated(
        r#"
parser grammar SampleParser;
source_file: IDENTIFIER EOF;
"#,
    )
    .unwrap();
    let metadata = parse_metadata_validated(&sample_metadata_with_sections(
        Some("crates/sample"),
        serde_json::json!({
            "nodes": {
                "source_file": "tree-sitter:source_file"
            },
            "facts": {
                "name": "source_file -> SampleName"
            },
            "diagnostics": {
                "parse": "ERROR -> sample/parse"
            },
            "spans": {
                "node": "byte range"
            },
            "recovery": {
                "error": "skip"
            }
        }),
    ))
    .unwrap()
    .remove(0);
    let sources = [parser, lexer];
    let schema = RawAstSchema::from_g4_sources("sample", 100, &sources).unwrap();

    let nodes = schema
        .render_rust_node_adapter("SampleKind", "node_kind", &metadata, "tree-sitter")
        .unwrap();
    assert!(nodes.contains(r#""source_file" => Some(SampleKind::SourceFile)"#));

    let tokens = schema
        .render_rust_token_adapter("SampleKind", "token_kind", &sources, "tree-sitter")
        .unwrap();
    assert!(tokens.contains(r#""identifier" => Some(SampleKind::Identifier)"#));

    let gaps = schema
        .render_rust_gap_adapter("SampleKind", "gap_kind", &sources, "WS", "RAW_TEXT")
        .unwrap();
    assert!(gaps.contains(r#""fn" => SampleKind::KeywordFn"#));
    assert!(gaps.contains("_ => SampleKind::RawText"));
}

#[test]
fn raw_ast_drift_rejected() {
    let first = parse_g4_source_validated(
        r#"
parser grammar FirstParser;
source: EOF;
"#,
    )
    .unwrap();
    let second = parse_g4_source_validated(
        r#"
parser grammar SecondParser;
source: EOF;
"#,
    )
    .unwrap();

    let errors = RawAstSchema::from_g4_sources("sample", u16::MAX, &[first, second]).unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("duplicate raw AST symbol `source`")));
    assert!(errors
        .iter()
        .any(|error| error.message.contains("overflowed u16")));

    let parser = parse_g4_source_validated(
        r#"
parser grammar SampleParser;
foo_bar: EOF;
"#,
    )
    .unwrap();
    let lexer = parse_g4_source_validated(
        r#"
lexer grammar SampleLexer;
FOO_BAR: 'x';
"#,
    )
    .unwrap();
    let schema = RawAstSchema::from_g4_sources("sample", 100, &[parser, lexer]).unwrap();
    assert_eq!(
        schema
            .symbol("foo_bar")
            .map(|symbol| symbol.variant.as_str()),
        Some("FooBarNode")
    );
    assert_eq!(
        schema
            .symbol("FOO_BAR")
            .map(|symbol| symbol.variant.as_str()),
        Some("FooBarToken")
    );
}

#[test]
fn metadata_shape_drift_rejected() {
    let source = sample_metadata_with_sections(
        Some("crates/sample"),
        serde_json::json!({
            "nodes": {
                "missing": "SampleMissing"
            },
            "facts": {
                "name": "source -> SampleName"
            },
            "diagnostics": {
                "parse": "ERROR -> sample/parse"
            },
            "spans": {
                "node": "byte range"
            },
            "recovery": {
                "error": "skip"
            }
        }),
    );
    let document = parse_metadata_validated(&source).unwrap().remove(0);
    let g4 = parse_g4_source_validated(
        r#"
parser grammar SampleParser;
source: IDENTIFIER EOF;
"#,
    )
    .unwrap();
    let errors = validate_metadata_source_shape(&document, &[g4]);
    assert!(errors
        .iter()
        .any(|error| error.message.contains("metadata node `missing`")));
}

#[test]
fn metadata_nodes_reference_symbols() {
    let root = grammar_root();
    for path in grammar_files(&root) {
        if path.file_name().and_then(|value| value.to_str()) != Some("metadata.json") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        let documents = parse_metadata_validated(&source).unwrap_or_else(|errors| {
            panic!("{} parse errors: {errors:?}", path.display());
        });
        for document in &documents {
            let sources = parse_g4_sources(&path, document);
            let errors = validate_metadata_source_shape(document, &sources);
            assert!(
                errors.is_empty(),
                "{} source shape errors: {errors:?}",
                path.display()
            );
        }
    }
}

#[test]
fn metadata_derives_raw_schemas() {
    let root = grammar_root();
    for path in grammar_files(&root) {
        if path.file_name().and_then(|value| value.to_str()) != Some("metadata.json") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        let documents = parse_metadata_validated(&source).unwrap_or_else(|errors| {
            panic!("{} parse errors: {errors:?}", path.display());
        });
        for document in &documents {
            let sources = parse_g4_sources(&path, document);
            let schema = RawAstSchema::from_g4_sources(&document.name, 100, &sources)
                .unwrap_or_else(|errors| {
                    panic!("{} schema errors: {errors:?}", path.display());
                });
            assert!(
                !schema.symbols.is_empty(),
                "{} should derive raw AST symbols for {}",
                path.display(),
                document.name
            );
            if let Some(nodes) = document.section("nodes") {
                for node in &nodes.entries {
                    assert!(
                        schema.symbol(&node.key).is_some(),
                        "{} schema should include metadata node {}.{}",
                        path.display(),
                        document.name,
                        node.key
                    );
                }
            }
        }
    }
}

fn grammar_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("grammars")
}

fn grammar_files(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_grammar_files(root, &mut paths);
    paths
}

fn collect_grammar_files(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_grammar_files(&path, paths);
        } else {
            paths.push(path);
        }
    }
}

fn parse_g4_sources(metadata_path: &Path, document: &GrammarMetadata) -> Vec<G4Source> {
    document
        .sources
        .iter()
        .map(|source| {
            let source_path = metadata_path.parent().unwrap().join(source);
            let source = fs::read_to_string(&source_path).unwrap();
            parse_g4_source_validated(&source).unwrap_or_else(|errors| {
                panic!("{} G4 parse errors: {errors:?}", source_path.display());
            })
        })
        .collect()
}

fn sample_metadata_with_sections(owner: Option<&str>, sections: serde_json::Value) -> String {
    let mut directives = serde_json::Map::new();
    directives.insert("entry".to_string(), serde_json::json!("source"));
    if let Some(owner) = owner {
        directives.insert("owner".to_string(), serde_json::json!(owner));
    }
    serde_json::json!({
        "bundle": "sample",
        "grammars": [{
            "id": "sample",
            "sources": {
                "lexer": "SampleLexer.g4",
                "parser": "SampleParser.g4"
            },
            "directives": directives,
            "sections": sections
        }]
    })
    .to_string()
}
