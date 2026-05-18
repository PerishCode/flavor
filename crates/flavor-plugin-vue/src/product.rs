use std::collections::BTreeMap;

use flavor_core::{diagnostics, product, FactPayload, GrammarProduct, PendingFact, SourceText};
use flavor_plugin_typescript::product as typescript_product;

use crate::{run as run_vue, sfc::VueSfcBlock, VuePluginConfig};

pub fn satisfy<F>(entrypoint: &F, path: &str, source: &str, products: &mut Vec<GrammarProduct>)
where
    F: Fn(&str) -> Option<&'static str>,
{
    let Some(vue_entrypoint) = entrypoint("vue-sfc") else {
        return;
    };

    let source_text = SourceText::new(path, source);
    let line_index = source_text.line_index();
    let output = run_vue(source_text, VuePluginConfig::default());
    let diagnostics = diagnostics(output.diagnostics, &line_index, 0);
    let mut facts = Vec::new();
    if let Some(block) = output.descriptor.script {
        push_block_fact("descriptor.script", &block, &mut facts);
        push_embedded_script(entrypoint, path, block, &mut facts, products);
    }
    if let Some(block) = output.descriptor.script_setup {
        push_block_fact("descriptor.script_setup", &block, &mut facts);
        push_embedded_script(entrypoint, path, block, &mut facts, products);
    }
    product(products, "vue-sfc", vue_entrypoint, diagnostics, facts);
}

fn push_block_fact(key: &'static str, block: &VueSfcBlock, facts: &mut Vec<PendingFact>) {
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
    block: VueSfcBlock,
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
