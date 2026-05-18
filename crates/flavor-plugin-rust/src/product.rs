use flavor_plugin_core::{
    diagnostics, product, FactPayload, GrammarProduct, PendingFact, SourceText,
};

use crate::{run as run_rust, RustNameKind, RustPluginConfig};

pub fn satisfy<F>(entrypoint: &F, path: &str, source: &str, products: &mut Vec<GrammarProduct>)
where
    F: Fn(&str) -> Option<&'static str>,
{
    let Some(entrypoint) = entrypoint("rust") else {
        return;
    };

    let source_text = SourceText::new(path, source);
    let line_index = source_text.line_index();
    let output = run_rust(source_text, RustPluginConfig::default());
    let diagnostics = diagnostics(output.diagnostics, &line_index, 0);
    let mut facts = Vec::new();

    for fact in output.facts.names {
        let (key, kind, issue_kind) = rust_name_shape(fact.kind);
        facts.push(
            PendingFact::new(
                key,
                FactPayload::new()
                    .text("name", fact.name)
                    .text("kind", kind)
                    .text("issue_kind", issue_kind),
            )
            .span(fact.span)
            .line(fact.line),
        );
    }

    facts.extend(output.facts.match_arms.into_iter().map(|fact| {
        PendingFact::new(
            "dispatch.branch",
            FactPayload::new().usize("lines", fact.lines),
        )
        .span(fact.span)
        .line(fact.line)
    }));

    facts.extend(output.facts.test_attributes.into_iter().map(|fact| {
        PendingFact::new("test.attribute", FactPayload::new())
            .span(fact.span)
            .line(fact.line)
    }));

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
