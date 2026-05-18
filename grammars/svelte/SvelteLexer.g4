lexer grammar SvelteLexer;

// Hand-written G4 lexer surface; flavor contracts live in metadata.json.
TAG_OPEN: '<';
TAG_CLOSE: '>';
END_TAG_OPEN: '</';
ATTRIBUTE_NAME: [a-zA-Z_] [a-zA-Z0-9_]*; // scanner:attribute_name
ATTRIBUTE_VALUE: ~[<>\r\n]+; // scanner:attribute_value
RAW_TEXT: ~[<\r\n]+; // scanner:block_content
WS: [ \t\r\n]+ -> channel(HIDDEN);
