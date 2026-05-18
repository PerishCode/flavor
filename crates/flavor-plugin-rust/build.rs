use std::{env, fs, path::PathBuf};

use flavor_grammar::{
    parse_g4_source_validated, parse_metadata_validated, validate_metadata_source_shape,
    GrammarMetadata, RawAstSchema,
};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let grammar_dir = manifest_dir.join("../../grammars/rust");
    let lexer = grammar_dir.join("RustLexer.g4");
    let parser = grammar_dir.join("RustParser.g4");
    let metadata = grammar_dir.join("metadata.json");
    println!("cargo:rerun-if-changed={}", lexer.display());
    println!("cargo:rerun-if-changed={}", parser.display());
    println!("cargo:rerun-if-changed={}", metadata.display());

    let parser_source = fs::read_to_string(&parser).unwrap();
    let lexer_source = fs::read_to_string(&lexer).unwrap();
    let metadata_source = fs::read_to_string(&metadata).unwrap();
    let parser = parse_g4_source_validated(&parser_source).unwrap();
    let lexer = parse_g4_source_validated(&lexer_source).unwrap();
    let sources = [parser, lexer];
    let metadata = rust_metadata(&metadata_source);
    let errors = validate_metadata_source_shape(&metadata, &sources);
    assert!(errors.is_empty(), "Rust metadata source errors: {errors:?}");

    let schema = RawAstSchema::from_g4_sources("rust", 3000, &sources).unwrap();
    let generated = schema
        .render_rust_enum_fallback(
            "RustSyntaxKind",
            "flavor_core::RawSyntaxKind",
            Some("RawText"),
        )
        .unwrap();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("rust_syntax_kind.rs"), generated).unwrap();
    fs::write(
        out_dir.join("rust_tree_sitter_nodes.rs"),
        schema
            .render_rust_node_adapter("RustSyntaxKind", "node_kind", &metadata, "tree-sitter")
            .unwrap(),
    )
    .unwrap();
    fs::write(
        out_dir.join("rust_tree_sitter_tokens.rs"),
        schema
            .render_rust_token_adapter(
                "RustSyntaxKind",
                "token_kind_for_node",
                &sources,
                "tree-sitter",
            )
            .unwrap(),
    )
    .unwrap();
    fs::write(
        out_dir.join("rust_gap_kind.rs"),
        schema
            .render_rust_gap_adapter("RustSyntaxKind", "gap_kind", &sources, "WS", "RAW_TEXT")
            .unwrap(),
    )
    .unwrap();
}

fn rust_metadata(source: &str) -> GrammarMetadata {
    let documents = parse_metadata_validated(source).unwrap();
    documents
        .into_iter()
        .find(|document| document.name == "rust")
        .expect("rust metadata")
}
