use flavor_compiler_core::SourceText;
use flavor_compiler_vue::{run, TemplateConfig, VueCompilerConfig};

#[test]
fn run_accepts_injected_config() {
    let config = VueCompilerConfig {
        template: TemplateConfig {
            ast: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let output = run(
        SourceText::new("Sample.vue", "<template><div /></template>"),
        config,
    );

    assert_eq!(output.source.name(), "Sample.vue");
    assert!(output.descriptor.template.is_some());
    assert!(output.template.is_none());
    assert!(output.diagnostics.is_empty());
}

#[test]
fn run_collects_template_facts() {
    let output = run(
        SourceText::new(
            "Sample.vue",
            r#"<template><button v-if="ok" @click="save">Save</button></template><script setup>const ok = true</script><style scoped>.ok {}</style>"#,
        ),
        VueCompilerConfig::default(),
    );

    assert_eq!(output.facts.script_count, 1);
    assert_eq!(output.facts.style_count, 1);
    assert_eq!(output.facts.template_element_count, 1);
    assert_eq!(output.facts.template_directive_count, 2);
    assert_eq!(output.facts.template_control_directive_count, 1);
    assert_eq!(output.facts.template_event_directive_count, 1);
    assert_eq!(output.facts.template_expression_count, 2);
    assert_eq!(output.facts.template_directive_expression_count, 2);
    assert!(output.diagnostics.is_empty());
}

#[test]
fn classifies_template_directives() {
    let output = run(
        SourceText::new(
            "Sample.vue",
            r##"<template><slot v-if="ok" v-for="item in items" :name="name" @click="save" #default v-model="value" v-focus /></template>"##,
        ),
        VueCompilerConfig::default(),
    );

    assert_eq!(output.facts.template_directive_count, 7);
    assert_eq!(output.facts.template_control_directive_count, 2);
    assert_eq!(output.facts.template_bind_directive_count, 1);
    assert_eq!(output.facts.template_event_directive_count, 1);
    assert_eq!(output.facts.template_slot_directive_count, 1);
    assert_eq!(output.facts.template_model_directive_count, 1);
    assert_eq!(output.facts.template_custom_directive_count, 1);
    assert_eq!(output.facts.template_expression_count, 5);
    assert_eq!(output.facts.template_directive_expression_count, 5);
    assert!(output.diagnostics.is_empty());
}

#[test]
fn counts_template_expressions() {
    let output = run(
        SourceText::new(
            "Sample.vue",
            r#"<template><div :class="kind">{{ title }}</div></template>"#,
        ),
        VueCompilerConfig::default(),
    );

    assert_eq!(output.facts.template_interpolation_count, 1);
    assert_eq!(output.facts.template_directive_expression_count, 1);
    assert_eq!(output.facts.template_expression_count, 2);
    assert!(output.diagnostics.is_empty());
}

#[test]
fn skips_v_pre_exprs() {
    let output = run(
        SourceText::new(
            "Sample.vue",
            r#"<template><div v-pre>{{ broken( }}</div></template>"#,
        ),
        VueCompilerConfig::default(),
    );

    assert_eq!(output.facts.template_interpolation_count, 0);
    assert_eq!(output.facts.template_expression_count, 0);
    assert!(output.diagnostics.is_empty());
}

#[test]
fn reports_template_expr_errors() {
    let output = run(
        SourceText::new(
            "Broken.vue",
            r#"<template><div :class="user.">{{ call( }}</div></template>"#,
        ),
        VueCompilerConfig::default(),
    );

    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "expected property name"));
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "expected ')' to close call arguments"));
}

#[test]
fn reports_dynamic_arg_errors() {
    let output = run(
        SourceText::new(
            "Broken.vue",
            r#"<template><div :[user.]="value" /></template>"#,
        ),
        VueCompilerConfig::default(),
    );

    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "expected property name"));
}

#[test]
fn can_skip_template_exprs() {
    let config = VueCompilerConfig {
        template: TemplateConfig {
            expressions: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let output = run(
        SourceText::new(
            "Broken.vue",
            r#"<template><div :class="user." /></template>"#,
        ),
        config,
    );

    assert!(output.diagnostics.is_empty());
}

#[test]
fn maps_sfc_errors() {
    let output = run(
        SourceText::new(
            "Broken.vue",
            "<template><div /></template>\n<script setup>const value = 1;",
        ),
        VueCompilerConfig::default(),
    );

    assert_eq!(output.descriptor.errors.len(), 1);
    assert_eq!(output.diagnostics.len(), 1);
    assert!(output.diagnostics[0]
        .message
        .contains("missing closing </script>"));
    assert_eq!(output.diagnostics[0].span.map(|span| span.start), Some(29));
}

#[test]
fn maps_template_errors() {
    let output = run(
        SourceText::new("Broken.vue", "<template><div><span></div></template>"),
        VueCompilerConfig::default(),
    );

    assert!(output.descriptor.errors.is_empty());
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(output.diagnostics[0].message, "missing closing </span> tag");
    assert_eq!(output.diagnostics[0].span.map(|span| span.start), Some(15));
}
