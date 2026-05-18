parser grammar TsxParser;
options { tokenVocab=TypeScriptLexer; }

// Hand-written G4 parser surface; flavor contracts live in metadata.json.
program: statement* EOF;
statement: import_statement | jsx_element | jsx_self_closing_element | tsx_token;
import_statement: KEYWORD_IMPORT import_clause? from_clause?;
import_clause: type_marker? (default_import | namespace_import | named_imports)+;
from_clause: STRING_LITERAL;
type_marker: IDENTIFIER;
default_import: IDENTIFIER;
namespace_import: IDENTIFIER;
named_imports: import_specifier+;
import_specifier: IDENTIFIER;
jsx_element: jsx_opening_element jsx_child* jsx_closing_element;
jsx_self_closing_element: LESS_THAN jsx_name jsx_attribute* JSX_SELF_CLOSE;
jsx_opening_element: LESS_THAN jsx_name jsx_attribute* GREATER_THAN;
jsx_name: identifier | member_expression | jsx_namespace_name;
jsx_child: jsx_text | jsx_expression | jsx_element | jsx_self_closing_element;
jsx_closing_element: LESS_THAN jsx_name GREATER_THAN;
jsx_attribute: JSX_IDENTIFIER;
jsx_expression: tsx_token+;
identifier: JSX_IDENTIFIER | IDENTIFIER;
member_expression: identifier identifier+;
jsx_namespace_name: JSX_NAMESPACE | JSX_IDENTIFIER;
jsx_text: JSX_TEXT_TOKEN;
tsx_token: IDENTIFIER | JSX_IDENTIFIER | JSX_NAMESPACE | JSX_TEXT_TOKEN | STRING_LITERAL | KEYWORD_IMPORT | KEYWORD_FUNCTION | KEYWORD_CASE | KEYWORD_EXPORT | KEYWORD_CONST | KEYWORD_LET | KEYWORD_SWITCH;
