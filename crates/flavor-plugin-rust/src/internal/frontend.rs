use flavor_core::SourceText;
use flavor_grammar::{parse_tree_sitter, GrammarParseOutput, TreeSitterParseConfig};

use crate::internal::grammar;

pub(crate) fn parse(source: SourceText) -> GrammarParseOutput {
    parse_tree_sitter(
        &grammar::bundle(),
        tree_sitter_rust::LANGUAGE.into(),
        source,
        TreeSitterParseConfig {
            backend: "tree-sitter",
            root_kind: grammar::SOURCE_FILE,
            whitespace_kind: grammar::WS,
            fallback_kind: grammar::RAW_TEXT,
            failure_code: "rust/parse/error",
            failure_message: "failed to parse Rust source",
            error_code: "rust/parse/error",
            error_message: "invalid Rust syntax",
        },
    )
    .expect("valid Rust tree-sitter grammar parser")
}
