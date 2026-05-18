parser grammar SvelteMarkupParser;
options { tokenVocab=SvelteMarkupLexer; }

// G4-facing source of truth; flavor-specific facts live in flavor.g4.json.
markup_document: child* EOF;
child: TEXT | element | component | mustache | block | render_tag | special_tag;
element: start_tag child* end_tag?;
component: uppercase start_tag child* end_tag?;
mustache: MUSTACHE_OPEN expression_text MUSTACHE_CLOSE;
block: BLOCK_OPEN block_keyword expression_text? child* BLOCK_BRANCH* BLOCK_CLOSE;
directive: DIRECTIVE_NAME directive_expression?;
