use flavor_plugin_core::{Span, Token};
use tree_sitter::{Node, Parser};

use crate::{
    ast::TsSourceFile,
    state::{SourceMode, TsPluginConfig},
    syntax_kind::TsSyntaxKind,
};

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct TsFacts {
    pub import_count: usize,
    pub export_count: usize,
    pub names: Vec<TsNameFact>,
    pub dispatch_branches: Vec<TsDispatchBranchFact>,
    pub imports: Vec<TsImportFact>,
    pub jsx_elements: Vec<TsxElementFact>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TsNameKind {
    Function,
    Method,
    Binding,
    Parameter,
}

impl TsNameKind {
    pub fn label(self) -> &'static str {
        match self {
            TsNameKind::Function => "function",
            TsNameKind::Method => "method",
            TsNameKind::Binding => "binding",
            TsNameKind::Parameter => "parameter",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TsNameFact {
    pub kind: TsNameKind,
    pub name: String,
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TsDispatchBranchFact {
    pub span: Span,
    pub line: usize,
    pub lines: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TsImportFact {
    pub source: String,
    pub type_only: bool,
    pub specifiers: Vec<TsImportSpecifier>,
    pub span: Span,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TsImportSpecifier {
    Default(String),
    Named(String),
    Namespace(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TsxElementFact {
    pub name: String,
    pub root: Option<String>,
    pub intrinsic: Option<String>,
    pub self_closing: bool,
    pub span: Span,
    pub line: usize,
}

pub fn collect(source_file: &TsSourceFile, config: &TsPluginConfig) -> TsFacts {
    let mut facts = legacy_counts(source_file.tokens());
    let Some(tree) = parse_tree(source_file.source().as_str(), config) else {
        return facts;
    };
    collect_node(
        tree.root_node(),
        source_file.source().as_str().as_bytes(),
        &mut facts,
    );
    facts
}

fn legacy_counts(tokens: &[Token<TsSyntaxKind>]) -> TsFacts {
    let mut facts = TsFacts::default();
    for token in tokens {
        match token.kind {
            TsSyntaxKind::KeywordImport => facts.import_count += 1,
            TsSyntaxKind::KeywordExport => facts.export_count += 1,
            _ => {}
        }
    }
    facts
}

fn parse_tree(source: &str, config: &TsPluginConfig) -> Option<tree_sitter::Tree> {
    let language = match config.source_mode {
        SourceMode::JavaScript | SourceMode::TypeScript | SourceMode::Declaration => {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT
        }
        SourceMode::Jsx | SourceMode::Tsx => tree_sitter_typescript::LANGUAGE_TSX,
    };
    let mut parser = Parser::new();
    parser.set_language(&language.into()).ok()?;
    parser.parse(source, None)
}

fn collect_node(node: Node<'_>, source: &[u8], facts: &mut TsFacts) {
    match node.kind() {
        "function_declaration" | "function_expression" | "generator_function" => {
            collect_named_child(node, source, facts, "name", TsNameKind::Function);
        }
        "method_definition" | "method_signature" => {
            collect_named_child(node, source, facts, "name", TsNameKind::Method);
        }
        "variable_declarator" => {
            if let Some(name) = node.child_by_field_name("name") {
                collect_pattern_names(name, source, facts, TsNameKind::Binding);
            }
        }
        "required_parameter" | "optional_parameter" => {
            if let Some(pattern) = node
                .child_by_field_name("pattern")
                .or_else(|| node.child_by_field_name("name"))
                .or_else(|| first_named_child(node))
            {
                collect_pattern_names(pattern, source, facts, TsNameKind::Parameter);
            }
        }
        "switch_case" => {
            facts.dispatch_branches.push(TsDispatchBranchFact {
                span: span_for(node),
                line: line_for(node),
                lines: line_span(node),
            });
        }
        "import_statement" => {
            if let Some(import) = collect_import(node, source) {
                facts.imports.push(import);
            }
        }
        "jsx_opening_element" | "jsx_self_closing_element" => {
            if let Some(element) = collect_jsx_element(node, source) {
                facts.jsx_elements.push(element);
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_node(child, source, facts);
    }
}

fn collect_named_child(
    node: Node<'_>,
    source: &[u8],
    facts: &mut TsFacts,
    field: &str,
    kind: TsNameKind,
) {
    let Some(name) = node.child_by_field_name(field) else {
        return;
    };
    push_name(name, source, facts, kind);
}

fn collect_pattern_names(node: Node<'_>, source: &[u8], facts: &mut TsFacts, kind: TsNameKind) {
    match node.kind() {
        "identifier" | "shorthand_property_identifier_pattern" => {
            push_name(node, source, facts, kind);
        }
        "pair_pattern" => {
            if let Some(value) = node.child_by_field_name("value") {
                collect_pattern_names(value, source, facts, kind);
            }
        }
        "rest_pattern" => {
            if let Some(pattern) = first_named_child(node) {
                collect_pattern_names(pattern, source, facts, kind);
            }
        }
        "array_pattern" | "object_pattern" | "assignment_pattern" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                collect_pattern_names(child, source, facts, kind);
            }
        }
        _ => {}
    }
}

fn push_name(node: Node<'_>, source: &[u8], facts: &mut TsFacts, kind: TsNameKind) {
    let Some(name) = node_text(node, source) else {
        return;
    };
    if name == "this" {
        return;
    }
    facts.names.push(TsNameFact {
        kind,
        name,
        span: span_for(node),
        line: line_for(node),
    });
}

fn collect_import(node: Node<'_>, source: &[u8]) -> Option<TsImportFact> {
    let source_node = node.child_by_field_name("source")?;
    let source_text = string_value(&node_text(source_node, source)?)?;
    let statement_text = node_text(node, source)?;
    let mut specifiers = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "import_clause" {
            collect_import_clause(child, source, &mut specifiers);
        }
    }
    Some(TsImportFact {
        source: source_text,
        type_only: statement_text.trim_start().starts_with("import type "),
        specifiers,
        span: span_for(node),
        line: line_for(node),
    })
}

fn collect_import_clause(node: Node<'_>, source: &[u8], specifiers: &mut Vec<TsImportSpecifier>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                if let Some(name) = node_text(child, source) {
                    specifiers.push(TsImportSpecifier::Default(name));
                }
            }
            "namespace_import" => {
                if let Some(name) =
                    first_named_child(child).and_then(|node| node_text(node, source))
                {
                    specifiers.push(TsImportSpecifier::Namespace(name));
                }
            }
            "named_imports" => collect_named_imports(child, source, specifiers),
            _ => {}
        }
    }
}

fn collect_named_imports(node: Node<'_>, source: &[u8], specifiers: &mut Vec<TsImportSpecifier>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() != "import_specifier" {
            continue;
        }
        let Some(local) = child
            .child_by_field_name("alias")
            .or_else(|| child.child_by_field_name("name"))
            .and_then(|node| node_text(node, source))
        else {
            continue;
        };
        specifiers.push(TsImportSpecifier::Named(local));
    }
}

fn collect_jsx_element(node: Node<'_>, source: &[u8]) -> Option<TsxElementFact> {
    let name = node.child_by_field_name("name")?;
    let text = node_text(name, source)?;
    let intrinsic = intrinsic_name(name, &text);
    Some(TsxElementFact {
        root: jsx_root_name(name, source),
        name: text,
        intrinsic,
        self_closing: node.kind() == "jsx_self_closing_element",
        span: span_for(node),
        line: line_for(node),
    })
}

fn intrinsic_name(node: Node<'_>, text: &str) -> Option<String> {
    match node.kind() {
        "identifier" => text
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_lowercase())
            .then(|| text.to_string()),
        "jsx_namespace_name" => Some(text.to_string()),
        _ => None,
    }
}

fn jsx_root_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    match node.kind() {
        "identifier" => node_text(node, source),
        "member_expression" => node
            .child_by_field_name("object")
            .and_then(|object| jsx_root_name(object, source)),
        _ => None,
    }
}

fn first_named_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    let child = node.named_children(&mut cursor).next();
    child
}

fn node_text(node: Node<'_>, source: &[u8]) -> Option<String> {
    node.utf8_text(source).ok().map(str::to_string)
}

fn string_value(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let quote = trimmed.as_bytes().first().copied()?;
    if !matches!(quote, b'\'' | b'"') || trimmed.as_bytes().last().copied() != Some(quote) {
        return None;
    }
    Some(trimmed[1..trimmed.len().saturating_sub(1)].to_string())
}

fn line_for(node: Node<'_>) -> usize {
    node.start_position().row + 1
}

fn line_span(node: Node<'_>) -> usize {
    node.end_position()
        .row
        .saturating_sub(node.start_position().row)
        + 1
}

fn span_for(node: Node<'_>) -> Span {
    Span::from_usize(node.start_byte(), node.end_byte())
}
