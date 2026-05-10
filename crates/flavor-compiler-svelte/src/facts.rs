use flavor_compiler_core::RawSyntaxKind;

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
        for node in markup.syntax().descendants() {
            match node.kind() {
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::StartTag) => {
                    facts.markup_element_count += 1;
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::Component) => {
                    facts.markup_component_count += 1;
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::Block) => {
                    facts.markup_block_count += 1;
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::BlockBranch) => {
                    facts.markup_branch_count += 1;
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::RenderTag) => {
                    facts.markup_render_count += 1;
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::SpecialTag) => {
                    facts.markup_special_count += 1;
                }
                kind if kind == RawSyntaxKind::from(SvelteMarkupKind::Directive) => {
                    facts.markup_directive_count += 1;
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
