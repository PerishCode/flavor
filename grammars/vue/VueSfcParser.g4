parser grammar VueSfcParser;
options { tokenVocab=VueSfcLexer; }

// G4-facing source of truth; flavor-specific facts live in flavor.g4.json.
sfc_document: block* EOF;
block: template_block | script_block | script_setup_block | style_block | custom_block;
template_block: start_tag template_content end_tag;
script_block: start_tag script_content end_tag;
script_setup_block: start_tag setup_attribute script_content end_tag;
style_block: start_tag style_content end_tag;
attribute: ATTRIBUTE_NAME ATTRIBUTE_VALUE?;
