use flavor_core::{diagnostics, product, FactPayload, GrammarProduct, PendingFact, SourceText};
use flavor_shared::product as shared_product;

use crate::{run as run_rust, RustNameKind, RustPluginConfig};

pub fn satisfy<F>(entrypoint: &F, path: &str, source: &str, products: &mut Vec<GrammarProduct>)
where
    F: Fn(&str) -> Option<&'static str>,
{
    satisfy_with_config(
        entrypoint,
        path,
        source,
        RustPluginConfig::default(),
        products,
    );
}

pub fn satisfy_with_config<F>(
    entrypoint: &F,
    path: &str,
    source: &str,
    config: RustPluginConfig,
    products: &mut Vec<GrammarProduct>,
) where
    F: Fn(&str) -> Option<&'static str>,
{
    let Some(entrypoint) = entrypoint("rust") else {
        return;
    };

    let source_text = SourceText::new(path, source);
    let line_index = source_text.line_index();
    let output = run_rust(source_text, config);
    let diagnostics = diagnostics(output.diagnostics, &line_index, 0);
    let mut facts = Vec::new();

    for fact in output.facts.names {
        let (key, kind, issue_kind) = rust_name_shape(fact.kind);
        facts.push(shared_product::name_fact(
            key, fact.name, kind, issue_kind, fact.span, fact.line,
        ));
    }

    facts.extend(output.facts.match_arms.into_iter().map(|fact| {
        shared_product::line_count_fact("dispatch.branch", fact.lines, fact.span, fact.line)
    }));

    facts.extend(output.facts.test_attributes.into_iter().map(|fact| {
        PendingFact::new("test.attribute", FactPayload::new())
            .span(fact.span)
            .line(fact.line)
    }));
    facts.extend(
        output
            .facts
            .repeated_token_patterns
            .into_iter()
            .map(|fact| {
                PendingFact::new(
                    "shape.repeated_token_pattern",
                    FactPayload::new()
                        .usize("occurrences", fact.occurrences)
                        .usize("total_lines", fact.total_lines)
                        .usize("token_count", fact.token_count)
                        .usize("node_kind", usize::from(fact.node_kind))
                        .usize("depth", fact.depth),
                )
                .span(fact.span)
                .line(fact.line)
            }),
    );

    product(products, "rust", entrypoint, diagnostics, facts);
}

fn rust_name_shape(kind: RustNameKind) -> (&'static str, &'static str, &'static str) {
    match kind {
        RustNameKind::Function => ("name.function", "function", "function"),
        RustNameKind::Method => ("name.method", "method", "method"),
        RustNameKind::Binding => ("name.binding", "binding", "binding"),
        // Preserve the historical CLI message label for Rust parameters while
        // still exposing the more precise product fact key.
        RustNameKind::Parameter => ("name.parameter", "parameter", "binding"),
    }
}
