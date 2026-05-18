use flavor_core::SourceText;
use flavor_grammar::{
    parse_contract, parse_contract_values, GrammarContractExpectation,
    GrammarEntryValueExpectation, GrammarSectionExpectation,
};
use flavor_plugin_rust::{run, RustNameKind, RustPluginConfig};

const RUST_METADATA: &str = include_str!("../../../grammars/rust/metadata.json");
const RUST_CONTRACT: GrammarContractExpectation<'static> = GrammarContractExpectation {
    name: "rust",
    directives: &[
        ("owner", "crates/flavor-plugin-rust"),
        ("entry", "source_file"),
    ],
    sections: &[
        GrammarSectionExpectation {
            name: "facts",
            entries: &[
                "dispatch.branch",
                "name.binding",
                "name.function",
                "name.method",
                "name.parameter",
                "name.trait_signature",
                "test.attribute",
            ],
        },
        GrammarSectionExpectation {
            name: "diagnostics",
            entries: &["parse.error", "parse.missing"],
        },
        GrammarSectionExpectation {
            name: "spans",
            entries: &["diagnostic.range", "fact.line", "node.line", "node.range"],
        },
        GrammarSectionExpectation {
            name: "recovery",
            entries: &[
                "rule.continuation",
                "tree_sitter.errors",
                "tree_sitter.missing",
            ],
        },
    ],
};
const RUST_VALUES: &[GrammarEntryValueExpectation<'static>] = &[
    GrammarEntryValueExpectation {
        section: "facts",
        key: "name.function",
        contains: &[
            "RustNameFact",
            "payload.name",
            "payload.issue_kind",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "name.method",
        contains: &[
            "RustNameFact",
            "payload.name",
            "payload.issue_kind",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "name.trait_signature",
        contains: &[
            "RustNameFact",
            "payload.name",
            "payload.issue_kind",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "name.binding",
        contains: &[
            "RustNameFact",
            "payload.name",
            "payload.issue_kind",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "name.parameter",
        contains: &[
            "RustNameFact",
            "kind:parameter",
            "payload.name",
            "payload.issue_kind",
            "span",
            "line",
        ],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "dispatch.branch",
        contains: &["RustMatchArmFact", "payload.lines", "span", "line"],
    },
    GrammarEntryValueExpectation {
        section: "facts",
        key: "test.attribute",
        contains: &["RustTestAttributeFact", "span", "line"],
    },
];

#[test]
fn rust_contract_sections() {
    parse_contract_values(RUST_METADATA, &RUST_CONTRACT, RUST_VALUES).unwrap();
}

#[test]
fn rust_contract_facts() {
    parse_contract(RUST_METADATA, &RUST_CONTRACT).unwrap();
    let output = run(
        SourceText::new(
            "contract.rs",
            "#[cfg(test)]\n\
             mod tests {\n\
                 #[test]\n\
                 fn sample_test() {}\n\
             }\n\
             \n\
             trait Repo {\n\
                 fn find(&self, input: i32);\n\
             }\n\
             \n\
             impl Repo for DbRepo {\n\
                 fn find(&self, input: i32) {}\n\
             }\n\
             \n\
             impl DbRepo {\n\
                 fn save(&self, value: i32) {\n\
                     let local_value = value;\n\
                     match local_value {\n\
                         1 => local_value,\n\
                         _ => 0,\n\
                     };\n\
                 }\n\
             }\n",
        ),
        RustPluginConfig::default(),
    );

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let sample_test =
        find_name(&output, RustNameKind::Function, "sample_test", 4).expect("sample_test fact");
    assert_eq!(output.source.slice(sample_test.span), "sample_test");
    let find = find_name(&output, RustNameKind::Method, "find", 8).expect("find fact");
    assert_eq!(output.source.slice(find.span), "find");
    assert_eq!(
        output
            .facts
            .names
            .iter()
            .filter(|name| name.name == "find")
            .count(),
        1
    );
    let save = find_name(&output, RustNameKind::Method, "save", 16).expect("save fact");
    assert_eq!(output.source.slice(save.span), "save");
    assert!(find_name(&output, RustNameKind::Parameter, "value", 16).is_some());
    let local_value =
        find_name(&output, RustNameKind::Binding, "local_value", 17).expect("local_value fact");
    assert_eq!(output.source.slice(local_value.span), "local_value");
    let local_arm = output
        .facts
        .match_arms
        .iter()
        .find(|arm| arm.line == 19 && arm.lines == 1)
        .expect("local_value match arm");
    assert!(output.source.slice(local_arm.span).contains("local_value"));
    let cfg_test = output
        .facts
        .test_attributes
        .iter()
        .find(|attribute| attribute.line == 1)
        .expect("cfg test attribute");
    assert!(output
        .source
        .slice(cfg_test.span)
        .starts_with("#[cfg(test)]"));
    assert!(output
        .facts
        .test_attributes
        .iter()
        .any(|attribute| attribute.line == 3));
}

#[test]
fn rust_contract_diagnostics() {
    parse_contract(RUST_METADATA, &RUST_CONTRACT).unwrap();
    let output = run(
        SourceText::new("broken.rs", "fn broken("),
        RustPluginConfig::default(),
    );

    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message == "invalid Rust syntax")
        .expect("missing invalid Rust syntax diagnostic");
    assert_eq!(diagnostic.code.as_deref(), Some("rust/parse/error"));
    let span = diagnostic.span.expect("diagnostic span");
    assert!(span.start <= span.end);
    assert!(span.end as usize <= "fn broken(".len());
}

fn find_name<'a>(
    output: &'a flavor_plugin_rust::RustAnalysisOutput,
    kind: RustNameKind,
    name: &str,
    line: usize,
) -> Option<&'a flavor_plugin_rust::RustNameFact> {
    output
        .facts
        .names
        .iter()
        .find(|fact| fact.kind == kind && fact.name == name && fact.line == line)
}
