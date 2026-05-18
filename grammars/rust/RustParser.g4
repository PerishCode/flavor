parser grammar RustParser;
options { tokenVocab=RustLexer; }

// Hand-written G4 parser surface; flavor contracts live in metadata.json.
source_file: item* EOF;
item: ATTRIBUTE* function_item | ATTRIBUTE* impl_item | ATTRIBUTE* trait_item | ATTRIBUTE* mod_item | statement;
function_item: ATTRIBUTE* KEYWORD_FN IDENTIFIER parameters return_type? block;
function_signature_item: ATTRIBUTE* KEYWORD_FN IDENTIFIER parameters return_type?;
impl_item: KEYWORD_IMPL trait_ref? type body?;
trait_item: KEYWORD_TRAIT IDENTIFIER body;
let_declaration: KEYWORD_LET pattern type? value?;
match_expression: KEYWORD_MATCH expression match_block;
match_arm: pattern guard? arm_value;
mod_item: ATTRIBUTE* IDENTIFIER block?;
statement: let_declaration | match_expression | expression;
parameters: parameter*;
parameter: pattern type?;
return_type: rust_token+;
block: rust_token+;
trait_ref: IDENTIFIER;
type: IDENTIFIER;
body: block;
pattern: IDENTIFIER | rust_token;
value: expression;
expression: rust_token+;
match_block: block;
guard: expression;
arm_value: expression | block;
rust_token: IDENTIFIER | ATTRIBUTE | INNER_ATTRIBUTE | KEYWORD_FN | KEYWORD_IMPL | KEYWORD_TRAIT | KEYWORD_LET | KEYWORD_MATCH | WS | RAW_TEXT;
