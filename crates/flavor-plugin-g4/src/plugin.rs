use flavor_core::{
    product, FactPayload, GrammarProduct, PendingDiagnostic, PendingFact, SourceText,
};
use flavor_grammar::{G4GrammarKind, G4Source};

use crate::{run as run_g4, G4PluginConfig};

pub fn satisfy<F>(entrypoint: &F, path: &str, source: &str, products: &mut Vec<GrammarProduct>)
where
    F: Fn(&str) -> Option<&'static str>,
{
    let Some(entrypoint) = entrypoint("g4") else {
        return;
    };

    let output = run_g4(SourceText::new(path, source), G4PluginConfig);
    let diagnostics = output
        .diagnostics
        .into_iter()
        .map(|diagnostic| PendingDiagnostic {
            message: diagnostic.message,
            span: None,
            line: Some(diagnostic.line),
        })
        .collect();
    let facts = output
        .grammar
        .as_ref()
        .map(grammar_facts)
        .unwrap_or_default();

    product(products, "g4", entrypoint, diagnostics, facts);
}

fn grammar_facts(grammar: &G4Source) -> Vec<PendingFact> {
    let mut facts = vec![PendingFact::new(
        "grammar.declaration",
        FactPayload::new()
            .text("name", &grammar.name)
            .text("kind", grammar_kind(grammar.kind)),
    )
    .line(1)];

    facts.extend(grammar.parser_rules.iter().map(|rule| {
        PendingFact::new(
            "grammar.parser_rule",
            FactPayload::new()
                .text("name", &rule.name)
                .usize("references", reference_count(grammar, &rule.name)),
        )
        .line(rule.line)
    }));
    facts.extend(grammar.lexer_tokens.iter().map(|rule| {
        PendingFact::new(
            "grammar.lexer_token",
            FactPayload::new().text("name", &rule.name),
        )
        .line(rule.line)
    }));
    facts.extend(grammar.parser_references.iter().map(|reference| {
        PendingFact::new(
            "grammar.reference",
            FactPayload::new().text("name", &reference.name),
        )
        .line(reference.line)
    }));

    facts
}

fn grammar_kind(kind: G4GrammarKind) -> &'static str {
    match kind {
        G4GrammarKind::Combined => "combined",
        G4GrammarKind::Lexer => "lexer",
        G4GrammarKind::Parser => "parser",
    }
}

fn reference_count(grammar: &G4Source, name: &str) -> usize {
    grammar
        .parser_references
        .iter()
        .filter(|reference| reference.name == name)
        .count()
}
