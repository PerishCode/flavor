use std::{env, fs, path::PathBuf};

use flavor_grammar::{parse_g4_source_validated, RawAstSchema};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let grammar_dir = manifest_dir.join("../../grammars/svelte");
    let lexer = grammar_dir.join("SvelteMarkupLexer.g4");
    let parser = grammar_dir.join("SvelteMarkupParser.g4");
    println!("cargo:rerun-if-changed={}", lexer.display());
    println!("cargo:rerun-if-changed={}", parser.display());

    let parser_source = fs::read_to_string(&parser).unwrap();
    let lexer_source = fs::read_to_string(&lexer).unwrap();
    let parser = parse_g4_source_validated(&parser_source).unwrap();
    let lexer = parse_g4_source_validated(&lexer_source).unwrap();
    let schema = RawAstSchema::from_g4_sources("svelte-markup", 2000, &[parser, lexer]).unwrap();
    let generated = schema
        .render_rust_enum_fallback(
            "SvelteMarkupKind",
            "flavor_core::RawSyntaxKind",
            Some("Error"),
        )
        .unwrap();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("svelte_markup_kind.rs"), generated).unwrap();
}
