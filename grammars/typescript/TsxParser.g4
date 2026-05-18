parser grammar TsxParser;
options { tokenVocab=TypeScriptLexer; }

// G4-facing source of truth; flavor-specific facts live in flavor.g4.json.
program: typescript.statement* EOF;
jsx_element: jsx_opening_element jsx_child* jsx_closing_element;
jsx_self_closing_element: JSX_OPEN jsx_name jsx_attribute* JSX_SELF_CLOSE;
jsx_opening_element: JSX_OPEN jsx_name jsx_attribute* JSX_CLOSE;
jsx_name: identifier | member_expression | jsx_namespace_name;
jsx_child: JSX_TEXT | jsx_expression | jsx_element | jsx_self_closing_element;
