use std::collections::BTreeSet;

use crate::{valid_ident, GrammarError};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct G4Source {
    pub name: String,
    pub kind: G4GrammarKind,
    pub parser_rules: Vec<G4Rule>,
    pub lexer_tokens: Vec<G4Rule>,
    pub parser_references: Vec<G4Reference>,
}

impl G4Source {
    pub fn defines_parser_rule(&self, name: &str) -> bool {
        self.parser_rules.iter().any(|rule| rule.name == name)
    }

    pub fn defines_lexer_token(&self, name: &str) -> bool {
        self.lexer_tokens.iter().any(|rule| rule.name == name)
    }

    pub fn defines_raw_ast_symbol(&self, name: &str) -> bool {
        self.defines_parser_rule(name) || self.defines_lexer_token(name)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum G4GrammarKind {
    Combined,
    Lexer,
    Parser,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct G4Rule {
    pub name: String,
    pub line: usize,
    pub body: String,
    pub bindings: Vec<G4Binding>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct G4Binding {
    pub backend: String,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct G4Reference {
    pub name: String,
    pub line: usize,
}

pub fn parse_g4_source(source: &str) -> Result<G4Source, Vec<GrammarError>> {
    let mut errors = Vec::new();
    let mut grammar_name = None;
    let mut grammar_kind = None;
    let mut parser_rules = Vec::new();
    let mut lexer_tokens = Vec::new();
    let mut parser_references = Vec::new();
    let mut chunk = String::new();
    let mut bindings = Vec::new();
    let mut chunk_line = 1;

    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let (line, comment) = split_line_comment(line);
        let binding = comment.and_then(parse_binding_comment);
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((kind, name)) = grammar_declaration(line) {
            grammar_kind = Some(kind);
            grammar_name = Some(name.to_string());
            continue;
        }
        if line.starts_with("options ") {
            continue;
        }
        if let Some(binding) = binding {
            bindings.push(binding);
        }
        if chunk.is_empty() {
            chunk_line = line_number;
        }
        chunk.push(' ');
        chunk.push_str(line);
        if !line.ends_with(';') {
            continue;
        }
        parse_rule_chunk(
            &chunk,
            std::mem::take(&mut bindings),
            chunk_line,
            &mut parser_rules,
            &mut lexer_tokens,
            &mut parser_references,
            &mut errors,
        );
        chunk.clear();
    }

    if !chunk.trim().is_empty() {
        errors.push(GrammarError {
            line: chunk_line,
            message: "unterminated G4 rule".to_string(),
        });
    }

    let name = grammar_name.unwrap_or_else(|| {
        errors.push(GrammarError {
            line: 1,
            message: "missing G4 grammar declaration".to_string(),
        });
        String::new()
    });
    let kind = grammar_kind.unwrap_or(G4GrammarKind::Combined);

    if errors.is_empty() {
        Ok(G4Source {
            name,
            kind,
            parser_rules,
            lexer_tokens,
            parser_references,
        })
    } else {
        Err(errors)
    }
}

pub fn parse_g4_source_validated(source: &str) -> Result<G4Source, Vec<GrammarError>> {
    let document = parse_g4_source(source)?;
    let errors = validate_g4_source(&document);
    if errors.is_empty() {
        Ok(document)
    } else {
        Err(errors)
    }
}

pub fn validate_g4_source(source: &G4Source) -> Vec<GrammarError> {
    let mut errors = Vec::new();
    validate_unique_rules("parser rule", &source.parser_rules, &mut errors);
    validate_unique_rules("lexer token", &source.lexer_tokens, &mut errors);

    let parser_rules = source
        .parser_rules
        .iter()
        .map(|rule| rule.name.as_str())
        .collect::<BTreeSet<_>>();

    for reference in &source.parser_references {
        if !parser_rules.contains(reference.name.as_str()) {
            errors.push(GrammarError {
                line: reference.line,
                message: format!("references undefined parser rule `{}`", reference.name),
            });
        }
    }
    errors
}

fn validate_unique_rules(label: &str, rules: &[G4Rule], errors: &mut Vec<GrammarError>) {
    let mut seen = BTreeSet::new();
    for rule in rules {
        if !seen.insert(rule.name.as_str()) {
            errors.push(GrammarError {
                line: rule.line,
                message: format!("duplicate {label} `{}`", rule.name),
            });
        }
    }
}

fn split_line_comment(line: &str) -> (&str, Option<&str>) {
    match line.split_once("//") {
        Some((source, comment)) => (source, Some(comment)),
        None => (line, None),
    }
}

fn parse_binding_comment(comment: &str) -> Option<G4Binding> {
    let (backend, name) = comment.trim().split_once(':')?;
    let backend = backend.trim();
    let name = name.trim();
    if valid_ident(backend) && valid_binding_name(name) {
        Some(G4Binding {
            backend: backend.to_string(),
            name: name.to_string(),
        })
    } else {
        None
    }
}

fn valid_binding_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

fn grammar_declaration(line: &str) -> Option<(G4GrammarKind, &str)> {
    if let Some(rest) = line.strip_prefix("lexer grammar ") {
        return grammar_name(rest).map(|name| (G4GrammarKind::Lexer, name));
    }
    if let Some(rest) = line.strip_prefix("parser grammar ") {
        return grammar_name(rest).map(|name| (G4GrammarKind::Parser, name));
    }
    if let Some(rest) = line.strip_prefix("grammar ") {
        return grammar_name(rest).map(|name| (G4GrammarKind::Combined, name));
    }
    None
}

fn grammar_name(rest: &str) -> Option<&str> {
    rest.trim().strip_suffix(';').map(str::trim)
}

fn parse_rule_chunk(
    chunk: &str,
    bindings: Vec<G4Binding>,
    line: usize,
    parser_rules: &mut Vec<G4Rule>,
    lexer_tokens: &mut Vec<G4Rule>,
    parser_references: &mut Vec<G4Reference>,
    errors: &mut Vec<GrammarError>,
) {
    let Some((raw_name, body)) = chunk.split_once(':') else {
        return;
    };
    let body = body.trim().trim_end_matches(';').trim();
    let name = raw_name
        .trim()
        .strip_prefix("fragment ")
        .unwrap_or(raw_name.trim())
        .trim();
    if !valid_ident(name) || !(is_parser_rule(name) || is_lexer_token(name)) {
        errors.push(GrammarError {
            line,
            message: format!("invalid G4 rule name `{name}`"),
        });
        return;
    }

    let rule = G4Rule {
        name: name.to_string(),
        line,
        body: body.to_string(),
        bindings,
    };
    if is_parser_rule(name) {
        parser_references.extend(parser_rule_references(body, line));
        parser_rules.push(rule);
    } else if is_lexer_token(name) {
        lexer_tokens.push(rule);
    }
}

fn parser_rule_references(body: &str, line: usize) -> Vec<G4Reference> {
    let mut references = Vec::new();
    let mut current = String::new();
    for ch in body.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
            continue;
        }
        add_parser_rule_reference(&mut references, &mut current, line);
    }
    add_parser_rule_reference(&mut references, &mut current, line);
    references
}

fn add_parser_rule_reference(references: &mut Vec<G4Reference>, current: &mut String, line: usize) {
    if is_parser_rule(current) {
        references.push(G4Reference {
            name: std::mem::take(current),
            line,
        });
    } else {
        current.clear();
    }
}

fn is_parser_rule(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|ch| ch.is_ascii_lowercase())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn is_lexer_token(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|ch| ch.is_ascii_uppercase())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
