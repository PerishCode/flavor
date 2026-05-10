use flavor_compiler_core::RawSyntaxKind;

use crate::{
    sfc::VueSfcDescriptor,
    template::{TemplateAst, VueTemplateKind},
};

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct VueFacts {
    pub style_count: usize,
    pub script_count: usize,
    pub template_element_count: usize,
    pub template_directive_count: usize,
    pub template_bind_directive_count: usize,
    pub template_event_directive_count: usize,
    pub template_slot_directive_count: usize,
    pub template_model_directive_count: usize,
    pub template_control_directive_count: usize,
    pub template_custom_directive_count: usize,
    pub template_interpolation_count: usize,
    pub template_expression_count: usize,
    pub template_directive_expression_count: usize,
    pub template_dynamic_argument_count: usize,
    pub template_modifier_count: usize,
}

pub fn collect(descriptor: &VueSfcDescriptor, template: Option<&TemplateAst>) -> VueFacts {
    let mut facts = VueFacts {
        style_count: descriptor.styles.len(),
        script_count: usize::from(descriptor.script.is_some())
            + usize::from(descriptor.script_setup.is_some()),
        ..Default::default()
    };
    if let Some(template) = template {
        for node in template.syntax().descendants() {
            match node.kind() {
                kind if kind == RawSyntaxKind::from(VueTemplateKind::StartTag) => {
                    facts.template_element_count += 1;
                }
                kind if kind == RawSyntaxKind::from(VueTemplateKind::Directive) => {
                    facts.template_directive_count += 1;
                }
                kind if kind == RawSyntaxKind::from(VueTemplateKind::Interpolation) => {
                    facts.template_interpolation_count += 1;
                    facts.template_expression_count += 1;
                }
                kind if kind == RawSyntaxKind::from(VueTemplateKind::DirectiveExpression) => {
                    facts.template_directive_expression_count += 1;
                    facts.template_expression_count += 1;
                }
                kind if kind == RawSyntaxKind::from(VueTemplateKind::DirectiveName) => {
                    match classify_directive(&node.text().to_string()) {
                        DirectiveClass::Bind => facts.template_bind_directive_count += 1,
                        DirectiveClass::Event => facts.template_event_directive_count += 1,
                        DirectiveClass::Slot => facts.template_slot_directive_count += 1,
                        DirectiveClass::Model => facts.template_model_directive_count += 1,
                        DirectiveClass::Control => facts.template_control_directive_count += 1,
                        DirectiveClass::Custom => facts.template_custom_directive_count += 1,
                    }
                }
                _ => {}
            }
        }
        for token in template.syntax().descendants_with_tokens() {
            let Some(token) = token.into_token() else {
                continue;
            };
            match token.kind() {
                kind if kind == RawSyntaxKind::from(VueTemplateKind::DirectiveArgument)
                    && token.text().contains('[') =>
                {
                    facts.template_dynamic_argument_count += 1;
                    facts.template_expression_count += 1;
                }
                kind if kind == RawSyntaxKind::from(VueTemplateKind::DirectiveModifier) => {
                    facts.template_modifier_count += 1;
                }
                _ => {}
            }
        }
    }
    facts
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum DirectiveClass {
    Bind,
    Event,
    Slot,
    Model,
    Control,
    Custom,
}

fn classify_directive(name: &str) -> DirectiveClass {
    if name.starts_with(':') || name.starts_with("v-bind") {
        DirectiveClass::Bind
    } else if name.starts_with('@') || name.starts_with("v-on") {
        DirectiveClass::Event
    } else if name.starts_with('#') || name.starts_with("v-slot") {
        DirectiveClass::Slot
    } else if name.starts_with("v-model") {
        DirectiveClass::Model
    } else if matches!(
        name,
        "v-if" | "v-else-if" | "v-else" | "v-for" | "v-show" | "v-memo" | "v-once"
    ) {
        DirectiveClass::Control
    } else {
        DirectiveClass::Custom
    }
}
