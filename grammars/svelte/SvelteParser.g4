parser grammar SvelteParser;
options { tokenVocab=SvelteLexer; }

// G4-facing source of truth; flavor-specific facts live in flavor.g4.json.
svelte_document: top_block* markup EOF;
top_block: module_script | script | style;
module_script: script_tag module_attribute script_content end_tag;
script: script_tag script_content end_tag;
style: style_tag style_content end_tag;
markup: source regions outside top_block;
attribute: ATTRIBUTE_NAME ATTRIBUTE_VALUE?;
