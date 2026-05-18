lexer grammar SvelteMarkupLexer;

// Generated from flavor.g4.json metadata; parser runtime is transitional.
TEXT: ~[\r\n]*; // scanner:text
TAG_OPEN: '<';
TAG_CLOSE: '>';
END_TAG_OPEN: '</';
MUSTACHE_OPEN: '{';
MUSTACHE_CLOSE: '}';
BLOCK_OPEN: '{#';
BLOCK_BRANCH: '{:';
BLOCK_CLOSE: '{/';
DIRECTIVE_NAME: [a-zA-Z_] [a-zA-Z0-9_]*; // scanner:directive_name
ATTRIBUTE_VALUE: ~[\r\n]*; // scanner:attribute_value
WS: [ \t\r\n]+ -> channel(HIDDEN);
