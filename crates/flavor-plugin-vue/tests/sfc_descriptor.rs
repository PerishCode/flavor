use flavor_grammar::parse_vue_sfc;

#[test]
fn parses_sfc_descriptor() {
    let descriptor = parse_vue_sfc(
        "<!-- header -->\n\
         <template lang=\"pug\">\n\
         div {{ message }}\n\
         </template>\n\
         <script lang=\"ts\">\n\
         export default {}\n\
         </script>\n\
         <script setup lang=\"tsx\">\n\
         const message = <span />;\n\
         </script>\n\
         <style scoped module=\"classes\">\n\
         .root { color: red; }\n\
         </style>\n\
         <i18n locale='en'>hello</i18n>",
    );

    let template = descriptor.template.expect("template block");
    let script = descriptor.script.expect("script block");
    let script_setup = descriptor.script_setup.expect("script setup block");

    assert_eq!(
        template
            .attrs
            .get("lang")
            .and_then(|value| value.as_deref()),
        Some("pug")
    );
    assert_eq!(
        script.attrs.get("lang").and_then(|value| value.as_deref()),
        Some("ts")
    );
    assert!(script.content.contains("export default"));
    assert_eq!(
        script_setup
            .attrs
            .get("lang")
            .and_then(|value| value.as_deref()),
        Some("tsx")
    );
    assert_eq!(descriptor.styles.len(), 1);
    assert!(descriptor.styles[0].attrs.contains_key("scoped"));
    assert_eq!(
        descriptor.styles[0]
            .attrs
            .get("module")
            .and_then(|value| value.as_deref()),
        Some("classes")
    );
    assert_eq!(descriptor.custom_blocks.len(), 1);
    assert_eq!(descriptor.custom_blocks[0].tag, "i18n");
    assert!(descriptor.errors.is_empty());
}

#[test]
fn reports_duplicate_script_setup() {
    let descriptor = parse_vue_sfc(
        "<script setup>const first = 1;</script>\n\
         <script setup>const second = 2;</script>",
    );

    assert_eq!(descriptor.errors.len(), 1);
    assert_eq!(descriptor.errors[0].line, 2);
    assert!(descriptor.errors[0]
        .message
        .contains("duplicate top-level <script setup>"));
}

#[test]
fn reports_missing_closing_block() {
    let descriptor =
        parse_vue_sfc("<template><div></div></template>\n<script setup>const value = 1;");

    assert_eq!(descriptor.errors.len(), 1);
    assert_eq!(descriptor.errors[0].line, 2);
    assert!(descriptor.errors[0]
        .message
        .contains("missing closing </script>"));
}

#[test]
fn reports_script_setup_src() {
    let descriptor = parse_vue_sfc("<script setup src=\"./setup.ts\"></script>");

    assert_eq!(descriptor.errors.len(), 1);
    assert!(descriptor.errors[0].message.contains("cannot use src"));
}

#[test]
fn rejects_script_src_setup() {
    let descriptor = parse_vue_sfc(
        "<script src=\"./options.ts\"></script>\n\
         <script setup lang=\"ts\">const value = 1;</script>",
    );

    assert_eq!(descriptor.errors.len(), 1);
    assert_eq!(descriptor.errors[0].line, 1);
    assert!(descriptor.errors[0]
        .message
        .contains("<script> cannot use src"));
}

#[test]
fn reports_missing_main_block() {
    let descriptor = parse_vue_sfc("<style>.root { color: red; }</style>");

    assert_eq!(descriptor.errors.len(), 1);
    assert!(descriptor.errors[0]
        .message
        .contains("at least one <template> or <script>"));
}

#[test]
fn maps_script_line_offset() {
    let descriptor = parse_vue_sfc(
        "<template></template>\n\
         <script setup lang=\"ts\">\n\
         const value = 1;\n\
         </script>",
    );

    assert_eq!(descriptor.script_setup.expect("script setup").start_line, 1);
}
