parser grammar PythonParser;
options { tokenVocab=PythonLexer; }

// Contract grammar for Python code-shape facts. Runtime parsing is staged in
// the Python plugin while raw AST names stay governed by this source.
program: source_file;
source_file: statement* end_of_file;
end_of_file: EOF;
statement:
    decorator
    | class_definition
    | function_definition
    | async_function_definition
    | assignment_statement
    | branch_statement
    | simple_statement
    | unknown_statement
    ;
decorator: AT dotted_name call_suffix? NEWLINE?;
class_definition: KEYWORD_CLASS IDENTIFIER base_list? COLON block;
function_definition: KEYWORD_DEF IDENTIFIER parameter_list return_annotation? COLON block;
async_function_definition: KEYWORD_ASYNC KEYWORD_DEF IDENTIFIER parameter_list return_annotation? COLON block;
parameter_list: OPEN_PAREN parameter? (COMMA parameter)* COMMA? CLOSE_PAREN;
parameter: STAR? STAR? IDENTIFIER annotation? default_value?;
annotation: COLON expression;
default_value: EQUALS expression;
return_annotation: ARROW expression;
assignment_statement: IDENTIFIER EQUALS expression NEWLINE?;
branch_statement:
    if_statement
    | elif_clause
    | else_clause
    | for_statement
    | while_statement
    | try_statement
    | except_clause
    | finally_clause
    | with_statement
    | match_statement
    | case_clause
    ;
if_statement: KEYWORD_IF expression COLON block;
elif_clause: KEYWORD_ELIF expression COLON block;
else_clause: KEYWORD_ELSE COLON block;
for_statement: KEYWORD_FOR expression KEYWORD_IN expression COLON block;
while_statement: KEYWORD_WHILE expression COLON block;
try_statement: KEYWORD_TRY COLON block;
except_clause: KEYWORD_EXCEPT expression? COLON block;
finally_clause: KEYWORD_FINALLY COLON block;
with_statement: KEYWORD_WITH expression COLON block;
match_statement: KEYWORD_MATCH expression COLON block;
case_clause: KEYWORD_CASE expression COLON block;
simple_statement: python_token+ NEWLINE?;
unknown_statement: python_token+ NEWLINE?;
block: INDENT statement* DEDENT;
base_list: OPEN_PAREN expression? CLOSE_PAREN;
call_suffix: OPEN_PAREN expression? CLOSE_PAREN;
dotted_name: IDENTIFIER (DOT IDENTIFIER)*;
expression: python_token+;
python_token:
    KEYWORD_AWAIT
    | KEYWORD_IMPORT
    | KEYWORD_FROM
    | KEYWORD_LAMBDA
    | OPEN_PAREN
    | CLOSE_PAREN
    | OPEN_BRACKET
    | CLOSE_BRACKET
    | OPEN_BRACE
    | CLOSE_BRACE
    | COLON
    | COMMA
    | DOT
    | STAR
    | DOUBLE_STAR
    | EQUALS
    | ARROW
    | IDENTIFIER
    | STRING_LITERAL
    | NUMBER_LITERAL
    | RAW_TEXT
    | UNKNOWN
    ;
