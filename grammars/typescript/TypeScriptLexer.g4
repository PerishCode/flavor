lexer grammar TypeScriptLexer;

// Generated from flavor.g4.json metadata; parser runtime is transitional.
JSX_IDENTIFIER: [a-zA-Z_] [a-zA-Z0-9_]*; // tree-sitter:identifier
JSX_NAMESPACE: [a-zA-Z_] [a-zA-Z0-9_]*; // tree-sitter:jsx_namespace_name
JSX_TEXT: ~[\r\n]*; // tree-sitter:jsx_text
JSX_OPEN: '<';
JSX_CLOSE: '>';
JSX_SELF_CLOSE: '/>';
WS: [ \t\r\n]+ -> channel(HIDDEN);
