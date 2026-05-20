use flavor_core::{Diagnostic, SourceText, Span, SyntaxNode};

use crate::{
    G4Source, GrammarBundle, GrammarError, GrammarMetadata, GrammarParseOutput, RawAstBuilder,
    RawAstSchema, RawAstSymbolKind,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TreeSitterParseConfig<'a> {
    pub backend: &'a str,
    pub root_kind: &'a str,
    pub whitespace_kind: &'a str,
    pub fallback_kind: &'a str,
    pub failure_code: &'a str,
    pub failure_message: &'a str,
    pub error_code: &'a str,
    pub error_message: &'a str,
}

pub fn parse_tree_sitter(
    bundle: &GrammarBundle,
    language: tree_sitter::Language,
    source: SourceText,
    config: TreeSitterParseConfig<'_>,
) -> Result<GrammarParseOutput, Vec<GrammarError>> {
    let adapter = TreeSitterRawAstAdapter::new(
        bundle,
        config.backend,
        config.root_kind,
        config.whitespace_kind,
        config.fallback_kind,
    )?;
    let mut parser = tree_sitter::Parser::new();
    if let Err(error) = parser.set_language(&language) {
        let syntax = adapter.build_error(&source);
        return Ok(GrammarParseOutput::new(
            source,
            syntax,
            vec![Diagnostic::error_code(
                None,
                config.failure_code,
                format!("{}: {error}", config.failure_message),
            )],
        ));
    }
    let Some(tree) = parser.parse(source.as_str(), None) else {
        let syntax = adapter.build_error(&source);
        return Ok(GrammarParseOutput::new(
            source,
            syntax,
            vec![Diagnostic::error_code(
                None,
                config.failure_code,
                config.failure_message,
            )],
        ));
    };

    let root = tree.root_node();
    let syntax = adapter.build(root, &source);
    let diagnostics = if root.has_error() {
        vec![Diagnostic::error_code(
            tree_sitter_error_span(root),
            config.error_code,
            config.error_message,
        )]
    } else {
        Vec::new()
    };
    Ok(GrammarParseOutput::new(source, syntax, diagnostics))
}

#[derive(Debug, Clone)]
pub struct TreeSitterRawAstAdapter<'bundle> {
    schema: &'bundle RawAstSchema,
    root_kind: String,
    whitespace_kind: String,
    fallback_kind: String,
    mappings: BackendKindMap,
}

impl<'bundle> TreeSitterRawAstAdapter<'bundle> {
    pub fn new(
        bundle: &'bundle GrammarBundle,
        backend: &str,
        root_kind: &str,
        whitespace_kind: &str,
        fallback_kind: &str,
    ) -> Result<Self, Vec<GrammarError>> {
        let mut errors = Vec::new();
        let schema = bundle.schema();
        validate_symbol(schema, root_kind, RawAstSymbolKind::Node, 1, &mut errors);
        validate_symbol(
            schema,
            whitespace_kind,
            RawAstSymbolKind::Token,
            1,
            &mut errors,
        );
        validate_symbol(
            schema,
            fallback_kind,
            RawAstSymbolKind::Token,
            1,
            &mut errors,
        );

        let mappings = BackendKindMap::new(
            schema,
            bundle.metadata(),
            bundle.sources(),
            backend,
            &mut errors,
        );
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Self {
            schema,
            root_kind: root_kind.to_string(),
            whitespace_kind: whitespace_kind.to_string(),
            fallback_kind: fallback_kind.to_string(),
            mappings,
        })
    }

    pub fn build(&self, root: tree_sitter::Node<'_>, source: &SourceText) -> SyntaxNode {
        let mut builder = RawAstBuilder::new(self.schema);
        builder.start_node(self.root_kind.as_str());
        self.build_children_in(
            &mut builder,
            root,
            source.as_str(),
            0,
            source.as_str().len(),
        );
        builder.finish_node();
        builder.finish()
    }

    pub fn build_error(&self, source: &SourceText) -> SyntaxNode {
        let mut builder = RawAstBuilder::new(self.schema);
        builder.start_node(self.root_kind.as_str());
        if !source.as_str().is_empty() {
            builder.token(self.fallback_kind.as_str(), source.as_str());
        }
        builder.finish_node();
        builder.finish()
    }

    fn build_node(
        &self,
        builder: &mut RawAstBuilder<'_>,
        node: tree_sitter::Node<'_>,
        kind: &str,
        source: &str,
    ) {
        builder.start_node(kind);
        self.build_children_in(builder, node, source, node.start_byte(), node.end_byte());
        builder.finish_node();
    }

    fn build_children_in(
        &self,
        builder: &mut RawAstBuilder<'_>,
        node: tree_sitter::Node<'_>,
        source: &str,
        start: usize,
        end: usize,
    ) {
        let mut cursor = node.walk();
        let mut position = start;
        for child in node.named_children(&mut cursor) {
            self.push_gap(builder, source, position, child.start_byte());
            self.build_child(builder, child, source);
            position = child.end_byte();
        }
        self.push_gap(builder, source, position, end);
    }

    fn build_child(
        &self,
        builder: &mut RawAstBuilder<'_>,
        node: tree_sitter::Node<'_>,
        source: &str,
    ) {
        if let Some(kind) = self.node_kind(node.kind()) {
            self.build_node(builder, node, kind, source);
        } else if let Some(kind) = self.token_kind(node.kind()) {
            self.push_token(builder, source, node.start_byte(), node.end_byte(), kind);
        } else if node.named_child_count() == 0 {
            let fallback = self.fallback_kind.as_str();
            self.push_token(
                builder,
                source,
                node.start_byte(),
                node.end_byte(),
                fallback,
            );
        } else {
            self.build_children_in(builder, node, source, node.start_byte(), node.end_byte());
        }
    }

    fn push_gap(&self, builder: &mut RawAstBuilder<'_>, source: &str, start: usize, end: usize) {
        if start >= end {
            return;
        }
        let text = &source[start..end];
        let kind = self.gap_kind(text);
        builder.token(kind, text);
    }

    fn push_token(
        &self,
        builder: &mut RawAstBuilder<'_>,
        source: &str,
        start: usize,
        end: usize,
        kind: &str,
    ) {
        if start < end {
            builder.token(kind, &source[start..end]);
        }
    }

    fn node_kind(&self, kind: &str) -> Option<&str> {
        self.mappings.nodes.get(kind)
    }

    fn token_kind(&self, kind: &str) -> Option<&str> {
        self.mappings.tokens.get(kind)
    }

    fn gap_kind(&self, text: &str) -> &str {
        if text.chars().all(char::is_whitespace) {
            return self.whitespace_kind.as_str();
        }
        self.mappings
            .literals
            .get(text.trim())
            .unwrap_or(self.fallback_kind.as_str())
    }
}

