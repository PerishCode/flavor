parser grammar RustParser;
options { tokenVocab=RustLexer; }

// G4-facing source of truth; flavor-specific facts live in flavor.g4.json.
source_file: item* EOF;
item: ATTRIBUTE* function_item | ATTRIBUTE* impl_item | ATTRIBUTE* trait_item | ATTRIBUTE* mod_item | statement;
function_item: ATTRIBUTE* KEYWORD_FN IDENTIFIER parameters return_type? block;
function_signature_item: ATTRIBUTE* KEYWORD_FN IDENTIFIER parameters return_type?;
impl_item: KEYWORD_IMPL trait_ref? type body?;
trait_item: KEYWORD_TRAIT IDENTIFIER body;
let_declaration: KEYWORD_LET pattern type? value?;
match_expression: KEYWORD_MATCH expression match_block;
match_arm: pattern guard? arm_value;
