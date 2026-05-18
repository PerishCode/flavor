parser grammar SvelteMarkupParser;
options { tokenVocab=SvelteMarkupLexer; }

// Hand-written G4 parser surface; flavor contracts live in metadata.json.
markup_document: root;
root: child* EOF;
child: TEXT | element | component | mustache | block | render_tag | special_tag | comment;
element: start_tag child* end_tag?;
component: start_tag child* end_tag?;
mustache: MUSTACHE_OPEN EXPRESSION_TEXT? MUSTACHE_CLOSE;
block: block_open child* block_branch* block_close?;
block_open: MUSTACHE_OPEN BLOCK_KEYWORD EXPRESSION_TEXT?;
block_branch: MUSTACHE_OPEN BLOCK_KEYWORD? EXPRESSION_TEXT? child*;
block_close: MUSTACHE_OPEN BLOCK_KEYWORD? MUSTACHE_CLOSE?;
directive: directive_name directive_expression?;
directive_name: DIRECTIVE_BASE DIRECTIVE_ARGUMENT? DIRECTIVE_MODIFIER*;
start_tag: LESS_THAN TAG_NAME (attribute | directive | spread_attribute | shorthand_attribute)* SLASH? GREATER_THAN;
end_tag: LESS_THAN SLASH TAG_NAME GREATER_THAN;
comment: COMMENT_TEXT;
attribute: ATTRIBUTE_NAME (EQUALS ATTRIBUTE_VALUE)?;
directive_expression: ATTRIBUTE_VALUE | EXPRESSION_TEXT;
render_tag: MUSTACHE_OPEN BLOCK_KEYWORD EXPRESSION_TEXT? MUSTACHE_CLOSE;
special_tag: MUSTACHE_OPEN BLOCK_KEYWORD EXPRESSION_TEXT? MUSTACHE_CLOSE | LESS_THAN TAG_NAME GREATER_THAN;
spread_attribute: MUSTACHE_OPEN EXPRESSION_TEXT? MUSTACHE_CLOSE;
shorthand_attribute: mustache;
