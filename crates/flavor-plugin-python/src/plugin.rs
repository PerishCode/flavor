use flavor_core::{diagnostics, product, FactPayload, GrammarProduct, PendingFact, SourceText};
use flavor_shared::product as shared_product;

use crate::{internal::grammar, run, PythonNameKind};

pub fn prewarm() {
    let _ = grammar::bundle();
}

pub fn satisfy<F>(entrypoint: &F, path: &str, source: &str, products: &mut Vec<GrammarProduct>)
where
    F: Fn(&str) -> Option<&'static str>,
{
    let Some(entrypoint) = entrypoint("python") else {
        return;
    };

    let source_text = SourceText::new(path, source);
    let line_index = source_text.line_index();
    let output = run(source_text);
    let diagnostics = diagnostics(output.diagnostics, &line_index, 0);
    let mut facts = Vec::new();

    for fact in output.facts.names {
        let (key, kind, issue_kind) = python_name_shape(fact.kind);
        facts.push(shared_product::name_fact(
            key, fact.name, kind, issue_kind, fact.span, fact.line,
        ));
    }

    facts.extend(output.facts.dispatch_branches.into_iter().map(|fact| {
        shared_product::line_count_fact("dispatch.branch", fact.lines, fact.span, fact.line)
    }));

    facts.extend(output.facts.function_bodies.into_iter().map(|fact| {
        PendingFact::new(
            "function.body",
            FactPayload::new()
                .text("name", fact.name)
                .text("kind", fact.kind)
                .usize("lines", fact.lines),
        )
        .span(fact.span)
        .line(fact.line)
    }));

    product(products, "python", entrypoint, diagnostics, facts);
}

fn python_name_shape(kind: PythonNameKind) -> (&'static str, &'static str, &'static str) {
    match kind {
        PythonNameKind::Function => ("name.function", "function", "function"),
        PythonNameKind::Method => ("name.method", "method", "method"),
        PythonNameKind::Binding => ("name.binding", "binding", "binding"),
        PythonNameKind::Parameter => ("name.parameter", "parameter", "binding"),
    }
}
