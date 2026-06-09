use std::collections::BTreeSet;

mod markup;
mod metadata;
mod parse;
mod raw_builder;
mod schema;
mod source;
#[cfg(feature = "tree-sitter-backend")]
mod tree_sitter_raw;
mod view;
mod vue;

pub use markup::{
    find_balanced_brace_close, find_html_comment_close, is_html_void_element, is_markup_name_char,
    markup_char_at, scan_markup_name,
};
pub use metadata::{parse_metadata, parse_metadata_validated};
pub use parse::GrammarParseOutput;
pub use raw_builder::RawAstBuilder;
pub use schema::{
    GrammarBundle, GrammarKindName, GrammarSpec, RawAstSchema, RawAstSymbol, RawAstSymbolKind,
};
pub use source::{
    parse_g4_source, parse_g4_source_validated, validate_g4_source, G4Binding, G4GrammarKind,
    G4Reference, G4Rule, G4Source,
};
#[cfg(feature = "tree-sitter-backend")]
pub use tree_sitter_raw::{
    parse_tree_sitter, tree_sitter_error_span, TreeSitterParseConfig, TreeSitterRawAstAdapter,
};
pub use view::{GrammarContext, GrammarNode, GrammarToken, GrammarTree, TokenTextRun};
pub use vue::{parse_vue_sfc, parse_vue_template, VueSfcBlock, VueSfcDescriptor, VueSfcError};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GrammarMetadata {
    pub name: String,
    pub sources: Vec<String>,
    pub directives: Vec<GrammarEntry>,
    pub sections: Vec<GrammarSection>,
}

impl GrammarMetadata {
    pub fn section(&self, name: &str) -> Option<&GrammarSection> {
        self.sections.iter().find(|section| section.name == name)
    }

