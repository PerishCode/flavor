use flavor_compiler_core::SourceText;
use flavor_compiler_vue::{run, VueCompilerConfig};

#[test]
fn parses_harness_cases() {
    let output = run(
        SourceText::new(
            "component.vue",
            include_str!("../harness/cases/component.vue"),
        ),
        VueCompilerConfig::default(),
    );

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert_eq!(output.facts.script_count, 1);
    assert_eq!(output.facts.style_count, 1);
    assert_eq!(output.facts.template_element_count, 2);
    assert_eq!(output.facts.template_control_directive_count, 1);
    assert_eq!(output.facts.template_bind_directive_count, 1);
    assert_eq!(output.facts.template_event_directive_count, 1);
    assert_eq!(output.facts.template_slot_directive_count, 1);
    assert_eq!(output.facts.template_model_directive_count, 1);
    assert_eq!(output.facts.template_dynamic_argument_count, 2);
    assert_eq!(output.facts.template_modifier_count, 1);
    assert_eq!(output.facts.template_interpolation_count, 1);
    assert_eq!(output.facts.template_expression_count, 8);
}
