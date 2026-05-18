use crate::{
    valid_ident, valid_key, validate, GrammarEntry, GrammarError, GrammarMetadata, GrammarSection,
};

pub fn parse_metadata(source: &str) -> Result<Vec<GrammarMetadata>, Vec<GrammarError>> {
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
        match parse_metadata_grammar(grammar) {
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

pub fn parse_metadata_validated(source: &str) -> Result<Vec<GrammarMetadata>, Vec<GrammarError>> {
    let documents = parse_metadata(source)?;
    let errors = documents.iter().flat_map(validate).collect::<Vec<_>>();
    if errors.is_empty() {
        Ok(documents)
    } else {
        Err(errors)
    }
}

fn parse_metadata_grammar(value: &serde_json::Value) -> Result<GrammarMetadata, Vec<GrammarError>> {
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
        Ok(GrammarMetadata {
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
    for (key, value) in object {
        if !valid_ident(key) {
            errors.push(GrammarError {
                line: 1,
                message: format!("invalid source label `{key}`"),
            });
            continue;
        }
        let Some(source) = value.as_str() else {
            errors.push(GrammarError {
                line: 1,
                message: format!("source `{key}` must be a string"),
            });
            continue;
        };
        if !source.ends_with(".g4") {
            errors.push(GrammarError {
                line: 1,
                message: format!("source `{key}` must reference a `.g4` file"),
            });
        }
        sources.push(source.to_string());
    }
    if sources.is_empty() {
        errors.push(GrammarError {
            line: 1,
            message: "metadata must reference at least one `.g4` source".to_string(),
        });
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
        if !valid_ident(section_name) {
            errors.push(GrammarError {
                line: 1,
                message: format!("invalid section name `{section_name}`"),
            });
            continue;
        }
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
            if !valid_key(key) {
                errors.push(GrammarError {
                    line: 1,
                    message: format!("invalid entry key `{section_name}.{key}`"),
                });
                continue;
            }
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
