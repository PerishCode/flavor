use std::collections::BTreeMap;

use flavor_core::{diagnostics, product, GrammarProduct, PendingFact, SourceText};
use flavor_plugin_typescript::{plugin as typescript_plugin, SourceMode, TsPluginConfig};
use flavor_shared::product as shared_product;

use crate::{run as run_vue, sfc::VueSfcBlock, template, VuePluginConfig};

pub fn prewarm() {
    let _ = template::kind::bundle();
    typescript_plugin::prewarm();
}

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
    facts.push(shared_product::descriptor_block_fact(
        key,
        block.content.clone(),
        block.start_offset,
        block.start_line,
    ));
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
    facts.push(shared_product::embedded_script_fact(
        "script.embedded",
        block.content.clone(),
        script_lang(&block.attrs).unwrap_or_else(|| "js".to_string()),
        tsx,
        block.start_offset,
        block.start_line,
    ));
    let config = TsPluginConfig {
        source_mode: if tsx {
            SourceMode::Tsx
        } else {
            SourceMode::TypeScript
        },
        ..Default::default()
    };
    typescript_plugin::satisfy_script_with_config(
        entrypoint,
        path,
        &block.content,
        block.start_line,
        tsx,
        config,
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
