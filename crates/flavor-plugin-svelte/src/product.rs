use std::collections::BTreeMap;

use flavor_plugin_core::{
    diagnostics, product, FactPayload, GrammarProduct, PendingFact, SourceText,
};
use flavor_plugin_typescript::product as typescript_product;

use crate::{facts::SvelteMarkupNameFact, run as run_svelte, SvelteBlock, SveltePluginConfig};

pub fn satisfy<F>(entrypoint: &F, path: &str, source: &str, products: &mut Vec<GrammarProduct>)
where
    F: Fn(&str) -> Option<&'static str>,
{
    let source_text = SourceText::new(path, source);
    let line_index = source_text.line_index();
    let output = run_svelte(source_text, SveltePluginConfig::default());
    let diagnostics = diagnostics(output.diagnostics, &line_index, 0);
    let line_count = source.lines().count();

    if let Some(svelte_entrypoint) = entrypoint("svelte") {
        let mut facts = Vec::new();
        if let Some(block) = &output.descriptor.module_script {
            push_block_fact("descriptor.module_script", block, &mut facts);
        }
        if let Some(block) = &output.descriptor.script {
            push_block_fact("descriptor.script", block, &mut facts);
        }
        facts.push(PendingFact::new(
            "descriptor.styles",
            FactPayload::new()
                .usize("style_count", output.facts.style_count)
                .usize("style_lines", output.facts.style_lines),
        ));
        facts.push(PendingFact::new(
            "descriptor.markup",
            FactPayload::new()
                .usize("line_count", line_count)
                .usize("script_count", output.facts.script_count)
                .usize("script_lines", output.facts.script_lines)
                .usize("style_count", output.facts.style_count)
                .usize("style_lines", output.facts.style_lines)
                .usize("markup_lines", output.facts.markup_lines)
                .usize("markup_block_count", output.facts.markup_block_count)
                .usize("markup_branch_count", output.facts.markup_branch_count)
                .usize("markup_render_count", output.facts.markup_render_count),
        ));
        for block in output
            .descriptor
            .module_script
            .clone()
            .into_iter()
            .chain(output.descriptor.script.clone())
        {
            push_embedded_script(entrypoint, path, block, &mut facts, products);
        }
        product(products, "svelte", svelte_entrypoint, diagnostics, facts);
    }

    if let Some(markup_entrypoint) = entrypoint("svelte-markup") {
        let mut facts = Vec::new();
        push_markup_facts("markup.block", &output.facts.markup_blocks, &mut facts);
        push_markup_facts("markup.branch", &output.facts.markup_branches, &mut facts);
        push_markup_facts("markup.render", &output.facts.markup_renders, &mut facts);
        product(
            products,
            "svelte-markup",
            markup_entrypoint,
            Vec::new(),
            facts,
        );
    }
}

fn push_block_fact(key: &'static str, block: &SvelteBlock, facts: &mut Vec<PendingFact>) {
    facts.push(
        PendingFact::new(
            key,
            FactPayload::new()
                .text("content", block.content.clone())
                .usize("start_offset", block.start_offset)
                .usize("start_line", block.start_line),
        )
        .line(block.start_line),
    );
}

fn push_embedded_script<F>(
    entrypoint: &F,
    path: &str,
    block: SvelteBlock,
    facts: &mut Vec<PendingFact>,
    products: &mut Vec<GrammarProduct>,
) where
    F: Fn(&str) -> Option<&'static str>,
{
    if block.content.trim().is_empty() {
        return;
    }
    let Some(tsx) = script_tsx(&block.attrs) else {
        return;
    };
    facts.push(
        PendingFact::new(
            "script.embedded",
            FactPayload::new()
                .text("content", block.content.clone())
                .text(
                    "lang",
                    script_lang(&block.attrs).unwrap_or_else(|| "js".to_string()),
                )
                .bool("tsx", tsx)
                .usize("start_offset", block.start_offset)
                .usize("start_line", block.start_line),
        )
        .line(block.start_line),
    );
    typescript_product::satisfy_script(
        entrypoint,
        path,
        &block.content,
        block.start_line,
        tsx,
        products,
    );
}

fn push_markup_facts(
    key: &'static str,
    markup_facts: &[SvelteMarkupNameFact],
    facts: &mut Vec<PendingFact>,
) {
    facts.extend(markup_facts.iter().map(|fact| {
        PendingFact::new(key, FactPayload::new().text("name", fact.name.clone()))
            .span(fact.span)
            .line(fact.line)
    }));
}

fn script_tsx(attrs: &BTreeMap<String, Option<String>>) -> Option<bool> {
    match script_lang(attrs).as_deref() {
        None | Some("js" | "ts") => Some(false),
        Some("jsx" | "tsx") => Some(true),
        _ => None,
    }
}

fn script_lang(attrs: &BTreeMap<String, Option<String>>) -> Option<String> {
    attrs
        .get("lang")
        .and_then(|value| value.as_deref())
        .map(|value| value.to_ascii_lowercase())
}
