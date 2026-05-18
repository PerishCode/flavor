parser grammar VueSfcParser;
options { tokenVocab=VueSfcLexer; }

// Hand-written G4 parser surface; flavor contracts live in metadata.json.
sfc_document: block* EOF;
block: template_block | script_block | script_setup_block | style_block | custom_block;
template_block: start_tag template_content end_tag;
script_block: start_tag script_content end_tag;
script_setup_block: start_tag setup_attribute script_content end_tag;
style_block: start_tag style_content end_tag;
attribute: ATTRIBUTE_NAME ATTRIBUTE_VALUE?;
custom_block: start_tag RAW_TEXT? end_tag;
start_tag: TAG_OPEN ATTRIBUTE_NAME attribute* TAG_CLOSE;
end_tag: END_TAG_OPEN ATTRIBUTE_NAME TAG_CLOSE;
template_content: RAW_TEXT+;
script_content: RAW_TEXT+;
style_content: RAW_TEXT+;
setup_attribute: attribute;
