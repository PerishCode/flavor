use std::{fs, path::PathBuf};

use flavor_g4::{
    parse_contract, parse_contract_values, parse_sidecar_validated, GrammarContractExpectation,
    GrammarEntryValueExpectation, GrammarSectionExpectation,
};

#[test]
fn parses_contract_files() {
    let root = grammar_root();
    let mut count = 0;
    for path in grammar_files(&root) {
        assert_ne!(
            path.extension().and_then(|value| value.to_str()),
            Some("grammar"),
            "{} must not remain canonical grammar source",
            path.display()
        );
        if path.file_name().and_then(|value| value.to_str()) != Some("flavor.g4.json") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        let documents = parse_sidecar_validated(&source).unwrap_or_else(|errors| {
            panic!("{} parse errors: {errors:?}", path.display());
        });
        for document in documents {
            count += 1;
            for source in document.sources {
                assert!(
                    path.parent().unwrap().join(&source).exists(),
                    "{} references missing G4 source {source}",
                    path.display()
                );
            }
        }
    }
    assert!(count >= 7, "expected frontend G4 grammar contracts");
}

#[test]
fn rejects_missing_grammars() {
    let errors = parse_sidecar_validated("{}").unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("grammars")));
}

#[test]
fn rejects_bad_fact() {
    let source = sample_sidecar("missing mapping", Some("crates/flavor-plugin-sample"));
    let errors = parse_sidecar_validated(&source).unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("must declare a `->` mapping")));
}

#[test]
fn checks_expected_contract_keys() {
    let source = sample_sidecar("identifier -> SampleName", Some("crates/sample"));
    let document = parse_contract(
        &source,
        &GrammarContractExpectation {
            name: "sample",
            directives: &[("owner", "crates/sample")],
            sections: &[
                GrammarSectionExpectation {
                    name: "facts",
                    entries: &["name"],
                },
                GrammarSectionExpectation {
                    name: "diagnostics",
                    entries: &["parse"],
                },
            ],
        },
    )
    .unwrap();

    assert_eq!(document.name, "sample");
}

#[test]
fn checks_expected_values() {
    let source = sample_sidecar(
        "identifier -> SampleNameFact(span, line)",
        Some("crates/sample"),
    );
    let document = parse_contract_values(
        &source,
        &GrammarContractExpectation {
            name: "sample",
            directives: &[("owner", "crates/sample")],
            sections: &[GrammarSectionExpectation {
                name: "facts",
                entries: &["name"],
            }],
        },
        &[GrammarEntryValueExpectation {
            section: "facts",
            key: "name",
            contains: &["SampleNameFact", "span", "line"],
        }],
    )
    .unwrap();

    assert_eq!(document.name, "sample");
}

#[test]
fn rejects_unaligned_contract_keys() {
    let source = sample_sidecar("identifier -> SampleName", Some("crates/sample"));
    let errors = parse_contract(
        &source,
        &GrammarContractExpectation {
            name: "sample",
            directives: &[("owner", "crates/sample")],
            sections: &[GrammarSectionExpectation {
                name: "facts",
                entries: &["name", "missing"],
            }],
        },
    )
    .unwrap_err();

    assert!(errors
        .iter()
        .any(|error| error.message.contains("missing `missing` entry")));
}

#[test]
fn rejects_value_drift() {
    let source = sample_sidecar("identifier -> SampleNameFact(line)", Some("crates/sample"));
    let errors = parse_contract_values(
        &source,
        &GrammarContractExpectation {
            name: "sample",
            directives: &[("owner", "crates/sample")],
            sections: &[GrammarSectionExpectation {
                name: "facts",
                entries: &["name"],
            }],
        },
        &[GrammarEntryValueExpectation {
            section: "facts",
            key: "name",
            contains: &["span"],
        }],
    )
    .unwrap_err();

    assert!(errors
        .iter()
        .any(|error| error.message.contains("facts.name")));
}

fn grammar_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("grammars")
}

fn grammar_files(root: &std::path::Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_grammar_files(root, &mut paths);
    paths
}

fn collect_grammar_files(root: &std::path::Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_grammar_files(&path, paths);
        } else {
            paths.push(path);
        }
    }
}

fn sample_sidecar(fact_value: &str, owner: Option<&str>) -> String {
    let mut directives = serde_json::Map::new();
    directives.insert("entry".to_string(), serde_json::json!("source"));
    if let Some(owner) = owner {
        directives.insert("owner".to_string(), serde_json::json!(owner));
    }
    serde_json::json!({
        "bundle": "sample",
        "grammars": [{
            "id": "sample",
            "sources": {
                "lexer": "SampleLexer.g4",
                "parser": "SampleParser.g4"
            },
            "directives": directives,
            "sections": {
                "tokens": {
                    "identifier": "scanner:identifier"
                },
                "nodes": {
                    "source": "Source"
                },
                "productions": {
                    "source": "identifier"
                },
                "facts": {
                    "name": fact_value
                },
                "diagnostics": {
                    "parse": "ERROR -> sample/parse"
                },
                "spans": {
                    "node": "byte range"
                },
                "recovery": {
                    "error": "skip"
                }
            }
        }]
    })
    .to_string()
}