#[derive(Debug, Clone)]
struct BackendKindMap {
    nodes: KindMap,
    tokens: KindMap,
    literals: KindMap,
}

impl BackendKindMap {
    fn new(
        schema: &RawAstSchema,
        metadata: &GrammarMetadata,
        sources: &[G4Source],
        backend: &str,
        errors: &mut Vec<GrammarError>,
    ) -> Self {
        Self {
            nodes: node_mappings(schema, metadata, backend, errors),
            tokens: token_mappings(schema, sources, backend, errors),
            literals: literal_mappings(schema, sources, errors),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct KindMap {
    entries: Vec<KindMapping>,
}

impl KindMap {
    fn push(&mut self, source: impl Into<String>, kind: impl Into<String>) {
        self.entries.push(KindMapping {
            source: source.into(),
            kind: kind.into(),
        });
    }

    fn get(&self, source: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| entry.source == source)
            .map(|entry| entry.kind.as_str())
    }
}

#[derive(Debug, Clone)]
struct KindMapping {
    source: String,
    kind: String,
}

fn node_mappings(
    schema: &RawAstSchema,
    metadata: &GrammarMetadata,
    backend: &str,
    errors: &mut Vec<GrammarError>,
) -> KindMap {
    let mut mappings = KindMap::default();
    let prefix = format!("{backend}:");
    if let Some(nodes) = metadata.section("nodes") {
        for entry in &nodes.entries {
            let Some(binding) = entry.value.strip_prefix(&prefix) else {
                continue;
            };
            if validate_symbol(
                schema,
                &entry.key,
                RawAstSymbolKind::Node,
                entry.line,
                errors,
            ) {
                mappings.push(binding, entry.key.as_str());
            }
        }
    }
    mappings
}

fn token_mappings(
    schema: &RawAstSchema,
    sources: &[G4Source],
    backend: &str,
    errors: &mut Vec<GrammarError>,
) -> KindMap {
    let mut mappings = KindMap::default();
    for rule in sources.iter().flat_map(|source| &source.lexer_tokens) {
        for binding in rule
            .bindings
            .iter()
            .filter(|binding| binding.backend == backend)
        {
            if validate_symbol(
                schema,
                &rule.name,
                RawAstSymbolKind::Token,
                rule.line,
                errors,
            ) {
                mappings.push(binding.name.as_str(), rule.name.as_str());
            }
        }
    }
    mappings
}

fn literal_mappings(
    schema: &RawAstSchema,
    sources: &[G4Source],
    errors: &mut Vec<GrammarError>,
) -> KindMap {
    let mut mappings = KindMap::default();
    for rule in sources.iter().flat_map(|source| &source.lexer_tokens) {
        let Some(literal) = simple_literal(&rule.body) else {
            continue;
        };
        if validate_symbol(
            schema,
            &rule.name,
            RawAstSymbolKind::Token,
            rule.line,
            errors,
        ) {
            mappings.push(literal, rule.name.as_str());
        }
    }
    mappings
}

fn validate_symbol(
    schema: &RawAstSchema,
    name: &str,
    expected: RawAstSymbolKind,
    line: usize,
    errors: &mut Vec<GrammarError>,
) -> bool {
    let Some(symbol) = schema.symbol(name) else {
        errors.push(GrammarError {
            line,
            message: format!("raw AST schema is missing symbol `{name}`"),
        });
        return false;
    };
    if symbol.kind != expected {
        errors.push(GrammarError {
            line,
            message: format!("raw AST symbol `{name}` has wrong category"),
        });
        return false;
    }
    true
}

fn simple_literal(body: &str) -> Option<String> {
    let body = body.trim();
    let rest = body.strip_prefix('\'')?;
    let (literal, rest) = rest.split_once('\'')?;
    rest.trim().is_empty().then(|| literal.to_string())
}

pub fn tree_sitter_error_span(node: tree_sitter::Node<'_>) -> Option<Span> {
    if node.is_error() || node.is_missing() {
        return Some(Span::from_usize(node.start_byte(), node.end_byte()));
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Some(span) = tree_sitter_error_span(child) {
            return Some(span);
        }
    }
    None
}
