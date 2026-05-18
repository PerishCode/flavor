use flavor_plugin_core::GrammarProduct;

use crate::{
    config::SourceKind,
    plugins::{PluginManifest, Scope},
};

pub(super) fn satisfy(manifest: PluginManifest, scope: Scope<'_>) -> Vec<GrammarProduct> {
    let mut products = Vec::new();
    let Some(source) = scope.source_file_data() else {
        return products;
    };

    match source.kind {
        SourceKind::Rust => flavor_plugin_rust::product::satisfy(
            &|grammar_id| entrypoint(manifest, grammar_id),
            source.path,
            source.source,
            &mut products,
        ),
        SourceKind::TypeScript => flavor_plugin_typescript::product::satisfy_source(
            &|grammar_id| entrypoint(manifest, grammar_id),
            source.path,
            source.source,
            &mut products,
        ),
        SourceKind::Vue => flavor_plugin_vue::product::satisfy(
            &|grammar_id| entrypoint(manifest, grammar_id),
            source.path,
            source.source,
            &mut products,
        ),
        SourceKind::Svelte => flavor_plugin_svelte::product::satisfy(
            &|grammar_id| entrypoint(manifest, grammar_id),
            source.path,
            source.source,
            &mut products,
        ),
    }

    products
}

fn entrypoint(manifest: PluginManifest, grammar_id: &str) -> Option<&'static str> {
    manifest
        .grammars
        .iter()
        .find(|grammar| grammar.grammar_id == grammar_id)
        .map(|grammar| grammar.entrypoint)
}
