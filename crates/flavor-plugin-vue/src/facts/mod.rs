use flavor_core::{LineIndex, Span};
use flavor_grammar::{GrammarNode, GrammarToken, GrammarTree};

use crate::{
    sfc::{VueSfcBlock, VueSfcDescriptor},
    template::{kind, TemplateAst},
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
    pub template_elements: Vec<VueTemplateElementFact>,
    pub template_directives: Vec<VueTemplateDirectiveFact>,
    pub template_expressions: Vec<VueTemplateExpressionFact>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VueTemplateElementFact {
    pub name: String,
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VueTemplateDirectiveFact {
    pub name: String,
    pub class: VueTemplateDirectiveClass,
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VueTemplateExpressionFact {
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VueTemplateDirectiveClass {
    Bind,
    Event,
    Slot,
    Model,
    Control,
    Custom,
}

type VueNode = GrammarNode;
type VueToken = GrammarToken;

pub fn collect(descriptor: &VueSfcDescriptor, template: Option<&TemplateAst>) -> VueFacts {
    let mut facts = VueFacts {
        style_count: descriptor.styles.len(),
        script_count: usize::from(descriptor.script.is_some())
            + usize::from(descriptor.script_setup.is_some()),
        ..Default::default()
    };
    if let Some(template) = template {
        let host = TemplateHost::new(descriptor.template.as_ref());
        let tree = GrammarTree::new(template.syntax().clone(), kind::schema());
        let root = tree.root();
        for node in root.descendants() {
            match node.kind_name() {
                Some("start_tag") => {
                    facts.template_element_count += 1;
                    if let Some(fact) = element_fact(&node, &host) {
                        facts.template_elements.push(fact);
                    }
                }
                Some("directive") => {
                    facts.template_directive_count += 1;
                    if let Some(fact) = directive_fact(&node, &host) {
                        facts.template_directives.push(fact);
                    }
                }
                Some("interpolation") => {
                    facts.template_interpolation_count += 1;
                    facts.template_expression_count += 1;
                    facts
                        .template_expressions
                        .push(expression_fact(&node, &host));
                }
                Some("directive_expression") => {
                    facts.template_directive_expression_count += 1;
                    facts.template_expression_count += 1;
                    facts
                        .template_expressions
                        .push(expression_fact(&node, &host));
                }
                Some("directive_name") => match classify_directive(&node.text()) {
                    VueTemplateDirectiveClass::Bind => facts.template_bind_directive_count += 1,
                    VueTemplateDirectiveClass::Event => facts.template_event_directive_count += 1,
                    VueTemplateDirectiveClass::Slot => facts.template_slot_directive_count += 1,
                    VueTemplateDirectiveClass::Model => facts.template_model_directive_count += 1,
                    VueTemplateDirectiveClass::Control => {
                        facts.template_control_directive_count += 1
                    }
                    VueTemplateDirectiveClass::Custom => facts.template_custom_directive_count += 1,
                },
                _ => {}
            }
        }
        for token in root.tokens() {
            match token.kind_name() {
                Some("DIRECTIVE_ARGUMENT") if token.text().contains('[') => {
                    facts.template_dynamic_argument_count += 1;
                    facts.template_expression_count += 1;
                    facts
                        .template_expressions
                        .push(token_expression_fact(&token, &host));
                }
                Some("DIRECTIVE_MODIFIER") => {
                    facts.template_modifier_count += 1;
                }
                _ => {}
            }
        }
    }
    facts
}

struct TemplateHost<'a> {
    block: Option<&'a VueSfcBlock>,
    line_index: Option<LineIndex>,
}

impl<'a> TemplateHost<'a> {
    fn new(block: Option<&'a VueSfcBlock>) -> Self {
        Self {
            block,
            line_index: block.map(|block| LineIndex::new(&block.content)),
        }
    }

    fn span(&self, span: Span) -> Span {
        span.shifted(
            self.block
                .map(|block| u32::try_from(block.start_offset).unwrap_or(u32::MAX))
                .unwrap_or(0),
        )
    }

    fn line(&self, local_start: u32) -> usize {
        match (self.block, self.line_index.as_ref()) {
            (Some(block), Some(line_index)) => block.start_line + line_index.line(local_start),
            _ => 1,
        }
    }
}

fn element_fact(node: &VueNode, host: &TemplateHost<'_>) -> Option<VueTemplateElementFact> {
    let span = node.span();
    Some(VueTemplateElementFact {
        name: node.child_token_text("TAG_NAME")?,
        span: host.span(span),
        line: host.line(span.start),
    })
}

fn directive_fact(node: &VueNode, host: &TemplateHost<'_>) -> Option<VueTemplateDirectiveFact> {
    let name = node.child_text("directive_name")?;
    let span = node.span();
    Some(VueTemplateDirectiveFact {
        class: classify_directive(&name),
        name,
        span: host.span(span),
        line: host.line(span.start),
    })
}

fn expression_fact(node: &VueNode, host: &TemplateHost<'_>) -> VueTemplateExpressionFact {
    let span = node.span();
    VueTemplateExpressionFact {
        span: host.span(span),
        line: host.line(span.start),
    }
}

fn token_expression_fact(token: &VueToken, host: &TemplateHost<'_>) -> VueTemplateExpressionFact {
    let span = token.span();
    VueTemplateExpressionFact {
        span: host.span(span),
        line: host.line(span.start),
    }
}

fn classify_directive(name: &str) -> VueTemplateDirectiveClass {
    if name.starts_with(':') || name.starts_with("v-bind") {
        VueTemplateDirectiveClass::Bind
    } else if name.starts_with('@') || name.starts_with("v-on") {
        VueTemplateDirectiveClass::Event
    } else if name.starts_with('#') || name.starts_with("v-slot") {
        VueTemplateDirectiveClass::Slot
    } else if name.starts_with("v-model") {
        VueTemplateDirectiveClass::Model
    } else if matches!(
        name,
        "v-if" | "v-else-if" | "v-else" | "v-for" | "v-show" | "v-memo" | "v-once"
    ) {
        VueTemplateDirectiveClass::Control
    } else {
        VueTemplateDirectiveClass::Custom
    }
}
