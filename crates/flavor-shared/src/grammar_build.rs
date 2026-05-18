use std::{
    env, fs,
    path::{Path, PathBuf},
};

use flavor_grammar::{parse_g4_source_validated, RawAstSchema};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SyntaxEnumOptions<'a> {
    pub grammar_dir: &'a str,
    pub lexer: &'a str,
    pub parser: &'a str,
    pub grammar_id: &'a str,
    pub raw_kind_start: u16,
    pub enum_name: &'a str,
    pub raw_kind_path: &'a str,
    pub fallback_kind: &'a str,
    pub output_file: &'a str,
}

pub fn write_workspace_syntax_enum(options: SyntaxEnumOptions<'_>) {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let grammar_dir = manifest_dir
        .join("../../grammars")
        .join(options.grammar_dir);
    let lexer = grammar_dir.join(options.lexer);
    let parser = grammar_dir.join(options.parser);
    rerun_if_changed(&lexer);
    rerun_if_changed(&parser);

    let parser_source = fs::read_to_string(&parser).unwrap();
    let lexer_source = fs::read_to_string(&lexer).unwrap();
    let parser = parse_g4_source_validated(&parser_source).unwrap();
    let lexer = parse_g4_source_validated(&lexer_source).unwrap();
    let schema =
        RawAstSchema::from_g4_sources(options.grammar_id, options.raw_kind_start, &[parser, lexer])
            .unwrap();
    let generated = schema
        .render_rust_enum_fallback(
            options.enum_name,
            options.raw_kind_path,
            Some(options.fallback_kind),
        )
        .unwrap();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join(options.output_file), generated).unwrap();
}

pub fn rerun_if_changed(path: &Path) {
    println!("cargo:rerun-if-changed={}", path.display());
}
