#![allow(dead_code)]

use flavor_grammar::{GrammarBundle, GrammarSpec, RawAstSchema};

pub type Kind = &'static str;

pub const SOURCE_FILE: Kind = "source_file";
pub const ITEM: Kind = "item";
pub const FUNCTION_ITEM: Kind = "function_item";
pub const FUNCTION_SIGNATURE_ITEM: Kind = "function_signature_item";
pub const IMPL_ITEM: Kind = "impl_item";
pub const TRAIT_ITEM: Kind = "trait_item";
pub const LET_DECLARATION: Kind = "let_declaration";
pub const MATCH_EXPRESSION: Kind = "match_expression";
pub const MATCH_ARM: Kind = "match_arm";
pub const MOD_ITEM: Kind = "mod_item";
pub const STATEMENT: Kind = "statement";
pub const PARAMETERS: Kind = "parameters";
pub const PARAMETER: Kind = "parameter";
pub const RETURN_TYPE: Kind = "return_type";
pub const BLOCK: Kind = "block";
pub const TRAIT_REF: Kind = "trait_ref";
pub const TYPE: Kind = "type";
pub const BODY: Kind = "body";
pub const PATTERN: Kind = "pattern";
pub const VALUE: Kind = "value";
pub const EXPRESSION: Kind = "expression";
pub const MATCH_BLOCK: Kind = "match_block";
pub const GUARD: Kind = "guard";
pub const ARM_VALUE: Kind = "arm_value";
pub const RUST_TOKEN: Kind = "rust_token";
pub const KEYWORD_FN: Kind = "KEYWORD_FN";
pub const KEYWORD_IMPL: Kind = "KEYWORD_IMPL";
pub const KEYWORD_TRAIT: Kind = "KEYWORD_TRAIT";
pub const KEYWORD_FOR: Kind = "KEYWORD_FOR";
pub const KEYWORD_LET: Kind = "KEYWORD_LET";
pub const KEYWORD_MATCH: Kind = "KEYWORD_MATCH";
pub const IDENTIFIER: Kind = "IDENTIFIER";
pub const INNER_ATTRIBUTE: Kind = "INNER_ATTRIBUTE";
pub const ATTRIBUTE: Kind = "ATTRIBUTE";
pub const WS: Kind = "WS";
pub const RAW_TEXT: Kind = "RAW_TEXT";

const SOURCES: &[&str] = &[
    include_str!("../../../../grammars/rust/RustParser.g4"),
    include_str!("../../../../grammars/rust/RustLexer.g4"),
];

pub const SPEC: GrammarSpec<'static> = GrammarSpec::new(
    "rust",
    3000,
    SOURCES,
    include_str!("../../../../grammars/rust/metadata.json"),
);

pub fn bundle() -> GrammarBundle {
    SPEC.bundle().expect("valid Rust grammar bundle")
}

pub fn schema() -> RawAstSchema {
    SPEC.schema().expect("valid Rust grammar schema")
}
