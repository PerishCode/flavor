parser grammar TypeScriptParser;
options { tokenVocab=TypeScriptLexer; }

// Hand-written G4 parser surface; flavor contracts live in metadata.json.
program: source_file;
source_file: statement* end_of_file;
end_of_file: EOF;
statement:
    import_statement
    | export_statement
    | declaration
    | return_statement
    | if_statement
    | for_statement
    | while_statement
    | do_statement
    | switch_statement
    | try_statement
    | throw_statement
    | break_statement
    | continue_statement
    | expression_statement
    | unknown_statement
    ;
import_statement: import_declaration;
import_declaration: KEYWORD_IMPORT import_clause? from_clause? SEMICOLON?;
import_clause: type_marker? (default_import | namespace_import | named_imports)+;
import_equals_declaration: KEYWORD_IMPORT IDENTIFIER EQUALS external_module_reference SEMICOLON?;
external_module_reference: IDENTIFIER OPEN_PAREN STRING_LITERAL CLOSE_PAREN;
export_statement: export_declaration;
export_declaration: KEYWORD_EXPORT statement?;
export_clause: STAR | named_exports;
named_exports: OPEN_BRACE export_specifier (COMMA export_specifier)* COMMA? CLOSE_BRACE;
export_specifier: IDENTIFIER (KEYWORD_AS IDENTIFIER)?;
export_assignment: KEYWORD_EXPORT KEYWORD_DEFAULT expression SEMICOLON?;
namespace_export_declaration: KEYWORD_EXPORT KEYWORD_AS KEYWORD_NAMESPACE IDENTIFIER SEMICOLON?;
declaration:
    function_declaration
    | function_expression
    | class_declaration
    | interface_declaration
    | type_alias_declaration
    | enum_declaration
    | namespace_declaration
    | variable_statement
    | method_definition
    | method_declaration
    | method_signature
    | property_declaration
    | property_signature
    | variable_declarator
    ;
function_declaration: KEYWORD_FUNCTION IDENTIFIER? type_parameters? parameter_list return_type? body;
function_expression: KEYWORD_FUNCTION IDENTIFIER? type_parameters? parameter_list return_type? body;
method_definition: property_name parameter_list return_type? body;
method_declaration: property_name parameter_list return_type? body;
method_signature: property_name parameter_list return_type? SEMICOLON?;
property_declaration: property_name type_annotation? initializer? SEMICOLON?;
property_signature: property_name type_annotation? SEMICOLON?;
variable_statement: (KEYWORD_CONST | KEYWORD_LET) variable_declaration_list SEMICOLON?;
variable_declaration_list: variable_declaration (COMMA variable_declaration)*;
variable_declaration: variable_declarator;
variable_declarator: binding_pattern type_annotation? initializer?;
parameter_list: OPEN_PAREN (parameter (COMMA parameter)*)? CLOSE_PAREN;
formal_parameters: parameter_list;
parameter: required_parameter | optional_parameter;
required_parameter: binding_pattern type_annotation? initializer?;
optional_parameter: binding_pattern QUESTION type_annotation? initializer?;
class_declaration: decorator_list? modifier_list? KEYWORD_CLASS IDENTIFIER type_parameters? heritage_clause? class_body;
decorator_list: decorator+;
decorator: AT expression;
class_body: OPEN_BRACE class_member* CLOSE_BRACE;
class_member: decorator_list? modifier_list? (property_declaration | method_declaration | method_signature | property_signature);
interface_declaration: KEYWORD_INTERFACE IDENTIFIER type_parameters? heritage_clause? interface_body;
interface_body: OPEN_BRACE type_member* CLOSE_BRACE;
type_alias_declaration: KEYWORD_TYPE IDENTIFIER type_parameters? EQUALS type SEMICOLON?;
enum_declaration: KEYWORD_ENUM IDENTIFIER enum_body;
enum_body: OPEN_BRACE enum_member* CLOSE_BRACE;
enum_member: property_name initializer? COMMA?;
namespace_declaration: (KEYWORD_NAMESPACE | KEYWORD_MODULE) IDENTIFIER namespace_body;
namespace_body: OPEN_BRACE statement* CLOSE_BRACE;
expression_statement: expression;
unknown_statement: ts_token+;
return_statement: KEYWORD_RETURN expression? SEMICOLON?;
if_statement: KEYWORD_IF OPEN_PAREN expression CLOSE_PAREN statement else_clause?;
else_clause: KEYWORD_ELSE statement;
for_statement: KEYWORD_FOR ts_token+;
while_statement: KEYWORD_WHILE OPEN_PAREN expression CLOSE_PAREN statement;
do_statement: KEYWORD_DO statement KEYWORD_WHILE OPEN_PAREN expression CLOSE_PAREN SEMICOLON?;
switch_statement: KEYWORD_SWITCH OPEN_PAREN expression CLOSE_PAREN switch_body;
switch_body: OPEN_BRACE switch_case* CLOSE_BRACE;
switch_case: (KEYWORD_CASE expression | KEYWORD_DEFAULT) statements?;
try_statement: KEYWORD_TRY block catch_clause? finally_clause?;
catch_clause: KEYWORD_CATCH catch_binding? block;
catch_binding: OPEN_PAREN binding_pattern type_annotation? CLOSE_PAREN;
finally_clause: KEYWORD_FINALLY block;
throw_statement: KEYWORD_THROW expression? SEMICOLON?;
break_statement: KEYWORD_BREAK SEMICOLON?;
continue_statement: KEYWORD_CONTINUE SEMICOLON?;
from_clause: KEYWORD_FROM STRING_LITERAL;
type_marker: KEYWORD_TYPE | IDENTIFIER;
default_import: IDENTIFIER;
namespace_import: STAR KEYWORD_AS IDENTIFIER;
named_imports: import_specifier+;
import_specifier: IDENTIFIER;
body: block;
block: OPEN_BRACE statement* CLOSE_BRACE;
property_name: IDENTIFIER;
binding_pattern: IDENTIFIER;
object_binding_pattern: OPEN_BRACE binding_element* CLOSE_BRACE;
array_binding_pattern: OPEN_BRACKET binding_element* CLOSE_BRACKET;
binding_element: rest_element | binding_pattern initializer?;
rest_element: DOT_DOT_DOT binding_pattern;
type_annotation: COLON type;
type_parameters: LESS_THAN type (COMMA type)* GREATER_THAN;
return_type: type_annotation;
initializer: EQUALS expression;
value: expression;
expression:
    conditional_expression
    | arrow_function
    | new_expression
    | unary_expression
    | await_expression
    | binary_expression
    | call_expression
    | member_expression
    | element_access_expression
    | object_expression
    | array_expression
    | parenthesized_expression
    | jsx_element
    | jsx_self_closing_element
    | ts_token+
    ;
