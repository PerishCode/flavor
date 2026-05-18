parser grammar TypeScriptParser;
options { tokenVocab=TypeScriptLexer; }

// G4-facing source of truth; flavor-specific facts live in flavor.g4.json.
program: statement* EOF;
statement: import_statement | export_statement | declaration | expression_statement;
import_statement: KEYWORD_IMPORT import_clause? from_clause?;
import_clause: type_marker? default_import? namespace_import? named_imports?;
function_declaration: KEYWORD_FUNCTION IDENTIFIER formal_parameters body;
method_definition: property_name formal_parameters body;
variable_declarator: binding_pattern type? value?;
parameter: required_parameter | optional_parameter;
switch_case: KEYWORD_CASE expression statements;
