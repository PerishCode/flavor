#![allow(dead_code)]

use std::sync::OnceLock;

use flavor_grammar::{GrammarBundle, GrammarSpec, RawAstSchema};

pub type Kind = &'static str;

pub const SOURCE_FILE: Kind = "source_file";
pub const RAW_TEXT: Kind = "RAW_TEXT";

const SOURCES: &[&str] = &[
    include_str!("../../../../grammars/python/PythonParser.g4"),
    include_str!("../../../../grammars/python/PythonLexer.g4"),
];

pub const SPEC: GrammarSpec<'static> = GrammarSpec::new(
    "python",
    7000,
    SOURCES,
    include_str!("../../../../grammars/python/metadata.json"),
);

pub fn bundle() -> &'static GrammarBundle {
    static BUNDLE: OnceLock<GrammarBundle> = OnceLock::new();
    BUNDLE.get_or_init(|| SPEC.bundle().expect("valid Python grammar bundle"))
}

pub fn schema() -> &'static RawAstSchema {
    bundle().schema()
}
