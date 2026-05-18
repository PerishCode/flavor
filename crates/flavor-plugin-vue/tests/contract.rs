use flavor_core::SourceText;
use flavor_grammar::{
    parse_contract, parse_contract_values, GrammarContractExpectation,
    GrammarEntryValueExpectation, GrammarSectionExpectation,
};
use flavor_plugin_vue::{run, VuePluginConfig};

const VUE_SFC_METADATA: &str = include_str!("../../../grammars/vue/metadata.json");
const VUE_TEMPLATE_METADATA: &str = include_str!("../../../grammars/vue/metadata.json");
const VUE_SFC_CONTRACT: GrammarContractExpectation<'static> = GrammarContractExpectation {
    name: "vue-sfc",
    directives: &[
        ("owner", "crates/flavor-plugin-vue"),
        ("entry", "sfc_document"),
    ],
    sections: &[
        GrammarSectionExpectation {
            name: "facts",
            entries: &[
                "descriptor.script",
                "descriptor.script_setup",
                "descriptor.styles",
                "descriptor.template",
                "script.embedded",
            ],
        },
        GrammarSectionExpectation {
            name: "diagnostics",
            entries: &[
                "duplicate.script",
                "duplicate.setup",
                "missing.close",
                "unsupported.setup_src",
            ],
        },
        GrammarSectionExpectation {
            name: "spans",
            entries: &[
                "block.content",
                "block.line",
                "diagnostic.range",
                "embedded.offset",
            ],
        },
        GrammarSectionExpectation {
            name: "recovery",
            entries: &["descriptor.blocks", "embedded.skip", "missing.close"],
        },
    ],
};
const VUE_SFC_VALUES: &[GrammarEntryValueExpectation<'static>] = &[
    GrammarEntryValueExpectation {
        section: "facts",
        key: "descriptor.script",
        contains: &[
            "VueSfcBlock",
            "payload.content",
            "payload.start_offset",
            "payload.start_line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "descriptor.script_setup",
        contains: &[
            "VueSfcBlock",
            "payload.content",
            "payload.start_offset",
            "payload.start_line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "script.embedded",
        contains: &[
            "embedded",
            "payload.content",
            "payload.lang",
            "payload.tsx",
            "payload.start_offset",
            "payload.start_line",
        ],
    },
];
const VUE_TEMPLATE_CONTRACT: GrammarContractExpectation<'static> = GrammarContractExpectation {
    name: "vue-template",
    directives: &[
        ("owner", "crates/flavor-plugin-vue"),
        ("entry", "template_document"),
    ],
    sections: &[
        GrammarSectionExpectation {
            name: "facts",
            entries: &[
                "directive.class",
                "embedded.expression",
                "template.directive",
                "template.element",
                "template.expression",
            ],
        },
        GrammarSectionExpectation {
            name: "diagnostics",
            entries: &[
                "expression.error",
                "missing.end_tag",
                "missing.interpolation",
            ],
        },
        GrammarSectionExpectation {
            name: "spans",
            entries: &["diagnostic.range", "host.range", "line", "node.range"],
        },
        GrammarSectionExpectation {
            name: "recovery",
            entries: &["missing.end_tag", "missing.interpolation", "v_pre"],
        },
    ],
};
const VUE_TEMPLATE_VALUES: &[GrammarEntryValueExpectation<'static>] = &[
    GrammarEntryValueExpectation {
        section: "facts",
        key: "template.element",
        contains: &[
            "VueFacts.template_elements",
            "VueTemplateElementFact",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "template.directive",
        contains: &[
            "VueFacts.template_directives",
            "VueTemplateDirectiveFact",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "template.expression",
        contains: &[
            "VueFacts.template_expressions",
            "VueTemplateExpressionFact",
            "span",
            "line",
        ],
    },
];

#[test]
fn vue_sfc_sections() {
    parse_contract_values(VUE_SFC_METADATA, &VUE_SFC_CONTRACT, VUE_SFC_VALUES).unwrap();
}

#[test]
fn vue_sfc_facts() {
    parse_contract(VUE_SFC_METADATA, &VUE_SFC_CONTRACT).unwrap();
    let output = run(
        SourceText::new(
            "Contract.vue",
            "<template><button v-if=\"ok\">{{ label }}</button></template>\n\
             <script lang=\"ts\">export default {}</script>\n\
             <script setup lang=\"tsx\">const vnode = <span />;</script>\n\
             <style scoped>.root { color: red; }</style>\n",
        ),
        VuePluginConfig::default(),
    );

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert!(output.descriptor.template.is_some());
    assert_eq!(
        output
            .descriptor
            .script
            .as_ref()
            .and_then(|block| block.attrs.get("lang"))
            .and_then(|value| value.as_deref()),
        Some("ts")
    );
    assert_eq!(
        output
            .descriptor
            .script_setup
            .as_ref()
            .and_then(|block| block.attrs.get("lang"))
            .and_then(|value| value.as_deref()),
        Some("tsx")
    );
    assert_eq!(output.descriptor.styles.len(), 1);
    assert_eq!(output.facts.script_count, 2);
    assert_eq!(output.facts.style_count, 1);
}

#[test]
fn vue_sfc_diagnostics() {
    parse_contract(VUE_SFC_METADATA, &VUE_SFC_CONTRACT).unwrap();

    let duplicate = run(
        SourceText::new(
            "Duplicate.vue",
            "<script>const first = 1;</script>\n<script>const second = 2;</script>",
        ),
        VuePluginConfig::default(),
    );
    assert_has_diagnostic(&duplicate.diagnostics, "duplicate top-level <script>");

    let setup_src = run(
        SourceText::new("SetupSrc.vue", "<script setup src=\"./setup.ts\"></script>"),
        VuePluginConfig::default(),
    );
    assert_has_diagnostic(&setup_src.diagnostics, "cannot use src");

    let missing = run(
        SourceText::new("Missing.vue", "<template><main /></template>\n<script>"),
        VuePluginConfig::default(),
    );
    assert_has_diagnostic(&missing.diagnostics, "missing closing </script>");
}

#[test]
fn vue_template_sections() {
    parse_contract_values(
        VUE_TEMPLATE_METADATA,
        &VUE_TEMPLATE_CONTRACT,
        VUE_TEMPLATE_VALUES,
    )
    .unwrap();
}

#[test]
fn vue_template_facts() {
    parse_contract(VUE_TEMPLATE_METADATA, &VUE_TEMPLATE_CONTRACT).unwrap();
    let output = run(
        SourceText::new(
            "Template.vue",
            r#"<template><slot v-if="ok" :[name].camel="value" @click="save" #default v-model="model">{{ label }}</slot></template>"#,
        ),
        VuePluginConfig::default(),
    );

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert_eq!(output.facts.template_element_count, 1);
    assert_eq!(output.facts.template_directive_count, 5);
    assert_eq!(output.facts.template_control_directive_count, 1);
    assert_eq!(output.facts.template_bind_directive_count, 1);
    assert_eq!(output.facts.template_event_directive_count, 1);
    assert_eq!(output.facts.template_slot_directive_count, 1);
    assert_eq!(output.facts.template_model_directive_count, 1);
    assert_eq!(output.facts.template_interpolation_count, 1);
    assert_eq!(output.facts.template_dynamic_argument_count, 1);
    assert_eq!(output.facts.template_modifier_count, 1);
    assert_eq!(output.facts.template_expression_count, 6);
    assert_eq!(output.facts.template_elements.len(), 1);
    assert_eq!(output.facts.template_directives.len(), 5);
    assert_eq!(output.facts.template_expressions.len(), 6);

    let slot = output
        .facts
        .template_elements
        .iter()
        .find(|element| element.name == "slot")
        .expect("slot element fact");
    assert!(output.source.slice(slot.span).starts_with("<slot "));
    assert_eq!(slot.line, 1);

    let bind = output
        .facts
        .template_directives
        .iter()
        .find(|directive| directive.name.starts_with(':'))
        .expect("bind directive fact");
    assert_eq!(output.source.slice(bind.span), r#":[name].camel="value""#);

    let dynamic_argument = output
        .facts
        .template_expressions
        .iter()
        .find(|expression| output.source.slice(expression.span) == "[name]")
        .expect("dynamic argument expression fact");
    assert_eq!(dynamic_argument.line, 1);
}

#[test]
fn vue_template_diagnostics() {
    parse_contract(VUE_TEMPLATE_METADATA, &VUE_TEMPLATE_CONTRACT).unwrap();

    let missing_end = run(
        SourceText::new("MissingEnd.vue", "<template><div><span></div></template>"),
        VuePluginConfig::default(),
    );
    assert_has_diagnostic(&missing_end.diagnostics, "missing closing </span>");

    let missing_interpolation = run(
        SourceText::new(
            "MissingInterpolation.vue",
            "<template><div>{{ value</div></template>",
        ),
        VuePluginConfig::default(),
    );
    assert_has_diagnostic(
        &missing_interpolation.diagnostics,
        "missing interpolation close delimiter",
    );

    let expression = run(
        SourceText::new(
            "Expression.vue",
            r#"<template><div :class="user." /></template>"#,
        ),
        VuePluginConfig::default(),
    );
    assert_has_diagnostic(&expression.diagnostics, "expected property name");
}

#[test]
fn vue_recovery_maps_spans() {
    parse_contract(VUE_TEMPLATE_METADATA, &VUE_TEMPLATE_CONTRACT).unwrap();
    let source = r#"<template><main><span></main><button :class="user." /></template>"#;
    let output = run(
        SourceText::new("Recover.vue", source),
        VuePluginConfig::default(),
    );

    assert_eq!(output.facts.template_element_count, 3);
    let button = output
        .facts
        .template_elements
        .iter()
        .find(|element| element.name == "button")
        .expect("button element fact after recovery");
    assert!(output.source.slice(button.span).starts_with("<button "));

    let missing = find_diagnostic(&output.diagnostics, "missing closing </span>");
    assert_eq!(
        missing.span.map(|span| span.start as usize),
        source.find("<span>")
    );

    let expression = find_diagnostic(&output.diagnostics, "expected property name");
    let expression_span = expression.span.expect("expression span");
    assert_eq!(
        expression_span.start as usize,
        source.find("user.").expect("user expression") + "user.".len()
    );
}

fn find_diagnostic<'a>(
    diagnostics: &'a [flavor_core::Diagnostic],
    message: &str,
) -> &'a flavor_core::Diagnostic {
    diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains(message))
        .unwrap_or_else(|| panic!("missing diagnostic containing `{message}`: {diagnostics:?}"))
}

fn assert_has_diagnostic(diagnostics: &[flavor_core::Diagnostic], message: &str) {
    let diagnostic = find_diagnostic(diagnostics, message);
    assert_eq!(diagnostic.code.as_deref(), Some("vue/parse/error"));
    assert!(diagnostic.span.is_some(), "missing span for {diagnostic:?}");
}
