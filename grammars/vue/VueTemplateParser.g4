parser grammar VueTemplateParser;
options { tokenVocab=VueTemplateLexer; }

// G4-facing source of truth; flavor-specific facts live in flavor.g4.json.
template_document: child* EOF;
child: TEXT | element | interpolation;
element: start_tag child* end_tag?;
start_tag: TAG_OPEN tag_name attribute_or_directive* TAG_CLOSE;
interpolation: MUSTACHE_OPEN expression_text MUSTACHE_CLOSE;
directive: DIRECTIVE_NAME directive_argument? directive_modifier* directive_expression?;
directive_expression: ATTRIBUTE_VALUE | expression_text;
