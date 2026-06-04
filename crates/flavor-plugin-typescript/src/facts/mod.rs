use flavor_core::{LineIndex, SourceText, Span, Token};
use flavor_grammar::{GrammarNode, GrammarToken, GrammarTree, TokenTextRun};

use crate::{
    ast::TsSourceFile,
    internal::grammar::{self as kind, Kind},
    model::{
        TsDispatchBranchFact, TsFacts, TsImportFact, TsImportSpecifier, TsNameFact, TsNameKind,
        TsxElementFact,
    },
};

type TsNode = GrammarNode;
type TsToken = GrammarToken;

const BINDING_NODES: &[&str] = &[
    "binding_pattern",
    "object_binding_pattern",
    "array_binding_pattern",
    "rest_element",
];
const IMPORT_SPECIFIER_NODES: &[&str] = &["import_specifier"];
const JSX_JOIN_TOKENS: &[&str] = &["DOT", "COLON", "MINUS"];
const JSX_NAME_PREFIXES: &[&str] = &["KEYWORD_"];
const JSX_NAME_TOKENS: &[&str] = &["IDENTIFIER", "NUMERIC_LITERAL"];
const JSX_SKIP_TOKENS: &[&str] = &["LESS_THAN"];
const NAME_TOKENS: &[&str] = &[
    "IDENTIFIER",
    "KEYWORD_SATISFIES",
    "KEYWORD_KEYOF",
    "KEYWORD_INFER",
    "KEYWORD_UNIQUE",
];
const SPECIFIER_TOKENS: &[&str] = &[
    "IDENTIFIER",
    "KEYWORD_DEFAULT",
    "KEYWORD_SATISFIES",
    "KEYWORD_KEYOF",
    "KEYWORD_INFER",
    "KEYWORD_UNIQUE",
];
pub(crate) fn collect(source_file: &TsSourceFile) -> TsFacts {
    let tree = GrammarTree::new(source_file.syntax().clone(), kind::schema());
    let mut collector = Collector {
        source: source_file.source(),
        line_index: source_file.source().line_index(),
        facts: legacy_counts(source_file.tokens()),
    };
    collector.collect_node(tree.root());
    collector.facts
}

fn legacy_counts(tokens: &[Token<Kind>]) -> TsFacts {
    let mut facts = TsFacts::default();
    for token in tokens {
        match token.kind {
            kind::KEYWORD_IMPORT => facts.import_count += 1,
            kind::KEYWORD_EXPORT => facts.export_count += 1,
            _ => {}
        }
    }
    facts
}

struct Collector<'a> {
    source: &'a SourceText,
    line_index: LineIndex,
    facts: TsFacts,
}