conditional_expression: ts_token+;
arrow_function: ts_token+;
new_expression: ts_token+;
unary_expression: ts_token+;
await_expression: ts_token+;
binary_expression: ts_token+;
call_expression: ts_token+;
member_expression: ts_token+;
element_access_expression: ts_token+;
object_expression: OPEN_BRACE ts_token* CLOSE_BRACE;
array_expression: OPEN_BRACKET expression? CLOSE_BRACKET;
parenthesized_expression: OPEN_PAREN expression CLOSE_PAREN;
binary_operator:
    EQUALS_EQUALS
    | EQUALS_EQUALS_EQUALS
    | BANG_EQUALS
    | BANG_EQUALS_EQUALS
    | LESS_THAN
    | LESS_THAN_EQUALS
    | GREATER_THAN
    | GREATER_THAN_EQUALS
    | PLUS
    | MINUS
    | STAR
    | SLASH
    | PERCENT
    | PIPE
    | AMPERSAND
    | AMPERSAND_AMPERSAND
    | PIPE_PIPE
    | QUESTION_QUESTION
    | KEYWORD_INSTANCEOF
    | KEYWORD_IN
    | KEYWORD_SATISFIES
    ;
heritage_clause: (KEYWORD_EXTENDS | KEYWORD_IMPLEMENTS) type_reference+;
type:
    type_reference
    | union_type
    | intersection_type
    | array_type
    | object_type
    | tuple_type
    | parenthesized_type
    | conditional_type
    | mapped_type
    | indexed_access_type
    | type_operator
    | ts_token+
    ;
type_reference: IDENTIFIER;
union_type: ts_token+;
intersection_type: ts_token+;
array_type: ts_token+;
object_type: ts_token+;
tuple_type: ts_token+;
parenthesized_type: OPEN_PAREN type CLOSE_PAREN;
type_member: property_signature | method_signature | ts_token+;
type_operator: (KEYWORD_KEYOF | KEYWORD_INFER | KEYWORD_UNIQUE | KEYWORD_READONLY) type;
conditional_type: ts_token+;
mapped_type: ts_token+;
indexed_access_type: ts_token+;
modifier_list:
    (
        KEYWORD_ABSTRACT
        | KEYWORD_ASYNC
        | KEYWORD_DECLARE
        | KEYWORD_DEFAULT
        | KEYWORD_OVERRIDE
        | KEYWORD_PRIVATE
        | KEYWORD_PROTECTED
        | KEYWORD_PUBLIC
        | KEYWORD_READONLY
        | KEYWORD_STATIC
    )+
    ;
