#![allow(dead_code)]

use std::sync::OnceLock;

use flavor_grammar::{GrammarBundle, GrammarSpec, RawAstSchema};

pub type Kind = &'static str;

pub const TEMPLATE_DOCUMENT: Kind = "template_document";
pub const ROOT: Kind = "root";
pub const CHILD: Kind = "child";
pub const ELEMENT: Kind = "element";
pub const START_TAG: Kind = "start_tag";
pub const END_TAG: Kind = "end_tag";
pub const INTERPOLATION: Kind = "interpolation";
pub const DIRECTIVE: Kind = "directive";
pub const DIRECTIVE_NAME: Kind = "directive_name";
pub const DIRECTIVE_EXPRESSION: Kind = "directive_expression";
pub const COMMENT: Kind = "comment";
pub const ATTRIBUTE_OR_DIRECTIVE: Kind = "attribute_or_directive";
pub const ATTRIBUTE: Kind = "attribute";
pub const COMMENT_TEXT: Kind = "COMMENT_TEXT";
pub const INTERPOLATION_OPEN: Kind = "INTERPOLATION_OPEN";
pub const INTERPOLATION_CLOSE: Kind = "INTERPOLATION_CLOSE";
pub const LESS_THAN: Kind = "LESS_THAN";
pub const GREATER_THAN: Kind = "GREATER_THAN";
pub const SLASH: Kind = "SLASH";
pub const EQUALS: Kind = "EQUALS";
pub const DIRECTIVE_BASE: Kind = "DIRECTIVE_BASE";
pub const DIRECTIVE_ARGUMENT: Kind = "DIRECTIVE_ARGUMENT";
pub const DIRECTIVE_MODIFIER: Kind = "DIRECTIVE_MODIFIER";
pub const TAG_NAME: Kind = "TAG_NAME";
pub const ATTRIBUTE_NAME: Kind = "ATTRIBUTE_NAME";
pub const ATTRIBUTE_VALUE: Kind = "ATTRIBUTE_VALUE";
pub const EXPRESSION_TEXT: Kind = "EXPRESSION_TEXT";
pub const TEXT: Kind = "TEXT";
pub const WHITESPACE: Kind = "WHITESPACE";
pub const ERROR: Kind = "ERROR";

const SOURCES: &[&str] = &[
    include_str!("../../../../grammars/vue/VueTemplateParser.g4"),
    include_str!("../../../../grammars/vue/VueTemplateLexer.g4"),
];

pub const SPEC: GrammarSpec<'static> = GrammarSpec::new(
    "vue-template",
    1000,
    SOURCES,
    include_str!("../../../../grammars/vue/metadata.json"),
);

pub fn bundle() -> &'static GrammarBundle {
    static BUNDLE: OnceLock<GrammarBundle> = OnceLock::new();
    BUNDLE.get_or_init(|| SPEC.bundle().expect("valid Vue template grammar bundle"))
}

pub fn schema() -> &'static RawAstSchema {
    bundle().schema()
}
