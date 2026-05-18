use flavor_core::SourceText;
use flavor_plugin_g4::{run, G4PluginConfig};

#[test]
fn run_collects_g4_shape() {
    let output = run(
        SourceText::new(
            "SampleParser.g4",
            r#"
parser grammar SampleParser;
source_file: item EOF;
item: IDENTIFIER;
"#,
        ),
        G4PluginConfig,
    );

    let grammar = output.grammar.expect("parsed grammar");
    assert_eq!(grammar.name, "SampleParser");
    assert!(grammar.defines_parser_rule("source_file"));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn reports_bad_g4_shape() {
    let output = run(
        SourceText::new(
            "SampleParser.g4",
            r#"
parser grammar SampleParser;
source_file: missing_rule EOF;
"#,
        ),
        G4PluginConfig,
    );

    assert!(output.grammar.is_none());
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("missing_rule")));
}
