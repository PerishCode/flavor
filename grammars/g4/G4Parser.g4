parser grammar G4Parser;
options { tokenVocab=G4Lexer; }

grammar_file: grammar_declaration grammar_member* EOF;
grammar_declaration: (LEXER_GRAMMAR | PARSER_GRAMMAR | GRAMMAR) UPPER_IDENTIFIER SEMI;
grammar_member: options_block | parser_rule | lexer_token | ignored_member;
options_block: OPTIONS LBRACE option_assignment* RBRACE;
option_assignment: LOWER_IDENTIFIER ASSIGN UPPER_IDENTIFIER SEMI?;
parser_rule: LOWER_IDENTIFIER COLON rule_body SEMI;
lexer_token: UPPER_IDENTIFIER COLON rule_body SEMI;
rule_body: rule_atom*;
rule_atom:
    LOWER_IDENTIFIER
    | UPPER_IDENTIFIER
    | STRING_LITERAL
    | PIPE
    | STAR
    | PLUS
    | QUESTION
    | LPAREN
    | RPAREN
    | LBRACE
    | RBRACE
    | ASSIGN
    | RAW_TEXT
    ;
ignored_member: RAW_TEXT;
