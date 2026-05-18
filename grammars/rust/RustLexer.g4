lexer grammar RustLexer;

// Hand-written G4 lexer surface; flavor contracts live in metadata.json.
KEYWORD_FN: 'fn';
KEYWORD_IMPL: 'impl';
KEYWORD_TRAIT: 'trait';
KEYWORD_LET: 'let';
KEYWORD_MATCH: 'match';
IDENTIFIER: [a-zA-Z_] [a-zA-Z0-9_]*; // tree-sitter:identifier
INNER_ATTRIBUTE: '#![' .*? ']'; // tree-sitter:inner_attribute_item
ATTRIBUTE: '#[' .*? ']'; // tree-sitter:attribute_item
WS: [ \t\r\n]+ -> channel(HIDDEN);
RAW_TEXT: .;
