use flavor_plugin_core::{LineIndex, RawSyntaxKind, Span, SyntaxNode, SyntaxSpanExt};

use crate::{
    descriptor::{SvelteBlock, SvelteDescriptor},
    markup::{SvelteMarkupAst, SvelteMarkupKind},
};

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct SvelteFacts {
    pub script_count: usize,
    pub style_count: usize,
    pub script_lines: usize,
    pub style_lines: usize,
    pub markup_lines: usize,
    pub markup_element_count: usize,
    pub markup_component_count: usize,
    pub markup_block_count: usize,
    pub markup_branch_count: usize,
    pub markup_render_count: usize,
    pub markup_special_count: usize,
    pub markup_directive_count: usize,
    pub markup_elements: Vec<SvelteMarkupNameFact>,
    pub markup_components: Vec<SvelteMarkupNameFact>,
    pub markup_blocks: Vec<SvelteMarkupNameFact>,
    pub markup_branches: Vec<SvelteMarkupNameFact>,
    pub markup_renders: Vec<SvelteMarkupNameFact>,
    pub markup_specials: Vec<SvelteMarkupNameFact>,
    pub markup_directives: Vec<SvelteMarkupNameFact>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SvelteMarkupNameFact {
    pub name: String,
    pub span: Span,
    pub line: usize,
}

pub fn collect(descriptor: &SvelteDescriptor, markup: Option<&SvelteMarkupAst>) -> SvelteFacts {
    let mut facts = SvelteFacts {
        script_count: usize::from(descriptor.module_script.is_some())
            + usize::from(descriptor.script.is_some()),
        style_count: descriptor.styles.len(),
        script_lines: descriptor
            .module_script
            .iter()
            .chain(descriptor.script.iter())
            .map(block_lines)
            .sum(),
        style_lines: descriptor.styles.iter().map(block_lines).sum(),
        markup_lines: descriptor
            .markup
            .content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
        ..Default::default()
    };
    if let Some(markup) = markup {
        let context = MarkupFactContext::new(markup);
        for node in markup.syntax().descendants() {
            match node.kind() {
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::Element) => {
                    facts.markup_element_count += 1;
                    if let Some(fact) = element_fact(&node, &context) {
                        facts.markup_elements.push(fact);
                    }
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::Component) => {
                    facts.markup_component_count += 1;
                    if let Some(fact) = element_fact(&node, &context) {
                        facts.markup_components.push(fact);
                    }
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::Block) => {
                    facts.markup_block_count += 1;
                    if let Some(fact) = keyword_fact(&node, &context) {
                        facts.markup_blocks.push(fact);
                    }
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::BlockBranch) => {
                    facts.markup_branch_count += 1;
                    if let Some(fact) = keyword_fact(&node, &context) {
                        facts.markup_branches.push(fact);
                    }
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::RenderTag) => {
                    facts.markup_render_count += 1;
                    if let Some(fact) = keyword_fact(&node, &context) {
                        facts.markup_renders.push(fact);
                    }
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::SpecialTag) => {
                    facts.markup_special_count += 1;
                    if let Some(fact) = keyword_fact(&node, &context) {
                        facts.markup_specials.push(fact);
                    }
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::Directive) => {
                    facts.markup_directive_count += 1;
                    if let Some(fact) = directive_fact(&node, &context) {
                        facts.markup_directives.push(fact);
                    }
                }
                _ => {}
            }
        }
    }
    facts
}

fn block_lines(block: &SvelteBlock) -> usize {
    block.content.lines().count()
}

struct MarkupFactContext {
    line_index: LineIndex,
}

impl MarkupFactContext {
    fn new(markup: &SvelteMarkupAst) -> Self {
        let text = markup.syntax().text().to_string();
        Self {
            line_index: LineIndex::new(&text),
        }
    }

    fn line(&self, span: Span) -> usize {
        self.line_index.line(span.start)
    }
}

fn element_fact(node: &SyntaxNode, context: &MarkupFactContext) -> Option<SvelteMarkupNameFact> {
    let span = node.source_span();
    Some(SvelteMarkupNameFact {
        name: token_text(node, SvelteMarkupKind::TagName)?,
        span,
        line: context.line(span),
    })
}

fn keyword_fact(node: &SyntaxNode, context: &MarkupFactContext) -> Option<SvelteMarkupNameFact> {
    let span = node.source_span();
    Some(SvelteMarkupNameFact {
        name: token_text(node, SvelteMarkupKind::BlockKeyword)?,
        span,
        line: context.line(span),
    })
}

fn directive_fact(node: &SyntaxNode, context: &MarkupFactContext) -> Option<SvelteMarkupNameFact> {
    let name = node
        .children()
        .find(|child| child.kind() == RawSyntaxKind::from(SvelteMarkupKind::DirectiveName))
        .map(|child| child.text().to_string())?;
    let span = node.source_span();
    Some(SvelteMarkupNameFact {
        name,
        span,
        line: context.line(span),
    })
}

fn token_text(node: &SyntaxNode, kind: SvelteMarkupKind) -> Option<String> {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == RawSyntaxKind::from(kind))
        .map(|token| token.text().to_string())
}
