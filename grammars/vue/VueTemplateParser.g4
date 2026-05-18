parser grammar VueTemplateParser;
options { tokenVocab=VueTemplateLexer; }

// Hand-written G4 parser surface; flavor contracts live in metadata.json.
template_document: root;
root: child* EOF;
child: TEXT | element | interpolation | comment;
element: start_tag child* end_tag?;
start_tag: LESS_THAN TAG_NAME attribute_or_directive* SLASH? GREATER_THAN;
end_tag: LESS_THAN SLASH TAG_NAME GREATER_THAN;
interpolation: INTERPOLATION_OPEN EXPRESSION_TEXT? INTERPOLATION_CLOSE;
directive: directive_name directive_expression?;
directive_name: DIRECTIVE_BASE DIRECTIVE_ARGUMENT? DIRECTIVE_MODIFIER*;
directive_expression: ATTRIBUTE_VALUE | EXPRESSION_TEXT;
comment: COMMENT_TEXT;
attribute_or_directive: attribute | directive;
attribute: ATTRIBUTE_NAME ATTRIBUTE_VALUE?;
