use flavor_core::{LineIndex, RawSyntaxKind, Span, SyntaxNode, SyntaxSpanExt, SyntaxToken};

use crate::{
    sfc::{VueSfcBlock, VueSfcDescriptor},
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

pub fn collect(descriptor: &VueSfcDescriptor, template: Option<&TemplateAst>) -> VueFacts {
    let mut facts = VueFacts {
        style_count: descriptor.styles.len(),
        script_count: usize::from(descriptor.script.is_some())
            + usize::from(descriptor.script_setup.is_some()),
        ..Default::default()
    };
    if let Some(template) = template {
        let host = TemplateHost::new(descriptor.template.as_ref());
        for node in template.syntax().descendants() {
            match node.kind() {
                kind if kind == RawSyntaxKind::from(VueTemplateKind::StartTag) => {
                    facts.template_element_count += 1;
                    if let Some(fact) = element_fact(&node, &host) {
                        facts.template_elements.push(fact);
                    }
                }
                kind if kind == RawSyntaxKind::from(VueTemplateKind::Directive) => {
                    facts.template_directive_count += 1;
                    if let Some(fact) = directive_fact(&node, &host) {
                        facts.template_directives.push(fact);
                    }
                }
                kind if kind == RawSyntaxKind::from(VueTemplateKind::Interpolation) => {
                    facts.template_interpolation_count += 1;
                    facts.template_expression_count += 1;
                    facts
                        .template_expressions
                        .push(expression_fact(&node, &host));
                }
                kind if kind == RawSyntaxKind::from(VueTemplateKind::DirectiveExpression) => {
                    facts.template_directive_expression_count += 1;
                    facts.template_expression_count += 1;
                    facts
                        .template_expressions
                        .push(expression_fact(&node, &host));
                }
                kind if kind == RawSyntaxKind::from(VueTemplateKind::DirectiveName) => {
                    match classify_directive(&node.text().to_string()) {
                        VueTemplateDirectiveClass::Bind => facts.template_bind_directive_count += 1,
                        VueTemplateDirectiveClass::Event => {
                            facts.template_event_directive_count += 1
                        }
                        VueTemplateDirectiveClass::Slot => facts.template_slot_directive_count += 1,
                        VueTemplateDirectiveClass::Model => {
                            facts.template_model_directive_count += 1
                        }
                        VueTemplateDirectiveClass::Control => {
                            facts.template_control_directive_count += 1
                        }
                        VueTemplateDirectiveClass::Custom => {
                            facts.template_custom_directive_count += 1
                        }
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
                    facts
                        .template_expressions
                        .push(token_expression_fact(&token, &host));
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

fn element_fact(node: &SyntaxNode, host: &TemplateHost<'_>) -> Option<VueTemplateElementFact> {
    let span = node.source_span();
    Some(VueTemplateElementFact {
        name: token_text(node, VueTemplateKind::TagName)?,
        span: host.span(span),
        line: host.line(span.start),
    })
}

fn directive_fact(node: &SyntaxNode, host: &TemplateHost<'_>) -> Option<VueTemplateDirectiveFact> {
    let name = node
        .children()
        .find(|child| child.kind() == RawSyntaxKind::from(VueTemplateKind::DirectiveName))
        .map(|child| child.text().to_string())?;
    let span = node.source_span();
    Some(VueTemplateDirectiveFact {
        class: classify_directive(&name),
        name,
        span: host.span(span),
        line: host.line(span.start),
    })
}

fn expression_fact(node: &SyntaxNode, host: &TemplateHost<'_>) -> VueTemplateExpressionFact {
    let span = node.source_span();
    VueTemplateExpressionFact {
        span: host.span(span),
        line: host.line(span.start),
    }
}

fn token_expression_fact(
    token: &SyntaxToken,
    host: &TemplateHost<'_>,
) -> VueTemplateExpressionFact {
    let span = token.source_span();
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

fn token_text(node: &SyntaxNode, kind: VueTemplateKind) -> Option<String> {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == RawSyntaxKind::from(kind))
        .map(|token| token.text().to_string())
}
