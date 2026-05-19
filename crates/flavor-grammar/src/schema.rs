use std::collections::{BTreeMap, BTreeSet};

use flavor_core::RawSyntaxKind;

use crate::{
    metadata::parse_metadata_validated, parse_g4_source_validated, G4Rule, G4Source, GrammarError,
    GrammarMetadata,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RawAstSchema {
    grammar_id: String,
    symbols: Vec<RawAstSymbol>,
    by_name: BTreeMap<String, usize>,
    by_raw: BTreeMap<u16, usize>,
}

impl RawAstSchema {
    pub fn new(grammar_id: impl Into<String>, symbols: Vec<RawAstSymbol>) -> Self {
        let mut by_name = BTreeMap::new();
        let mut by_raw = BTreeMap::new();
        for (index, symbol) in symbols.iter().enumerate() {
            by_name.insert(symbol.name.clone(), index);
            by_raw.insert(symbol.raw_kind, index);
        }
        Self {
            grammar_id: grammar_id.into(),
            symbols,
            by_name,
            by_raw,
        }
    }

    pub fn from_g4_sources(
        grammar_id: &str,
        start: u16,
        sources: &[G4Source],
    ) -> Result<Self, Vec<GrammarError>> {
        let mut symbols = Vec::new();
        let mut errors = Vec::new();
        let mut seen = BTreeSet::new();
        let mut next = start;
        let mut source_symbols = sources
            .iter()
            .flat_map(|source| {
                source
                    .parser_rules
                    .iter()
                    .map(|rule| (rule, RawAstSymbolKind::Node))
            })
            .collect::<Vec<_>>();
        source_symbols.extend(sources.iter().flat_map(|source| {
            source
                .lexer_tokens
                .iter()
                .map(|rule| (rule, RawAstSymbolKind::Token))
        }));

        for (rule, kind) in source_symbols {
            push_raw_ast_symbol(&mut symbols, &mut seen, &mut next, rule, kind, &mut errors);
        }

        if errors.is_empty() {
            Ok(Self::new(grammar_id, symbols))
        } else {
            Err(errors)
        }
    }

    pub fn grammar_id(&self) -> &str {
        &self.grammar_id
    }

    pub fn symbols(&self) -> &[RawAstSymbol] {
        &self.symbols
    }

    pub fn symbol(&self, name: &str) -> Option<&RawAstSymbol> {
        self.by_name.get(name).map(|index| &self.symbols[*index])
    }

    pub fn symbol_for_raw(&self, kind: RawSyntaxKind) -> Option<&RawAstSymbol> {
        self.by_raw.get(&kind.0).map(|index| &self.symbols[*index])
    }

    pub fn raw_kind(&self, kind: impl GrammarKindName) -> RawSyntaxKind {
        let name = kind.grammar_kind_name();
        self.try_raw_kind(name)
            .unwrap_or_else(|| panic!("raw AST schema `{}` is missing `{name}`", self.grammar_id))
    }

    pub fn try_raw_kind(&self, kind: impl GrammarKindName) -> Option<RawSyntaxKind> {
        self.symbol(kind.grammar_kind_name())
            .map(|symbol| RawSyntaxKind(symbol.raw_kind))
    }

    pub fn node_kind(&self, kind: impl GrammarKindName) -> RawSyntaxKind {
        self.expect_kind(kind, RawAstSymbolKind::Node)
    }

    pub fn token_kind(&self, kind: impl GrammarKindName) -> RawSyntaxKind {
        self.expect_kind(kind, RawAstSymbolKind::Token)
    }

    pub fn raw_is_node(&self, kind: RawSyntaxKind) -> bool {
        self.symbol_for_raw(kind)
            .is_some_and(|symbol| symbol.kind == RawAstSymbolKind::Node)
    }

    pub fn raw_is_token(&self, kind: RawSyntaxKind) -> bool {
        self.symbol_for_raw(kind)
            .is_some_and(|symbol| symbol.kind == RawAstSymbolKind::Token)
    }

    pub fn raw_kind_name(&self, kind: RawSyntaxKind) -> Option<&str> {
        self.symbol_for_raw(kind).map(|symbol| symbol.name.as_str())
    }

    pub fn is_node_name(&self, kind: impl GrammarKindName) -> bool {
        self.symbol(kind.grammar_kind_name())
            .is_some_and(|symbol| symbol.kind == RawAstSymbolKind::Node)
    }

    pub fn is_token_name(&self, kind: impl GrammarKindName) -> bool {
        self.symbol(kind.grammar_kind_name())
            .is_some_and(|symbol| symbol.kind == RawAstSymbolKind::Token)
    }

    fn expect_kind(&self, kind: impl GrammarKindName, expected: RawAstSymbolKind) -> RawSyntaxKind {
        let name = kind.grammar_kind_name();
        let symbol = self
            .symbol(name)
            .unwrap_or_else(|| panic!("raw AST schema `{}` is missing `{name}`", self.grammar_id));
        assert_eq!(
            symbol.kind, expected,
            "raw AST schema `{}` symbol `{name}` has the wrong category",
            self.grammar_id
        );
        RawSyntaxKind(symbol.raw_kind)
    }
}

pub trait GrammarKindName {
    fn grammar_kind_name(&self) -> &str;
}

impl GrammarKindName for &str {
    fn grammar_kind_name(&self) -> &str {
        self
    }
}

impl GrammarKindName for String {
    fn grammar_kind_name(&self) -> &str {
        self
    }
}

impl GrammarKindName for &String {
    fn grammar_kind_name(&self) -> &str {
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct GrammarSpec<'a> {
    grammar_id: &'a str,
    raw_kind_start: u16,
    sources: &'a [&'a str],
    metadata: &'a str,
}

impl<'a> GrammarSpec<'a> {
    pub const fn new(
        grammar_id: &'a str,
        raw_kind_start: u16,
        sources: &'a [&'a str],
        metadata: &'a str,
    ) -> Self {
        Self {
            grammar_id,
            raw_kind_start,
            sources,
            metadata,
        }
    }

    pub fn grammar_id(&self) -> &str {
        self.grammar_id
    }

    pub fn parse_sources(&self) -> Result<Vec<G4Source>, Vec<GrammarError>> {
        self.sources
            .iter()
            .map(|source| parse_g4_source_validated(source))
            .collect()
    }

    pub fn schema(&self) -> Result<RawAstSchema, Vec<GrammarError>> {
        let sources = self.parse_sources()?;
        RawAstSchema::from_g4_sources(self.grammar_id, self.raw_kind_start, &sources)
    }

    pub fn metadata(&self) -> Result<GrammarMetadata, Vec<GrammarError>> {
        parse_metadata_validated(self.metadata)?
            .into_iter()
            .find(|document| document.name == self.grammar_id)
            .ok_or_else(|| {
                vec![GrammarError {
                    line: 1,
                    message: format!("missing metadata for grammar `{}`", self.grammar_id),
                }]
            })
    }

    pub fn bundle(&self) -> Result<GrammarBundle, Vec<GrammarError>> {
        let sources = self.parse_sources()?;
        let schema = RawAstSchema::from_g4_sources(self.grammar_id, self.raw_kind_start, &sources)?;
        let metadata = self.metadata()?;
        Ok(GrammarBundle {
            schema,
            sources,
            metadata,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GrammarBundle {
    schema: RawAstSchema,
    sources: Vec<G4Source>,
    metadata: GrammarMetadata,
}

impl GrammarBundle {
    pub fn schema(&self) -> &RawAstSchema {
        &self.schema
    }

    pub fn into_schema(self) -> RawAstSchema {
        self.schema
    }

    pub fn sources(&self) -> &[G4Source] {
        &self.sources
    }

    pub fn metadata(&self) -> &GrammarMetadata {
        &self.metadata
    }

    pub fn raw_kind(&self, kind: impl GrammarKindName) -> RawSyntaxKind {
        self.schema.raw_kind(kind)
    }

    pub fn raw_is_node(&self, kind: RawSyntaxKind) -> bool {
        self.schema.raw_is_node(kind)
    }

    pub fn raw_is_token(&self, kind: RawSyntaxKind) -> bool {
        self.schema.raw_is_token(kind)
    }

    pub fn is_token_name(&self, kind: impl GrammarKindName) -> bool {
        self.schema.is_token_name(kind)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RawAstSymbol {
    pub name: String,
    pub kind: RawAstSymbolKind,
    pub raw_kind: u16,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RawAstSymbolKind {
    Node,
    Token,
}

fn push_raw_ast_symbol(
    symbols: &mut Vec<RawAstSymbol>,
    seen: &mut BTreeSet<String>,
    next: &mut u16,
    rule: &G4Rule,
    kind: RawAstSymbolKind,
    errors: &mut Vec<GrammarError>,
) {
    if !seen.insert(rule.name.clone()) {
        errors.push(GrammarError {
            line: rule.line,
            message: format!("duplicate raw AST symbol `{}`", rule.name),
        });
        return;
    }
    let Some(raw_kind) = reserve_raw_kind(next) else {
        errors.push(GrammarError {
            line: rule.line,
            message: "raw AST kind allocation overflowed u16".to_string(),
        });
        return;
    };
    symbols.push(RawAstSymbol {
        name: rule.name.clone(),
        kind,
        raw_kind,
    });
}

fn reserve_raw_kind(next: &mut u16) -> Option<u16> {
    let current = *next;
    *next = next.checked_add(1)?;
    Some(current)
}
