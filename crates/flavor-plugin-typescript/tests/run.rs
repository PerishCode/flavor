use flavor_core::SourceText;
use flavor_plugin_typescript::{run, SourceMode, TsPluginConfig};

#[path = "../src/internal/grammar.rs"]
mod kind;

#[test]
fn run_accepts_injected_config() {
    let config = TsPluginConfig {
        source_mode: SourceMode::Tsx,
        ..Default::default()
    };
    let output = run(
        SourceText::new("sample.tsx", "const node = <div />;"),
        config,
    );

    assert_eq!(output.source.name(), "sample.tsx");
    assert_eq!(
        output.tokens.last().map(|token| token.kind),
        Some(kind::END_OF_FILE)
    );
    assert!(output.diagnostics.is_empty());
}

#[test]
fn run_collects_module_facts() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "import value from './x'; export const out = value;",
        ),
        TsPluginConfig::default(),
    );

    assert_eq!(output.facts.import_count, 1);
    assert_eq!(output.facts.export_count, 1);
    assert!(output.diagnostics.is_empty());
}
