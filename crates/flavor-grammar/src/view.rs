use std::sync::Arc;

use flavor_core::{
    RawSyntaxKind, SourceText, Span, SyntaxElement, SyntaxNode, SyntaxSpanExt, SyntaxToken,
};

use crate::RawAstSchema;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TokenTextRun<'a> {
    parts: &'a [&'a str],
    part_prefixes: &'a [&'a str],
    joins: &'a [&'a str],
    skip: &'a [&'a str],
}

impl<'a> TokenTextRun<'a> {
    pub fn new(parts: &'a [&'a str], joins: &'a [&'a str]) -> Self {
        Self {
            parts,
            part_prefixes: &[],
            joins,
            skip: &[],
        }
    }

    pub fn with_part_prefixes(mut self, prefixes: &'a [&'a str]) -> Self {
        self.part_prefixes = prefixes;
        self
    }

    pub fn with_skip(mut self, skip: &'a [&'a str]) -> Self {
        self.skip = skip;
        self
    }
}

#[derive(Debug, Clone)]
pub struct GrammarContext {
    schema: GrammarSchema,
}

#[derive(Debug, Clone)]
enum GrammarSchema {
    Static(&'static RawAstSchema),
    Shared(Arc<RawAstSchema>),
}

impl GrammarContext {
    pub fn new(schema: RawAstSchema) -> Self {
        Self {
            schema: GrammarSchema::Shared(Arc::new(schema)),
        }
    }

    pub fn from_static(schema: &'static RawAstSchema) -> Self {
        Self {
            schema: GrammarSchema::Static(schema),
        }
    }

    pub fn schema(&self) -> &RawAstSchema {
        match &self.schema {
            GrammarSchema::Static(schema) => schema,
            GrammarSchema::Shared(schema) => schema.as_ref(),
        }
    }
}

impl From<RawAstSchema> for GrammarContext {
    fn from(schema: RawAstSchema) -> Self {
        Self::new(schema)
    }
}

impl From<&'static RawAstSchema> for GrammarContext {
    fn from(schema: &'static RawAstSchema) -> Self {
        Self::from_static(schema)
    }
}

#[derive(Debug, Clone)]
pub struct GrammarTree {
    root: SyntaxNode,
    context: GrammarContext,
}

impl GrammarTree {
    pub fn new(root: SyntaxNode, context: impl Into<GrammarContext>) -> Self {
        Self {
            root,
            context: context.into(),
        }
    }

    pub fn root(&self) -> GrammarNode {
        GrammarNode::new(self.root.clone(), self.context.clone())
    }

    pub fn find<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = GrammarNode> + 'a {
        self.root
            .descendants()
            .map(|node| GrammarNode::new(node, self.context.clone()))
            .filter(move |node| node.is(kind))
    }
}

#[derive(Debug, Clone)]
pub struct GrammarNode {
    node: SyntaxNode,
    context: GrammarContext,
}

impl GrammarNode {
    pub fn new(node: SyntaxNode, context: impl Into<GrammarContext>) -> Self {
        Self {
            node,
            context: context.into(),
        }
    }

    pub fn raw(&self) -> &SyntaxNode {
        &self.node
    }

    pub fn raw_kind(&self) -> RawSyntaxKind {
        self.node.kind()
    }

    pub fn kind_name(&self) -> Option<&str> {
        self.context.schema().raw_kind_name(self.raw_kind())
    }

    pub fn is(&self, kind: &str) -> bool {
        self.kind_name() == Some(kind)
    }

    pub fn is_any(&self, kinds: &[&str]) -> bool {
        kind_matches(self.kind_name(), kinds)
    }

    pub fn span(&self) -> Span {
        self.node.source_span()
    }

    pub fn trimmed_span(&self, source: &SourceText) -> Span {
        trimmed_span(self.span(), source)
    }

    pub fn line(&self, source: &SourceText) -> usize {
        source.line_index().line(self.span().start)
    }

    pub fn line_count(&self, source: &SourceText) -> usize {
        let span = self.span();
        let index = source.line_index();
        index.line(span.end).saturating_sub(index.line(span.start)) + 1
    }

