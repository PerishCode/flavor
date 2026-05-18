lexer grammar RustLexer;

// Generated from flavor.g4.json metadata; parser runtime is transitional.
IDENTIFIER: [a-zA-Z_] [a-zA-Z0-9_]*; // tree-sitter:identifier
ATTRIBUTE: .; // tree-sitter:attribute_item
INNER_ATTRIBUTE: .; // tree-sitter:inner_attribute_item
KEYWORD_FN: 'fn';
KEYWORD_IMPL: 'impl';
KEYWORD_TRAIT: 'trait';
KEYWORD_LET: 'let';
KEYWORD_MATCH: 'match';
WS: [ \t\r\n]+ -> channel(HIDDEN);
