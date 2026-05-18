use std::{
    fs,
    path::{Path, PathBuf},
};

use flavor_grammar::{
    parse_contract, parse_contract_values, parse_g4_source_validated, parse_metadata_validated,
    validate_metadata_source_shape, GrammarContractExpectation, GrammarEntryValueExpectation,
    GrammarSectionExpectation,
};

const OLD_METADATA_NAME: &str = concat!("flavor", ".g4", ".json");
const GENERATED_COMMENT: &str = concat!("Generated", " from");

#[test]
fn parses_contract_files() {
    let root = grammar_root();
    let mut count = 0;
    for path in grammar_files(&root) {
        assert_ne!(
            path.file_name().and_then(|value| value.to_str()),
            Some(OLD_METADATA_NAME),
            "{} must use metadata.json for flavor contract metadata",
            path.display()
        );
        assert_ne!(
            path.extension().and_then(|value| value.to_str()),
            Some("grammar"),
            "{} must not remain canonical grammar source",
            path.display()
        );
        if path.file_name().and_then(|value| value.to_str()) != Some("metadata.json") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        let documents = parse_metadata_validated(&source).unwrap_or_else(|errors| {
            panic!("{} parse errors: {errors:?}", path.display());
        });
        for document in documents {
            count += 1;
            let mut sources = Vec::new();
            for source in &document.sources {
                let source_path = path.parent().unwrap().join(source);
                assert!(
                    source_path.exists(),
                    "{} references missing G4 source {source}",
                    path.display()
                );
                let source_text = fs::read_to_string(&source_path).unwrap();
                assert!(
                    !source_text.contains(GENERATED_COMMENT),
                    "{} must be treated as hand-written G4 source",
                    source_path.display()
                );
                assert!(
                    !source_text.contains(OLD_METADATA_NAME),
                    "{} must reference metadata.json, not the old metadata name",
                    source_path.display()
                );
                let g4 = parse_g4_source_validated(&source_text).unwrap_or_else(|errors| {
                    panic!("{} G4 parse errors: {errors:?}", source_path.display());
                });
                sources.push(g4);
            }
            let errors = validate_metadata_source_shape(&document, &sources);
            assert!(
                errors.is_empty(),
                "{} source shape errors: {errors:?}",
                path.display()
            );
        }
    }
    assert!(count >= 7, "expected frontend G4 grammar contracts");
}

#[test]
fn rejects_missing_grammars() {
    let errors = parse_metadata_validated("{}").unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("grammars")));
}

#[test]
fn rejects_bad_fact() {
    let source = sample_metadata("missing mapping", Some("crates/flavor-plugin-sample"));
    let errors = parse_metadata_validated(&source).unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("must declare a `->` mapping")));
}

#[test]
fn rejects_grammar_owned_sections() {
    let source = sample_metadata_with_sections(
        Some("crates/sample"),
        serde_json::json!({
            "facts": {
                "name": "identifier -> SampleName"
            },
            "diagnostics": {
                "parse": "ERROR -> sample/parse"
            },
            "spans": {
                "node": "byte range"
            },
            "recovery": {
                "error": "skip"
            },
            "productions": {
                "source": "identifier"
            }
        }),
    );
    let errors = parse_metadata_validated(&source).unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("belongs in `.g4` grammar source")));
}

#[test]
fn parser_refs_are_defined() {
    for path in grammar_files(&grammar_root()) {
        if path.extension().and_then(|value| value.to_str()) != Some("g4") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        parse_g4_source_validated(&source).unwrap_or_else(|errors| {
            panic!("{} G4 parse errors: {errors:?}", path.display());
        });
    }
}

#[test]
fn checks_expected_contract_keys() {
    let source = sample_metadata("identifier -> SampleName", Some("crates/sample"));
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
    let source = sample_metadata(
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
    let source = sample_metadata("identifier -> SampleName", Some("crates/sample"));
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
    let source = sample_metadata("identifier -> SampleNameFact(line)", Some("crates/sample"));
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

fn grammar_files(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_grammar_files(root, &mut paths);
    paths
}

fn collect_grammar_files(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_grammar_files(&path, paths);
        } else {
            paths.push(path);
        }
    }
}

fn sample_metadata(fact_value: &str, owner: Option<&str>) -> String {
    sample_metadata_with_sections(
        owner,
        serde_json::json!({
            "nodes": {
                "source": "Source"
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
        }),
    )
}

fn sample_metadata_with_sections(owner: Option<&str>, sections: serde_json::Value) -> String {
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
            "sections": sections
        }]
    })
    .to_string()
}
