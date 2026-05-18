use std::path::Path;

use flavor_core::{diagnostics, product, FactPayload, GrammarProduct, PendingFact, SourceText};
use flavor_shared::product as shared_product;

use crate::{run as run_ts, SourceMode, TsImportSpecifier, TsNameKind, TsPluginConfig};

pub fn satisfy_source<F>(
    entrypoint: &F,
    path: &str,
    source: &str,
    products: &mut Vec<GrammarProduct>,
) where
    F: Fn(&str) -> Option<&'static str>,
{
    let tsx = Path::new(path).extension().and_then(|value| value.to_str()) == Some("tsx");
    satisfy_script(entrypoint, path, source, 0, tsx, products);
}

pub fn satisfy_script<F>(
    entrypoint: &F,
    path: &str,
    source: &str,
    line_offset: usize,
    tsx: bool,
    products: &mut Vec<GrammarProduct>,
) where
    F: Fn(&str) -> Option<&'static str>,
{
    let source_text = SourceText::new(path, source);
    let line_index = source_text.line_index();
    let output = run_ts(source_text, ts_config(tsx));

    if let Some(entrypoint) = entrypoint("typescript") {
        let diagnostics = diagnostics(output.diagnostics.clone(), &line_index, line_offset);
        let mut facts = Vec::new();
        for fact in &output.facts.names {
            let (key, kind) = ts_name_shape(fact.kind);
            facts.push(shared_product::name_fact(
                key,
                fact.name.clone(),
                kind,
                kind,
                fact.span,
                fact.line + line_offset,
            ));
        }
        facts.extend(output.facts.dispatch_branches.iter().map(|fact| {
            shared_product::line_count_fact(
                "dispatch.branch",
                fact.lines,
                fact.span,
                fact.line + line_offset,
            )
        }));
        facts.extend(output.facts.imports.iter().map(|fact| {
            let imports = import_lists(&fact.specifiers);
            PendingFact::new(
                "module.import",
                FactPayload::new()
                    .text("source", fact.source.clone())
                    .bool("type_only", fact.type_only)
                    .texts("default_imports", imports.default)
                    .texts("named_imports", imports.named)
                    .texts("namespace_imports", imports.namespace),
            )
            .span(fact.span)
            .line(fact.line + line_offset)
        }));
        product(products, "typescript", entrypoint, diagnostics, facts);
    }

    if !tsx {
        return;
    }
    let Some(entrypoint) = entrypoint("tsx") else {
        return;
    };
    let facts = output
        .facts
        .jsx_elements
        .into_iter()
        .map(|fact| {
            PendingFact::new(
                if fact.self_closing {
                    "jsx.self_closing"
                } else {
                    "jsx.element"
                },
                FactPayload::new()
                    .text("name", fact.name)
                    .bool("self_closing", fact.self_closing)
                    .text("root", fact.root.unwrap_or_default())
                    .text("intrinsic", fact.intrinsic.unwrap_or_default()),
            )
            .span(fact.span)
            .line(fact.line + line_offset)
        })
        .collect();
    product(products, "tsx", entrypoint, Vec::new(), facts);
}

fn ts_config(tsx: bool) -> TsPluginConfig {
    TsPluginConfig {
        source_mode: if tsx {
            SourceMode::Tsx
        } else {
            SourceMode::TypeScript
        },
        ..Default::default()
    }
}

fn ts_name_shape(kind: TsNameKind) -> (&'static str, &'static str) {
    match kind {
        TsNameKind::Function => ("name.function", "function"),
        TsNameKind::Method => ("name.method", "method"),
        TsNameKind::Binding => ("name.binding", "binding"),
        TsNameKind::Parameter => ("name.parameter", "parameter"),
    }
}

#[derive(Default)]
struct ImportLists {
    default: Vec<String>,
    named: Vec<String>,
    namespace: Vec<String>,
}

fn import_lists(specifiers: &[TsImportSpecifier]) -> ImportLists {
    let mut lists = ImportLists::default();
    for specifier in specifiers {
        match specifier {
            TsImportSpecifier::Default(name) => lists.default.push(name.clone()),
            TsImportSpecifier::Named(name) => lists.named.push(name.clone()),
            TsImportSpecifier::Namespace(name) => lists.namespace.push(name.clone()),
        }
    }
    lists
}