    pub fn directive(&self, key: &str) -> Option<&GrammarEntry> {
        self.directives.iter().find(|entry| entry.key == key)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GrammarSection {
    pub name: String,
    pub line: usize,
    pub entries: Vec<GrammarEntry>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GrammarEntry {
    pub key: String,
    pub value: String,
    pub line: usize,
}

impl GrammarSection {
    pub fn entry(&self, key: &str) -> Option<&GrammarEntry> {
        self.entries.iter().find(|entry| entry.key == key)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GrammarError {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct GrammarContractExpectation<'a> {
    pub name: &'a str,
    pub directives: &'a [(&'a str, &'a str)],
    pub sections: &'a [GrammarSectionExpectation<'a>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct GrammarSectionExpectation<'a> {
    pub name: &'a str,
    pub entries: &'a [&'a str],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct GrammarEntryValueExpectation<'a> {
    pub section: &'a str,
    pub key: &'a str,
    pub contains: &'a [&'a str],
}

pub fn parse_contract(
    source: &str,
    expectation: &GrammarContractExpectation<'_>,
) -> Result<GrammarMetadata, Vec<GrammarError>> {
    let document = parse_metadata_validated(source)?
        .into_iter()
        .find(|document| document.name == expectation.name)
        .ok_or_else(|| {
            vec![GrammarError {
                line: 1,
                message: format!("missing metadata for grammar `{}`", expectation.name),
            }]
        })?;
    let errors = validate_contract(&document, expectation);
    if errors.is_empty() {
        Ok(document)
    } else {
        Err(errors)
    }
}

pub fn parse_contract_values(
    source: &str,
    expectation: &GrammarContractExpectation<'_>,
    values: &[GrammarEntryValueExpectation<'_>],
) -> Result<GrammarMetadata, Vec<GrammarError>> {
    let document = parse_contract(source, expectation)?;
    let errors = validate_entry_values(&document, values);
    if errors.is_empty() {
        Ok(document)
    } else {
        Err(errors)
    }
}

pub fn validate(document: &GrammarMetadata) -> Vec<GrammarError> {
    let mut errors = Vec::new();
    if document.sources.is_empty() {
        errors.push(GrammarError {
            line: 1,
            message: "missing `.g4` sources".to_string(),
        });
    }
    let mut sections = BTreeSet::new();
    for section in &document.sections {
        if !sections.insert(section.name.as_str()) {
            errors.push(GrammarError {
                line: section.line,
                message: format!("duplicate section `{}`", section.name),
            });
        }
        if section.entries.is_empty() {
            errors.push(GrammarError {
                line: section.line,
                message: format!("section `{}` must not be empty", section.name),
            });
        }
        if matches!(section.name.as_str(), "tokens" | "productions") {
            errors.push(GrammarError {
                line: section.line,
                message: format!(
                    "section `{}` belongs in `.g4` grammar source, not metadata",
                    section.name
                ),
            });
        }
        validate_entries(section, &mut errors);
        validate_mapping(section, &mut errors);
    }

    for required in ["facts", "diagnostics", "spans", "recovery"] {
        if document.section(required).is_none() {
            errors.push(GrammarError {
                line: 1,
                message: format!("missing required section `{required}`"),
            });
        }
    }

    for required in ["entry", "owner"] {
        if !document
            .directives
            .iter()
            .any(|entry| entry.key == required)
        {
            errors.push(GrammarError {
                line: 1,
                message: format!("missing required `{required}` directive"),
            });
        }
    }

    errors
}

pub fn validate_metadata_source_shape(
    document: &GrammarMetadata,
    sources: &[G4Source],
) -> Vec<GrammarError> {
    let mut errors = Vec::new();
    if let Some(entry) = document.directive("entry") {
        if !sources
            .iter()
            .any(|source| source.defines_parser_rule(&entry.value))
        {
            errors.push(GrammarError {
                line: entry.line,
                message: format!(
                    "entry `{}` is not defined by referenced G4 parser sources",
                    entry.value
                ),
            });
        }
    }

    if let Some(nodes) = document.section("nodes") {
        for node in &nodes.entries {
            if !sources
                .iter()
                .any(|source| source.defines_raw_ast_symbol(&node.key))
            {
                errors.push(GrammarError {
                    line: node.line,
                    message: format!(
                        "metadata node `{}` is not defined by referenced G4 sources",
                        node.key
                    ),
                });
            }
        }
    }
    errors
}

pub fn validate_contract(
    document: &GrammarMetadata,
    expectation: &GrammarContractExpectation<'_>,
) -> Vec<GrammarError> {
    let mut errors = Vec::new();
    if document.name != expectation.name {
        errors.push(GrammarError {
            line: 1,
            message: format!(
                "expected grammar `{}`, found `{}`",
                expectation.name, document.name
            ),
        });
    }

    for (key, value) in expectation.directives {
        match document.directive(key) {
            Some(entry) if entry.value == *value => {}
            Some(entry) => errors.push(GrammarError {
                line: entry.line,
                message: format!(
                    "expected directive `{key} = {value}`, found `{}`",
                    entry.value
                ),
            }),
            None => errors.push(GrammarError {
                line: 1,
                message: format!("missing directive `{key} = {value}`"),
            }),
        }
    }

    for expected in expectation.sections {
        let Some(section) = document.section(expected.name) else {
            errors.push(GrammarError {
                line: 1,
                message: format!("missing section `{}`", expected.name),
            });
            continue;
        };
        validate_section_expectation(section, expected, &mut errors);
    }

    errors
}

pub fn validate_entry_values(
    document: &GrammarMetadata,
    values: &[GrammarEntryValueExpectation<'_>],
) -> Vec<GrammarError> {
    let mut errors = Vec::new();
    for expected in values {
        let Some(section) = document.section(expected.section) else {
            errors.push(GrammarError {
                line: 1,
                message: format!("missing section `{}`", expected.section),
            });
            continue;
        };
        let Some(entry) = section.entry(expected.key) else {
            errors.push(GrammarError {
                line: section.line,
                message: format!("missing `{}` entry in `{}`", expected.key, section.name),
            });
            continue;
        };
        for fragment in expected.contains {
            if !entry.value.contains(fragment) {
                errors.push(GrammarError {
                    line: entry.line,
                    message: format!(
                        "expected `{}.{}` to contain `{fragment}`",
                        section.name, entry.key
                    ),
                });
            }
        }
    }
    errors
}

fn validate_section_expectation(
    section: &GrammarSection,
    expected: &GrammarSectionExpectation<'_>,
    errors: &mut Vec<GrammarError>,
) {
    let expected_entries = expected.entries.iter().copied().collect::<BTreeSet<_>>();
    let actual_entries = section
        .entries
        .iter()
        .map(|entry| entry.key.as_str())
        .collect::<BTreeSet<_>>();

    for key in expected_entries.difference(&actual_entries) {
        errors.push(GrammarError {
            line: section.line,
            message: format!("missing `{}` entry in `{}`", key, section.name),
        });
    }
    for key in actual_entries.difference(&expected_entries) {
        let line = section
            .entry(key)
            .map(|entry| entry.line)
            .unwrap_or(section.line);
        errors.push(GrammarError {
            line,
            message: format!("unexpected `{}` entry in `{}`", key, section.name),
        });
    }
}

fn validate_entries(section: &GrammarSection, errors: &mut Vec<GrammarError>) {
    let mut keys = BTreeSet::new();
    for entry in &section.entries {
        if !valid_key(&entry.key) {
            errors.push(GrammarError {
                line: entry.line,
                message: format!("invalid `{}` entry in `{}`", entry.key, section.name),
            });
        }
        if !keys.insert(entry.key.as_str()) {
            errors.push(GrammarError {
                line: entry.line,
                message: format!("duplicate `{}` entry in `{}`", entry.key, section.name),
            });
        }
    }
}

fn validate_mapping(section: &GrammarSection, errors: &mut Vec<GrammarError>) {
    if !matches!(section.name.as_str(), "facts" | "diagnostics") {
        return;
    }
    for entry in &section.entries {
        if !entry.value.contains("->") {
            errors.push(GrammarError {
                line: entry.line,
                message: format!(
                    "`{}` entry `{}` must declare a `->` mapping",
                    section.name, entry.key
                ),
            });
        }
    }
}

fn valid_ident(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn valid_key(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/'))
}
