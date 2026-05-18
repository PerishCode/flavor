mod adapters;

use flavor_core::{Fact, GrammarProduct, ProductDiagnostic};

use super::{PluginManifest, Scope};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ProductSet {
    products: Vec<GrammarProduct>,
}

impl ProductSet {
    pub(crate) fn new(manifest: PluginManifest, scope: Scope<'_>) -> Self {
        Self {
            products: adapters::satisfy(manifest, scope),
        }
    }

    pub(crate) fn facts<'a>(
        &'a self,
        grammar_id: &'static str,
        key: &'static str,
    ) -> impl Iterator<Item = &'a Fact> + 'a {
        self.products.iter().flat_map(move |product| {
            product
                .facts
                .iter()
                .filter(move |fact| product.grammar_id == grammar_id && fact.key == key)
        })
    }

    pub(crate) fn diagnostics<'a>(
        &'a self,
        grammar_id: &'static str,
    ) -> impl Iterator<Item = &'a ProductDiagnostic> + 'a {
        self.products
            .iter()
            .filter(move |product| product.grammar_id == grammar_id)
            .flat_map(|product| product.diagnostics.iter())
    }
}
