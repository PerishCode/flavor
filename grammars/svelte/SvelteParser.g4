parser grammar SvelteParser;
options { tokenVocab=SvelteLexer; }

// Hand-written G4 parser surface; flavor contracts live in metadata.json.
svelte_document: top_block* markup EOF;
top_block: module_script | script | style;
module_script: script_tag module_attribute script_content end_tag;
script: script_tag script_content end_tag;
style: style_tag style_content end_tag;
markup: RAW_TEXT+;
attribute: ATTRIBUTE_NAME ATTRIBUTE_VALUE?;
script_tag: TAG_OPEN ATTRIBUTE_NAME attribute* TAG_CLOSE;
style_tag: TAG_OPEN ATTRIBUTE_NAME attribute* TAG_CLOSE;
module_attribute: attribute;
script_content: RAW_TEXT+;
style_content: RAW_TEXT+;
end_tag: END_TAG_OPEN ATTRIBUTE_NAME TAG_CLOSE;
