use std::collections::BTreeSet;

mod sidecar;

pub use sidecar::{parse_sidecar, parse_sidecar_validated};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GrammarDocument {
    pub name: String,
    pub sources: Vec<String>,
    pub directives: Vec<GrammarEntry>,
    pub sections: Vec<GrammarSection>,
}

impl GrammarDocument {
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

pub fn parse(source: &str) -> Result<GrammarDocument, Vec<GrammarError>> {
    let mut parser = Parser {
        lines: source.lines().enumerate(),
        errors: Vec::new(),
        document: None,
        current: None,
    };
    parser.parse();
    if parser.errors.is_empty() {
        Ok(parser.document.expect("parser creates document or error"))
    } else {
        Err(parser.errors)
    }
}

pub fn parse_validated(source: &str) -> Result<GrammarDocument, Vec<GrammarError>> {
    let document = parse(source)?;
    let errors = validate(&document);
    if errors.is_empty() {
        Ok(document)
    } else {
        Err(errors)
    }
}

pub fn parse_contract(
    source: &str,
    expectation: &GrammarContractExpectation<'_>,
) -> Result<GrammarDocument, Vec<GrammarError>> {
    let document = if source.trim_start().starts_with('{') {
        parse_sidecar_validated(source)?
            .into_iter()
            .find(|document| document.name == expectation.name)
            .ok_or_else(|| {
                vec![GrammarError {
                    line: 1,
                    message: format!("missing grammar `{}`", expectation.name),
                }]
            })?
    } else {
        parse_validated(source)?
    };
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
) -> Result<GrammarDocument, Vec<GrammarError>> {
    let document = parse_contract(source, expectation)?;
    let errors = validate_entry_values(&document, values);
    if errors.is_empty() {
        Ok(document)
    } else {
        Err(errors)
    }
}

pub fn validate(document: &GrammarDocument) -> Vec<GrammarError> {
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
        validate_entries(section, &mut errors);
        validate_mapping(section, &mut errors);
    }

    for required in [
        "tokens",
        "nodes",
        "productions",
        "facts",
        "diagnostics",
        "spans",
        "recovery",
    ] {
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

pub fn validate_contract(
    document: &GrammarDocument,
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
    document: &GrammarDocument,
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

struct Parser<'a> {
    lines: std::iter::Enumerate<std::str::Lines<'a>>,
    errors: Vec<GrammarError>,
    document: Option<GrammarDocument>,
    current: Option<GrammarSection>,
}

impl Parser<'_> {
    fn parse(&mut self) {
        while let Some((index, raw)) = self.lines.next() {
            let line = index + 1;
            let text = raw.trim();
            if text.is_empty() || text.starts_with('#') {
                continue;
            }

            if self.document.is_none() {
                self.parse_header(line, text);
                continue;
            }

            if text == "}" {
                self.close_section(line);
            } else if text.ends_with('{') {
                self.open_section(line, text);
            } else if self.current.is_some() {
                self.parse_entry(line, text, true);
            } else {
                self.parse_entry(line, text, false);
            }
        }

        if let Some(section) = self.current.take() {
            self.errors.push(GrammarError {
                line: section.line,
                message: format!("section `{}` is missing closing `}}`", section.name),
            });
        }
        if self.document.is_none() {
            self.errors.push(GrammarError {
                line: 1,
                message: "missing `grammar <name>;` header".to_string(),
            });
        }
    }

    fn parse_header(&mut self, line: usize, text: &str) {
        let Some(rest) = text
            .strip_prefix("grammar ")
            .and_then(|value| value.strip_suffix(';'))
        else {
            self.errors.push(GrammarError {
                line,
                message: "expected `grammar <name>;` header".to_string(),
            });
            return;
        };
        let name = rest.trim();
        if !valid_ident(name) {
            self.errors.push(GrammarError {
                line,
                message: format!("invalid grammar name `{name}`"),
            });
            return;
        }
        self.document = Some(GrammarDocument {
            name: name.to_string(),
            sources: Vec::new(),
            directives: Vec::new(),
            sections: Vec::new(),
        });
    }

    fn open_section(&mut self, line: usize, text: &str) {
        if self.current.is_some() {
            self.errors.push(GrammarError {
                line,
                message: "nested sections are not allowed".to_string(),
            });
            return;
        }
        let name = text.trim_end_matches('{').trim();
        if !valid_ident(name) {
            self.errors.push(GrammarError {
                line,
                message: format!("invalid section name `{name}`"),
            });
            return;
        }
        self.current = Some(GrammarSection {
            name: name.to_string(),
            line,
            entries: Vec::new(),
        });
    }

    fn close_section(&mut self, line: usize) {
        let Some(section) = self.current.take() else {
            self.errors.push(GrammarError {
                line,
                message: "unexpected closing `}`".to_string(),
            });
            return;
        };
        self.document
            .as_mut()
            .expect("section requires document")
            .sections
            .push(section);
    }

    fn parse_entry(&mut self, line: usize, text: &str, section_entry: bool) {
        let Some(entry) = parse_entry(line, text) else {
            self.errors.push(GrammarError {
                line,
                message: "expected `<key> = <value>;` entry".to_string(),
            });
            return;
        };
        if section_entry {
            self.current
                .as_mut()
                .expect("section entry requires current section")
                .entries
                .push(entry);
        } else {
            self.document
                .as_mut()
                .expect("directive requires document")
                .directives
                .push(entry);
        }
    }
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

fn parse_entry(line: usize, text: &str) -> Option<GrammarEntry> {
    let text = text.strip_suffix(';')?;
    let (key, value) = text.split_once('=')?;
    let key = key.trim();
    let value = value.trim();
    if !valid_key(key) || value.is_empty() {
        return None;
    }
    Some(GrammarEntry {
        key: key.to_string(),
        value: value.to_string(),
        line,
    })
}

fn validate_entries(section: &GrammarSection, errors: &mut Vec<GrammarError>) {
    let mut keys = BTreeSet::new();
    for entry in &section.entries {
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
