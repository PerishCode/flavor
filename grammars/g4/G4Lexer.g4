lexer grammar G4Lexer;

LEXER_GRAMMAR: 'lexer' HSPACE+ 'grammar';
PARSER_GRAMMAR: 'parser' HSPACE+ 'grammar';
GRAMMAR: 'grammar';
OPTIONS: 'options';
LOWER_IDENTIFIER: [a-z_] [a-zA-Z0-9_]*;
UPPER_IDENTIFIER: [A-Z] [a-zA-Z0-9_]*;
STRING_LITERAL: '\'' (~['\\] | '\\' .)* '\'';
COLON: ':';
SEMI: ';';
LBRACE: '{';
RBRACE: '}';
LPAREN: '(';
RPAREN: ')';
PIPE: '|';
STAR: '*';
PLUS: '+';
QUESTION: '?';
ASSIGN: '=';
COMMENT: '//' ~[\r\n]* -> skip;
WS: [ \t\r\n]+ -> skip;
RAW_TEXT: .;

fragment HSPACE: [ \t];