jsx_element: jsx_opening_element jsx_child* jsx_closing_element;
jsx_self_closing_element: LESS_THAN jsx_name jsx_attribute* SLASH GREATER_THAN;
jsx_opening_element: LESS_THAN jsx_name jsx_attribute* GREATER_THAN;
jsx_closing_element: LESS_THAN SLASH jsx_name GREATER_THAN;
jsx_child: jsx_text | jsx_expression | jsx_element | jsx_self_closing_element;
jsx_name: IDENTIFIER | member_expression | jsx_namespace_name;
jsx_namespace_name: JSX_NAMESPACE | IDENTIFIER;
jsx_attribute: IDENTIFIER (EQUALS (STRING_LITERAL | jsx_expression))?;
jsx_spread_attribute: OPEN_BRACE DOT_DOT_DOT expression CLOSE_BRACE;
jsx_expression: OPEN_BRACE expression? CLOSE_BRACE;
jsx_text: JSX_TEXT_TOKEN;
statements: statement+;
ts_token:
    IDENTIFIER
    | JSX_IDENTIFIER
    | JSX_NAMESPACE
    | JSX_TEXT_TOKEN
    | STRING_LITERAL
    | TEMPLATE_LITERAL
    | REGEX_LITERAL
    | BIG_INT_LITERAL
    | NUMERIC_LITERAL
    | KEYWORD_ABSTRACT
    | KEYWORD_AS
    | KEYWORD_ASYNC
    | KEYWORD_AWAIT
    | KEYWORD_BREAK
    | KEYWORD_CASE
    | KEYWORD_CATCH
    | KEYWORD_CLASS
    | KEYWORD_CONST
    | KEYWORD_CONTINUE
    | KEYWORD_DECLARE
    | KEYWORD_DEFAULT
    | KEYWORD_DELETE
    | KEYWORD_DO
    | KEYWORD_ELSE
    | KEYWORD_ENUM
    | KEYWORD_EXPORT
    | KEYWORD_EXTENDS
    | KEYWORD_FALSE
    | KEYWORD_FINALLY
    | KEYWORD_FOR
    | KEYWORD_FROM
    | KEYWORD_FUNCTION
    | KEYWORD_GET
    | KEYWORD_IF
    | KEYWORD_IMPLEMENTS
    | KEYWORD_IMPORT
    | KEYWORD_IN
    | KEYWORD_INFER
    | KEYWORD_INSTANCEOF
    | KEYWORD_INTERFACE
    | KEYWORD_KEYOF
    | KEYWORD_LET
    | KEYWORD_MODULE
    | KEYWORD_NAMESPACE
    | KEYWORD_NEW
    | KEYWORD_NULL
    | KEYWORD_OF
    | KEYWORD_OVERRIDE
    | KEYWORD_PRIVATE
    | KEYWORD_PROTECTED
    | KEYWORD_PUBLIC
    | KEYWORD_READONLY
    | KEYWORD_RETURN
    | KEYWORD_SATISFIES
    | KEYWORD_SET
    | KEYWORD_STATIC
    | KEYWORD_SUPER
    | KEYWORD_SWITCH
    | KEYWORD_THIS
    | KEYWORD_THROW
    | KEYWORD_TRUE
    | KEYWORD_TRY
    | KEYWORD_TYPE
    | KEYWORD_TYPEOF
    | KEYWORD_UNIQUE
    | KEYWORD_VOID
    | KEYWORD_WHILE
    | KEYWORD_YIELD
    | JSX_SELF_CLOSE
    | EQUALS_EQUALS_EQUALS
    | BANG_EQUALS_EQUALS
    | DOT_DOT_DOT
    | ARROW
    | EQUALS_EQUALS
    | BANG_EQUALS
    | LESS_THAN_EQUALS
    | GREATER_THAN_EQUALS
    | PLUS_PLUS
    | MINUS_MINUS
    | PLUS_EQUALS
    | MINUS_EQUALS
    | STAR_EQUALS
    | SLASH_EQUALS
    | PERCENT_EQUALS
    | AMPERSAND_AMPERSAND
    | PIPE_PIPE
    | QUESTION_QUESTION
    | QUESTION_DOT
    | OPEN_PAREN
    | CLOSE_PAREN
    | OPEN_BRACE
    | CLOSE_BRACE
    | OPEN_BRACKET
    | CLOSE_BRACKET
    | LESS_THAN
    | GREATER_THAN
    | SLASH
    | PLUS
    | MINUS
    | STAR
    | EQUALS
    | SEMICOLON
    | COLON
    | COMMA
    | DOT
    | AT
    | QUESTION
    | BANG
    | PIPE
    | AMPERSAND
    | PERCENT
    | UNKNOWN
    ;
