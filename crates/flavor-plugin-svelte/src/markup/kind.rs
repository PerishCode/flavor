#![allow(dead_code)]

use std::sync::OnceLock;

use flavor_grammar::{GrammarBundle, GrammarSpec, RawAstSchema};

pub type Kind = &'static str;

pub const MARKUP_DOCUMENT: Kind = "markup_document";
pub const ROOT: Kind = "root";
pub const CHILD: Kind = "child";
pub const ELEMENT: Kind = "element";
pub const COMPONENT: Kind = "component";
pub const MUSTACHE: Kind = "mustache";
pub const BLOCK: Kind = "block";
pub const BLOCK_OPEN: Kind = "block_open";
pub const BLOCK_BRANCH: Kind = "block_branch";
pub const BLOCK_CLOSE: Kind = "block_close";
pub const DIRECTIVE: Kind = "directive";
pub const DIRECTIVE_NAME: Kind = "directive_name";
pub const START_TAG: Kind = "start_tag";
pub const END_TAG: Kind = "end_tag";
pub const COMMENT: Kind = "comment";
pub const ATTRIBUTE: Kind = "attribute";
pub const DIRECTIVE_EXPRESSION: Kind = "directive_expression";
pub const RENDER_TAG: Kind = "render_tag";
pub const SPECIAL_TAG: Kind = "special_tag";
pub const SPREAD_ATTRIBUTE: Kind = "spread_attribute";
pub const SHORTHAND_ATTRIBUTE: Kind = "shorthand_attribute";
pub const COMMENT_TEXT: Kind = "COMMENT_TEXT";
pub const MUSTACHE_OPEN: Kind = "MUSTACHE_OPEN";
pub const MUSTACHE_CLOSE: Kind = "MUSTACHE_CLOSE";
pub const LESS_THAN: Kind = "LESS_THAN";
pub const GREATER_THAN: Kind = "GREATER_THAN";
pub const SLASH: Kind = "SLASH";
pub const EQUALS: Kind = "EQUALS";
pub const DIRECTIVE_BASE: Kind = "DIRECTIVE_BASE";
pub const DIRECTIVE_ARGUMENT: Kind = "DIRECTIVE_ARGUMENT";
pub const DIRECTIVE_MODIFIER: Kind = "DIRECTIVE_MODIFIER";
pub const TAG_NAME: Kind = "TAG_NAME";
pub const ATTRIBUTE_NAME: Kind = "ATTRIBUTE_NAME";
pub const BLOCK_KEYWORD: Kind = "BLOCK_KEYWORD";
pub const EXPRESSION_TEXT: Kind = "EXPRESSION_TEXT";
pub const ATTRIBUTE_VALUE: Kind = "ATTRIBUTE_VALUE";
pub const TEXT: Kind = "TEXT";
pub const WHITESPACE: Kind = "WHITESPACE";
pub const ERROR: Kind = "ERROR";

const SOURCES: &[&str] = &[
    include_str!("../../../../grammars/svelte/SvelteMarkupParser.g4"),
    include_str!("../../../../grammars/svelte/SvelteMarkupLexer.g4"),
];

pub const SPEC: GrammarSpec<'static> = GrammarSpec::new(
    "svelte-markup",
    2000,
    SOURCES,
    include_str!("../../../../grammars/svelte/metadata.json"),
);

pub fn bundle() -> &'static GrammarBundle {
    static BUNDLE: OnceLock<GrammarBundle> = OnceLock::new();
    BUNDLE.get_or_init(|| SPEC.bundle().expect("valid Svelte markup grammar bundle"))
}

pub fn schema() -> &'static RawAstSchema {
    bundle().schema()
}
