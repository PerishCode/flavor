use crate::{valid_ident, validate, GrammarDocument, GrammarEntry, GrammarError, GrammarSection};

pub fn parse_sidecar(source: &str) -> Result<Vec<GrammarDocument>, Vec<GrammarError>> {
    let value: serde_json::Value = serde_json::from_str(source).map_err(|error| {
        vec![GrammarError {
            line: error.line(),
            message: error.to_string(),
        }]
    })?;
    let Some(grammars) = value.get("grammars").and_then(|value| value.as_array()) else {
        return Err(vec![GrammarError {
            line: 1,
            message: "missing `grammars` array".to_string(),
        }]);
    };

    let mut documents = Vec::new();
    let mut errors = Vec::new();
    for grammar in grammars {
        match parse_sidecar_grammar(grammar) {
            Ok(document) => documents.push(document),
            Err(mut current) => errors.append(&mut current),
        }
    }
    if errors.is_empty() {
        Ok(documents)
    } else {
        Err(errors)
    }
}

pub fn parse_sidecar_validated(source: &str) -> Result<Vec<GrammarDocument>, Vec<GrammarError>> {
    let documents = parse_sidecar(source)?;
    let errors = documents.iter().flat_map(validate).collect::<Vec<_>>();
    if errors.is_empty() {
        Ok(documents)
    } else {
        Err(errors)
    }
}

fn parse_sidecar_grammar(value: &serde_json::Value) -> Result<GrammarDocument, Vec<GrammarError>> {
    let mut errors = Vec::new();
    let name = required_string(value, "id", &mut errors).unwrap_or_default();
    if !name.is_empty() && !valid_ident(&name) {
        errors.push(GrammarError {
            line: 1,
            message: format!("invalid grammar name `{name}`"),
        });
    }

    let mut directives = Vec::new();
    if let Some(object) = value.get("directives").and_then(|value| value.as_object()) {
        for (key, value) in object {
            let Some(value) = value.as_str() else {
                errors.push(GrammarError {
                    line: 1,
                    message: format!("directive `{key}` must be a string"),
                });
                continue;
            };
            directives.push(GrammarEntry {
                key: key.clone(),
                value: value.to_string(),
                line: 1,
            });
        }
    } else {
        errors.push(GrammarError {
            line: 1,
            message: "missing `directives` object".to_string(),
        });
    }

    let sources = parse_sources(value, &mut errors);
    let sections = parse_sections(value, &mut errors);
    if errors.is_empty() {
        Ok(GrammarDocument {
            name,
            sources,
            directives,
            sections,
        })
    } else {
        Err(errors)
    }
}

fn parse_sources(value: &serde_json::Value, errors: &mut Vec<GrammarError>) -> Vec<String> {
    let Some(object) = value.get("sources").and_then(|value| value.as_object()) else {
        errors.push(GrammarError {
            line: 1,
            message: "missing `sources` object".to_string(),
        });
        return Vec::new();
    };
    let mut sources = Vec::new();
    for key in ["lexer", "parser"] {
        match object.get(key).and_then(|value| value.as_str()) {
            Some(source) => sources.push(source.to_string()),
            None => errors.push(GrammarError {
                line: 1,
                message: format!("missing `{key}` source"),
            }),
        }
    }
    sources
}

fn parse_sections(
    value: &serde_json::Value,
    errors: &mut Vec<GrammarError>,
) -> Vec<GrammarSection> {
    let Some(object) = value.get("sections").and_then(|value| value.as_object()) else {
        errors.push(GrammarError {
            line: 1,
            message: "missing `sections` object".to_string(),
        });
        return Vec::new();
    };
    let mut sections = Vec::new();
    for (section_name, entries) in object {
        let Some(entries) = entries.as_object() else {
            errors.push(GrammarError {
                line: 1,
                message: format!("section `{section_name}` must be an object"),
            });
            continue;
        };
        let mut section = GrammarSection {
            name: section_name.clone(),
            line: 1,
            entries: Vec::new(),
        };
        for (key, value) in entries {
            let Some(value) = value.as_str() else {
                errors.push(GrammarError {
                    line: 1,
                    message: format!("entry `{section_name}.{key}` must be a string"),
                });
                continue;
            };
            section.entries.push(GrammarEntry {
                key: key.clone(),
                value: value.to_string(),
                line: 1,
            });
        }
        sections.push(section);
    }
    sections
}

fn required_string(
    value: &serde_json::Value,
    key: &str,
    errors: &mut Vec<GrammarError>,
) -> Option<String> {
    let Some(value) = value.get(key).and_then(|value| value.as_str()) else {
        errors.push(GrammarError {
            line: 1,
            message: format!("missing `{key}` string"),
        });
        return None;
    };
    Some(value.to_string())
}
