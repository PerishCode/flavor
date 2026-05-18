lexer grammar VueTemplateLexer;

// Hand-written G4 lexer surface; flavor contracts live in metadata.json.
COMMENT_TEXT: '<!--' .*? '-->';
INTERPOLATION_OPEN: '{{';
INTERPOLATION_CLOSE: '}}';
LESS_THAN: '<';
GREATER_THAN: '>';
SLASH: '/';
EQUALS: '=';
DIRECTIVE_BASE: [a-zA-Z_] [a-zA-Z0-9_]*; // scanner:directive_base
DIRECTIVE_ARGUMENT: [:@#] [a-zA-Z0-9_.$[\]-]*; // scanner:directive_argument
DIRECTIVE_MODIFIER: '.' [a-zA-Z0-9_.$[\]-]+; // scanner:directive_modifier
TAG_NAME: [a-zA-Z_] [a-zA-Z0-9_]*; // scanner:tag_name
ATTRIBUTE_NAME: [a-zA-Z_] [a-zA-Z0-9_]*; // scanner:attribute_name
ATTRIBUTE_VALUE: ~[<>{}\r\n]+; // scanner:attribute_value
EXPRESSION_TEXT: ~[<>{}\r\n]+; // scanner:expression_text
TEXT: ~[<{\r\n]+; // scanner:text
WHITESPACE: [ \t\r\n]+ -> channel(HIDDEN);
ERROR: .;