impl Collector<'_> {
    fn collect_node(&mut self, node: TsNode) {
        match node.kind_name() {
            Some("function_declaration" | "function_expression") => {
                self.collect_first_name(&node, TsNameKind::Function);
            }
            Some("method_definition" | "method_declaration" | "method_signature") => {
                self.collect_first_name(&node, TsNameKind::Method);
            }
            Some("variable_declarator" | "variable_declaration") => {
                self.collect_binding(&node, TsNameKind::Binding);
            }
            Some("parameter" | "required_parameter" | "optional_parameter") => {
                self.collect_binding(&node, TsNameKind::Parameter);
            }
            Some("switch_case") => self.collect_branch(&node),
            Some("import_statement" | "import_declaration") => self.collect_import(&node),
            Some("jsx_opening_element" | "jsx_self_closing_element") => {
                self.collect_jsx_element(&node);
            }
            _ => {}
        }

        for child in node.children() {
            self.collect_node(child);
        }
    }

    fn collect_first_name(&mut self, node: &TsNode, kind: TsNameKind) {
        let Some(name) = node.child_tokens_any(NAME_TOKENS).next() else {
            return;
        };
        self.push_name(kind, &name);
    }

    fn collect_binding(&mut self, node: &TsNode, kind: TsNameKind) {
        if let Some(pattern) = node.child_any(BINDING_NODES) {
            self.collect_pattern_names(pattern, kind);
        } else {
            self.collect_first_name(node, kind);
        }
    }

    fn collect_pattern_names(&mut self, node: TsNode, kind: TsNameKind) {
        for name in node.tokens_any(NAME_TOKENS) {
            self.push_name(kind, &name);
        }
    }

    fn collect_branch(&mut self, node: &TsNode) {
        let span = node.trimmed_span(self.source);
        self.facts.dispatch_branches.push(TsDispatchBranchFact {
            span,
            line: self.line_for(span),
            lines: self.line_span(span),
        });
    }

    fn collect_import(&mut self, node: &TsNode) {
        let Some(import) = self.import_fact(node) else {
            return;
        };
        self.facts.imports.push(import);
    }

    fn import_fact(&self, node: &TsNode) -> Option<TsImportFact> {
        let source = node
            .token("STRING_LITERAL")
            .and_then(|token| string_value(token.text()))?;
        let mut specifiers = Vec::new();
        if let Some(clause) = node.child("import_clause") {
            self.collect_import_clause(&clause, &mut specifiers);
        }
        let span = node.trimmed_span(self.source);
        Some(TsImportFact {
            source,
            type_only: node
                .trimmed_source_text(self.source)
                .trim_start()
                .starts_with("import type "),
            specifiers,
            span,
            line: self.line_for(span),
        })
    }

    fn collect_import_clause(&self, node: &TsNode, specifiers: &mut Vec<TsImportSpecifier>) {
        if let Some(default) = default_import(node) {
            specifiers.push(TsImportSpecifier::Default(default));
        }
        for child in node.children() {
            match child.kind_name() {
                Some("namespace_import") => {
                    if let Some(name) = last_specifier_name(&child) {
                        specifiers.push(TsImportSpecifier::Namespace(name));
                    }
                }
                Some("named_imports") => self.collect_named_imports(&child, specifiers),
                _ => {}
            }
        }
    }

    fn collect_named_imports(&self, node: &TsNode, specifiers: &mut Vec<TsImportSpecifier>) {
        for child in node.children_any(IMPORT_SPECIFIER_NODES) {
            if let Some(name) = last_specifier_name(&child) {
                specifiers.push(TsImportSpecifier::Named(name));
            }
        }
    }

    fn collect_jsx_element(&mut self, node: &TsNode) {
        let Some(name) = jsx_name(node) else {
            return;
        };
        let span = node.trimmed_span(self.source);
        self.facts.jsx_elements.push(TsxElementFact {
            root: jsx_root(&name),
            intrinsic: jsx_intrinsic(&name),
            name,
            self_closing: node.has_token("SLASH"),
            span,
            line: self.line_for(span),
        });
    }

    fn push_name(&mut self, kind: TsNameKind, token: &TsToken) {
        if token.text() == "this" {
            return;
        }
        let span = token.span();
        self.facts.names.push(TsNameFact {
            kind,
            name: token.text().to_string(),
            span,
            line: self.line_for(span),
        });
    }

    fn line_for(&self, span: Span) -> usize {
        self.line_index.line(span.start)
    }

    fn line_span(&self, span: Span) -> usize {
        self.line_index
            .line(span.end)
            .saturating_sub(self.line_index.line(span.start))
            + 1
    }
}

fn default_import(node: &TsNode) -> Option<String> {
    node.head_token_text_any(SPECIFIER_TOKENS)
}

fn last_specifier_name(node: &TsNode) -> Option<String> {
    node.last_token_text_any(SPECIFIER_TOKENS)
}

fn string_value(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let quote = trimmed.as_bytes().first().copied()?;
    if !matches!(quote, b'\'' | b'"') || trimmed.as_bytes().last().copied() != Some(quote) {
        return None;
    }
    Some(trimmed[1..trimmed.len().saturating_sub(1)].to_string())
}

fn jsx_name(node: &TsNode) -> Option<String> {
    node.token_run_text(
        TokenTextRun::new(JSX_NAME_TOKENS, JSX_JOIN_TOKENS)
            .with_part_prefixes(JSX_NAME_PREFIXES)
            .with_skip(JSX_SKIP_TOKENS),
    )
}

fn jsx_root(name: &str) -> Option<String> {
    name.split_once('.').map(|(root, _)| root.to_string())
}

fn jsx_intrinsic(name: &str) -> Option<String> {
    let intrinsic = name.contains(':')
        || name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_lowercase());
    intrinsic.then(|| name.to_string())
}