    pub fn source_text<'a>(&self, source: &'a SourceText) -> &'a str {
        source.slice(self.span())
    }

    pub fn trimmed_source_text<'a>(&self, source: &'a SourceText) -> &'a str {
        source.slice(self.trimmed_span(source))
    }

    pub fn text(&self) -> String {
        self.node.text().to_string()
    }

    pub fn children(&self) -> impl Iterator<Item = Self> + '_ {
        self.node
            .children()
            .map(|node| Self::new(node, self.context.clone()))
    }

    pub fn descendants(&self) -> impl Iterator<Item = Self> + '_ {
        self.node
            .descendants()
            .map(|node| Self::new(node, self.context.clone()))
    }

    pub fn child(&self, kind: &str) -> Option<Self> {
        self.children().find(|node| node.is(kind))
    }

    pub fn child_any(&self, kinds: &[&str]) -> Option<Self> {
        self.children().find(|node| node.is_any(kinds))
    }

    pub fn children_named<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = Self> + 'a {
        self.children().filter(move |node| node.is(kind))
    }

    pub fn children_any<'a>(&'a self, kinds: &'a [&'a str]) -> impl Iterator<Item = Self> + 'a {
        self.children().filter(move |node| node.is_any(kinds))
    }

    pub fn child_text(&self, kind: &str) -> Option<String> {
        self.child(kind).map(|node| node.text())
    }

    pub fn tokens(&self) -> impl Iterator<Item = GrammarToken> + '_ {
        self.node
            .descendants_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .map(|token| GrammarToken::new(token, self.context.clone()))
    }

    pub fn tokens_any<'a>(
        &'a self,
        kinds: &'a [&'a str],
    ) -> impl Iterator<Item = GrammarToken> + 'a {
        self.tokens().filter(move |token| token.is_any(kinds))
    }

    pub fn child_tokens(&self) -> impl Iterator<Item = GrammarToken> + '_ {
        self.node
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .map(|token| GrammarToken::new(token, self.context.clone()))
    }

    pub fn child_tokens_any<'a>(
        &'a self,
        kinds: &'a [&'a str],
    ) -> impl Iterator<Item = GrammarToken> + 'a {
        self.child_tokens().filter(move |token| token.is_any(kinds))
    }

    pub fn head_tokens(&self) -> impl Iterator<Item = GrammarToken> + '_ {
        let first_child_start = self.children().map(|child| child.span().start).min();
        self.child_tokens()
            .filter(move |token| first_child_start.is_none_or(|start| token.span().start < start))
    }

    pub fn token(&self, kind: &str) -> Option<GrammarToken> {
        self.child_tokens().find(|token| token.is(kind))
    }

    pub fn has_token(&self, kind: &str) -> bool {
        self.token(kind).is_some()
    }

    pub fn child_token_text(&self, kind: &str) -> Option<String> {
        self.token(kind).map(|token| token.text().to_string())
    }

    pub fn child_token_text_any(&self, kinds: &[&str]) -> Option<String> {
        self.child_tokens_any(kinds)
            .next()
            .map(|token| token.text().to_string())
    }

    pub fn head_token_text_any(&self, kinds: &[&str]) -> Option<String> {
        self.head_tokens()
            .find(|token| token.is_any(kinds))
            .map(|token| token.text().to_string())
    }

    pub fn token_text(&self, kind: &str) -> Option<String> {
        self.tokens_named(kind)
            .next()
            .map(|token| token.text().to_string())
    }

    pub fn token_text_any(&self, kinds: &[&str]) -> Option<String> {
        self.tokens_any(kinds)
            .next()
            .map(|token| token.text().to_string())
    }

    pub fn last_token_text_any(&self, kinds: &[&str]) -> Option<String> {
        self.tokens_any(kinds)
            .last()
            .map(|token| token.text().to_string())
    }

    pub fn tokens_named<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = GrammarToken> + 'a {
        self.tokens().filter(move |token| token.is(kind))
    }

    pub fn token_run_text(&self, run: TokenTextRun<'_>) -> Option<String> {
        let mut text = String::new();
        let mut started = false;
        for token in self.child_tokens() {
            if !started && token.is_any(run.skip) {
                continue;
            }
            if token.is_any(run.parts)
                || token.kind_starts_with_any(run.part_prefixes)
                || (started && token.is_any(run.joins))
            {
                started = true;
                text.push_str(token.text());
                continue;
            }
            if started {
                break;
            }
        }
        started.then_some(text)
    }
}

fn trimmed_span(span: Span, source: &SourceText) -> Span {
    let text = source.slice(span);
    let leading = text.len().saturating_sub(text.trim_start().len());
    let trailing = text.len().saturating_sub(text.trim_end().len());
    Span::from_usize(
        span.start as usize + leading,
        (span.end as usize).saturating_sub(trailing),
    )
}

#[derive(Debug, Clone)]
pub struct GrammarToken {
    token: SyntaxToken,
    context: GrammarContext,
}

impl GrammarToken {
    pub fn new(token: SyntaxToken, context: impl Into<GrammarContext>) -> Self {
        Self {
            token,
            context: context.into(),
        }
    }

    pub fn raw(&self) -> &SyntaxToken {
        &self.token
    }

    pub fn raw_kind(&self) -> RawSyntaxKind {
        self.token.kind()
    }

    pub fn kind_name(&self) -> Option<&str> {
        self.context.schema().raw_kind_name(self.raw_kind())
    }

    pub fn is(&self, kind: &str) -> bool {
        self.kind_name() == Some(kind)
    }

    pub fn is_any(&self, kinds: &[&str]) -> bool {
        kind_matches(self.kind_name(), kinds)
    }

    pub fn kind_starts_with(&self, prefix: &str) -> bool {
        self.kind_name()
            .is_some_and(|kind| kind.starts_with(prefix))
    }

    pub fn kind_starts_with_any(&self, prefixes: &[&str]) -> bool {
        self.kind_name()
            .is_some_and(|kind| prefixes.iter().any(|prefix| kind.starts_with(prefix)))
    }

    pub fn span(&self) -> Span {
        self.token.source_span()
    }

    pub fn line(&self, source: &SourceText) -> usize {
        source.line_index().line(self.span().start)
    }

    pub fn source_text<'a>(&self, source: &'a SourceText) -> &'a str {
        source.slice(self.span())
    }

    pub fn text(&self) -> &str {
        self.token.text()
    }
}

fn kind_matches(kind: Option<&str>, kinds: &[&str]) -> bool {
    kind.is_some_and(|kind| kinds.contains(&kind))
}
