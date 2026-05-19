use std::collections::BTreeMap;

use flavor_core::{RawSyntaxKind, SourceText, Span, SyntaxNode, SyntaxSpanExt};
use flavor_grammar::RawAstSchema;

use crate::{
    internal::grammar::{self as kind},
    model::RustRepeatedTokenPatternFact,
    state::RustRepeatedTokenPatternConfig,
};

pub(crate) fn collect(
    syntax: &SyntaxNode,
    source: &SourceText,
    config: &RustRepeatedTokenPatternConfig,
) -> Vec<RustRepeatedTokenPatternFact> {
    let mut candidates = Vec::new();
    let schema = kind::schema();
    summarize_node(syntax, &schema, source, config, 0, &mut candidates);

    let mut groups = BTreeMap::<GroupKey, GroupStats>::new();
    for candidate in candidates {
        groups
            .entry(candidate.key())
            .or_insert_with(|| GroupStats::new(&candidate))
            .add(candidate);
    }

    let mut facts = groups
        .into_values()
        .filter(|group| {
            group.occurrences >= config.min_occurrences
                && group.total_lines >= config.min_total_lines
        })
        .map(GroupStats::finish)
        .collect::<Vec<_>>();
    facts.sort_by(|left, right| {
        right
            .total_lines
            .cmp(&left.total_lines)
            .then_with(|| right.occurrences.cmp(&left.occurrences))
            .then_with(|| left.line.cmp(&right.line))
    });
    facts.truncate(config.max_reports);
    facts
}

fn summarize_node(
    node: &SyntaxNode,
    schema: &RawAstSchema,
    source: &SourceText,
    config: &RustRepeatedTokenPatternConfig,
    depth: usize,
    candidates: &mut Vec<Candidate>,
) -> Summary {
    let mut hash = hash_start();
    hash = hash_u16(hash, node.kind().0);
    let mut token_count = 0;
    let mut node_count = 1;

    for element in node.children_with_tokens() {
        match element {
            flavor_core::SyntaxElement::Node(child) => {
                let summary = summarize_node(&child, schema, source, config, depth + 1, candidates);
                hash = hash_u64(hash, summary.hash);
                token_count += summary.token_count;
                node_count += summary.node_count;
            }
            flavor_core::SyntaxElement::Token(token) => {
                let Some(token_hash) = normalized_token_hash(schema, token.kind(), token.text())
                else {
                    continue;
                };
                hash = hash_u64(hash, token_hash);
                token_count += 1;
            }
        }
    }

    let summary = Summary {
        hash,
        token_count,
        node_count,
    };
    if let Some(candidate) = candidate_for(node, source, config, depth, summary) {
        candidates.push(candidate);
    }
    summary
}

fn candidate_for(
    node: &SyntaxNode,
    source: &SourceText,
    config: &RustRepeatedTokenPatternConfig,
    depth: usize,
    summary: Summary,
) -> Option<Candidate> {
    if summary.token_count < config.min_tokens || summary.token_count > config.max_tokens {
        return None;
    }
    if summary.node_count < config.min_nodes {
        return None;
    }
    let span = node.source_span();
    let start_line = source.line_index().line(span.start);
    let end_line = source.line_index().line(span.end);
    let line_count = end_line.saturating_sub(start_line) + 1;
    if !(config.min_lines..=config.max_lines).contains(&line_count) {
        return None;
    }
    let token_bucket_size = config.token_bucket_size.max(1);

    Some(Candidate {
        hash: summary.hash,
        node_kind: node.kind().0,
        depth,
        token_bucket: summary.token_count / token_bucket_size,
        token_count: summary.token_count,
        span,
        line: start_line,
        line_count,
    })
}

fn normalized_token_hash(schema: &RawAstSchema, kind: RawSyntaxKind, text: &str) -> Option<u64> {
    match schema.raw_kind_name(kind) {
        Some(kind::WS) => None,
        Some(kind::IDENTIFIER) => Some(hash_tag(1)),
        Some(kind::ATTRIBUTE | kind::INNER_ATTRIBUTE) => Some(hash_tag(2)),
        Some(kind::RAW_TEXT) => normalized_raw_text_hash(text),
        _ => Some(hash_u16(hash_tag(3), kind.0)),
    }
}

fn normalized_raw_text_hash(text: &str) -> Option<u64> {
    let mut hash = hash_tag(4);
    let mut saw_part = false;
    let chars = text.as_bytes();
    let mut index = 0;
    while index < chars.len() {
        let byte = chars[index];
        if byte.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        saw_part = true;
        if is_ident_start(byte) {
            index += 1;
            while index < chars.len() && is_ident_continue(chars[index]) {
                index += 1;
            }
            hash = hash_u8(hash, b'I');
            continue;
        }
        if byte.is_ascii_digit() {
            index += 1;
            while index < chars.len() && is_ident_continue(chars[index]) {
                index += 1;
            }
            hash = hash_u8(hash, b'L');
            continue;
        }
        if matches!(byte, b'"' | b'\'') {
            index += 1;
            while index < chars.len() {
                let current = chars[index];
                index += 1;
                if current == b'\\' && index < chars.len() {
                    index += 1;
                    continue;
                }
                if current == byte {
                    break;
                }
            }
            hash = hash_u8(hash, b'L');
            continue;
        }
        hash = hash_u8(hash, byte);
        index += 1;
    }
    saw_part.then_some(hash)
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_ident_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[derive(Debug, Clone, Copy)]
struct Summary {
    hash: u64,
    token_count: usize,
    node_count: usize,
}

#[derive(Debug, Clone)]
struct Candidate {
    hash: u64,
    node_kind: u16,
    depth: usize,
    token_bucket: usize,
    token_count: usize,
    span: Span,
    line: usize,
    line_count: usize,
}

impl Candidate {
    fn key(&self) -> GroupKey {
        GroupKey {
            hash: self.hash,
            node_kind: self.node_kind,
            depth: self.depth,
            token_bucket: self.token_bucket,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct GroupKey {
    hash: u64,
    node_kind: u16,
    depth: usize,
    token_bucket: usize,
}

#[derive(Debug, Clone)]
struct GroupStats {
    first: Candidate,
    occurrences: usize,
    total_lines: usize,
}

impl GroupStats {
    fn new(candidate: &Candidate) -> Self {
        Self {
            first: candidate.clone(),
            occurrences: 0,
            total_lines: 0,
        }
    }

    fn add(&mut self, candidate: Candidate) {
        if candidate.line < self.first.line {
            self.first = candidate.clone();
        }
        self.occurrences += 1;
        self.total_lines += candidate.line_count;
    }

    fn finish(self) -> RustRepeatedTokenPatternFact {
        RustRepeatedTokenPatternFact {
            span: self.first.span,
            line: self.first.line,
            occurrences: self.occurrences,
            total_lines: self.total_lines,
            token_count: self.first.token_count,
            node_kind: self.first.node_kind,
            depth: self.first.depth,
        }
    }
}

fn hash_start() -> u64 {
    0xcbf29ce484222325
}

fn hash_tag(tag: u64) -> u64 {
    hash_u64(hash_start(), tag)
}

fn hash_u64(hash: u64, value: u64) -> u64 {
    value.to_le_bytes().into_iter().fold(hash, hash_u8)
}

fn hash_u16(hash: u64, value: u16) -> u64 {
    hash_u64(hash, u64::from(value))
}

fn hash_u8(hash: u64, value: u8) -> u64 {
    hash.wrapping_mul(0x100000001b3) ^ u64::from(value)
}
